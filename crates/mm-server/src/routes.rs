use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::extract::State;
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{middleware, Json, Router};
use futures_util::Stream;
use serde::Deserialize;
use serde_json::{json, Value};

use mm_core::host::disk_usage;
use mm_core::{Backend, DownloadRequest, JobCookie, Settings, Tool};

use crate::auth::require_token;
use crate::files::{self, FILE_CSP};
use crate::state::AppState;

/// For browser cookies transport
const MAX_BODY_BYTES: usize = 1024 * 1024;
const MAX_URL_LEN: usize = 8192;
const MAX_ID_LEN: usize = 128;
const MAX_ACTIVE_JOBS: usize = 100;

const APP_CSP: &str = "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; \
img-src 'self' data: blob:; media-src 'self' blob:; font-src 'self'; connect-src 'self'; \
object-src 'none'; base-uri 'self'; form-action 'self'; frame-ancestors 'none'";

pub fn router(state: Arc<AppState>) -> Router {
    let api = Router::new()
        .route("/health", get(health))
        .route(
            "/downloads",
            get(list_downloads).post(start_download).delete(cancel_all),
        )
        .route("/downloads/{id}", get(get_download).delete(cancel))
        .route("/downloads/{id}/file", get(output_file))
        .route("/downloads/{id}/archive", get(output_archive))
        .route("/downloads/{id}/pause", post(pause))
        .route("/downloads/{id}/resume", post(resume))
        .route("/settings", get(get_settings).put(update_settings))
        .route("/settings/reset", post(reset_settings))
        .route(
            "/cookies",
            get(cookies_unavailable).post(cookies_unavailable),
        )
        .route("/cookies", delete(cookies_unavailable))
        .route("/cookies/{domain}", delete(cookies_unavailable))
        .route("/paths/download-root", get(download_root))
        .route("/paths/disk-usage", get(disk))
        .route("/files", get(files::list))
        .route("/files/{*path}", get(files::download))
        .route("/events", get(events))
        .route("/tools", get(list_tools))
        .route("/tools/update", post(update_tools))
        .route(
            "/torrents",
            get(torrents_unavailable).post(torrents_unavailable),
        )
        .route("/torrents/{id}", delete(torrents_unavailable))
        .layer(axum::extract::DefaultBodyLimit::max(MAX_BODY_BYTES))
        .layer(middleware::from_fn_with_state(
            state.auth.clone(),
            require_token,
        ))
        .with_state(state.clone());

    Router::new()
        .nest("/api/v1", api)
        .fallback(get(crate::routes::spa))
        .layer(middleware::from_fn(security_headers))
        .with_state(state)
}

fn error(status: StatusCode, code: &str, message: impl Into<String>) -> Response {
    (
        status,
        Json(json!({ "error": code, "message": message.into() })),
    )
        .into_response()
}

/// File bytes are served on these API routes; only they get the sandbox policy.
#[doc(hidden)]
pub fn file_route(path: &str) -> bool {
    path.starts_with("/api/") && (path.contains("/files") || path.ends_with("/file"))
}

/// Response headers that apply to every route
async fn security_headers(request: axum::extract::Request, next: middleware::Next) -> Response {
    let path = request.uri().path().to_string();
    let mut response = next.run(request).await;

    let is_html = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("text/html"));
    let is_api = path.starts_with("/api/");
    let is_file_route = file_route(&path);

    let headers = response.headers_mut();
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));

    if is_file_route {
        headers.insert(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(FILE_CSP),
        );
    } else if is_html {
        headers.insert(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(APP_CSP),
        );
    }

    if is_api {
        headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    }

    response
}

async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

#[derive(Debug, Deserialize)]
pub struct DownloadBody {
    pub url: String,
    pub id: Option<String>,
    /// Settings from a client that keeps its own store; absent uses the server's.
    #[serde(default)]
    pub settings: Option<Value>,
    /// Cookies from that client's store; absent uses the server's cookie store.
    #[serde(default)]
    pub cookies: Option<Vec<JobCookie>>,
    /// Forces a backend for this job; absent detects it from the URL.
    #[serde(default)]
    pub backend: Option<Backend>,
    /// Sparse settings patch applied over `settings` (or the server's) for this job.
    #[serde(default)]
    pub overrides: Option<Value>,
}

/// Job ids come from the client so the desktop and browser flows stay identical.
/// A client that omits one gets a server-assigned id back.
async fn start_download(
    State(state): State<Arc<AppState>>,
    Json(body): Json<DownloadBody>,
) -> Response {
    let url = body.url.trim().to_string();
    if url.is_empty() {
        return error(StatusCode::BAD_REQUEST, "invalid_url", "url is required");
    }
    if !url.to_lowercase().starts_with("http://") && !url.to_lowercase().starts_with("https://") {
        return error(
            StatusCode::BAD_REQUEST,
            "invalid_url",
            "url must be http or https",
        );
    }
    if url.len() > MAX_URL_LEN || url.chars().any(char::is_control) {
        return error(
            StatusCode::BAD_REQUEST,
            "invalid_url",
            format!("url must be at most {MAX_URL_LEN} characters without control characters"),
        );
    }

    let settings = match body.settings.as_ref().map(Settings::from_value).transpose() {
        Ok(settings) => settings,
        Err(e) => return error(StatusCode::BAD_REQUEST, "invalid_settings", e),
    };
    if let Some(cookies) = &body.cookies {
        for cookie in cookies {
            if let Err(e) = mm_core::cookies::validate_domain(&cookie.domain) {
                return error(StatusCode::BAD_REQUEST, "invalid_cookie", e.to_string());
            }
        }
    }
    if let Some(overrides) = &body.overrides {
        if !overrides.is_object() {
            return error(
                StatusCode::BAD_REQUEST,
                "invalid_overrides",
                "overrides must be an object",
            );
        }
    }
    if let Some(backend) = body.backend {
        if let Err(e) = state.engine.ensure_backend(backend).await {
            return error(
                StatusCode::SERVICE_UNAVAILABLE,
                "backend_unavailable",
                e.to_string(),
            );
        }
    }

    let id = match body.id.filter(|id| !id.trim().is_empty()) {
        Some(id) => {
            if !valid_job_id(&id) {
                return error(
                    StatusCode::BAD_REQUEST,
                    "invalid_id",
                    format!(
                        "id must be at most {MAX_ID_LEN} characters of A-Z, a-z, 0-9, '-', '_', '.' or ':'"
                    ),
                );
            }
            id
        }
        None => new_id(),
    };

    if state.jobs.active_count() >= MAX_ACTIVE_JOBS {
        return error(
            StatusCode::TOO_MANY_REQUESTS,
            "too_many_downloads",
            format!("{MAX_ACTIVE_JOBS} downloads are already queued; wait for one to finish"),
        );
    }

    let request = DownloadRequest {
        id: id.clone(),
        url,
        backend: body.backend,
        base_dir: state.download_root.clone(),
        settings,
        cookies: body.cookies,
        overrides: body.overrides,
    };
    if !state.jobs.insert(request.clone()) {
        return error(
            StatusCode::CONFLICT,
            "duplicate_download",
            format!("download {id} is already running or paused"),
        );
    }

    let engine = state.engine.clone();
    tokio::spawn(async move {
        engine.run_job(request).await;
    });

    (StatusCode::ACCEPTED, Json(json!({ "id": id }))).into_response()
}

async fn list_downloads(State(state): State<Arc<AppState>>) -> Json<Value> {
    Json(json!({ "jobs": state.jobs.list() }))
}

async fn get_download(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response {
    match state.jobs.summary(&id) {
        Some(job) => Json(job).into_response(),
        None => error(
            StatusCode::NOT_FOUND,
            "no_such_download",
            format!("no download with id: {id}"),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct OutputFileQuery {
    /// Picks one file when the job produced several.
    #[serde(default)]
    pub path: Option<String>,
}

/// Streams the file a finished job produced, as an attachment. Jobs that wrote
/// several files need `?path=`; the names are also on `download-finished`.
async fn output_file(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
    axum::extract::Query(query): axum::extract::Query<OutputFileQuery>,
    headers: axum::http::HeaderMap,
) -> Response {
    let Some(job) = state.jobs.summary(&id) else {
        return error(
            StatusCode::NOT_FOUND,
            "no_such_download",
            format!("no download with id: {id}"),
        );
    };

    let relative = match query.path {
        Some(path) => {
            if !job.files.iter().any(|file| file == &path) {
                return error(
                    StatusCode::NOT_FOUND,
                    "no_such_file",
                    format!("download {id} produced no file: {path}"),
                );
            }
            path
        }
        None => match job.files.as_slice() {
            [] => {
                return error(
                    StatusCode::NOT_FOUND,
                    "no_output_file",
                    format!("download {id} has no recorded output file; is it still running?"),
                )
            }
            [only] => only.clone(),
            many => {
                return (
                    StatusCode::CONFLICT,
                    Json(json!({
                        "error": "multiple_files",
                        "message": format!(
                            "download {id} produced {} files; pass ?path=",
                            many.len()
                        ),
                        "files": many,
                    })),
                )
                    .into_response()
            }
        },
    };

    let mut response = files::serve(
        &state.download_root,
        &relative,
        headers,
        files::Disposition::Attachment,
        Some(files::DeleteAfter {
            retention: state.retention.clone(),
            job_id: id,
        }),
    )
    .await;
    if response.status().is_success() {
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/octet-stream"),
        );
    }
    response
}

/// Streams every file a finished multi-file job produced as one ZIP archive.
async fn output_archive(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
    headers: HeaderMap,
) -> Response {
    let Some(job) = state.jobs.summary(&id) else {
        return error(
            StatusCode::NOT_FOUND,
            "no_such_download",
            format!("no download with id: {id}"),
        );
    };
    if job.files.is_empty() {
        return error(
            StatusCode::NOT_FOUND,
            "no_output_file",
            format!("download {id} has no recorded output files"),
        );
    }

    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or(0);
    let zip_name = format!(".mm-archive-{id}-{stamp}.zip");
    let zip_path = state.download_root.join(&zip_name);
    let root = state.download_root.clone();
    let files = job.files.clone();

    let built = tokio::task::spawn_blocking(move || -> std::result::Result<(), String> {
        let root = root.canonicalize().map_err(|e| e.to_string())?;
        let file = std::fs::File::create(&zip_path).map_err(|e| e.to_string())?;
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        for relative in files {
            let name = relative.replace('\\', "/");
            let name = name.trim_start_matches("./");
            if name.is_empty() || name.split('/').any(|part| part.is_empty() || part == "..") {
                continue;
            }
            let Ok(path) = root.join(name).canonicalize() else {
                continue;
            };
            if !path.starts_with(&root) || !path.is_file() {
                continue;
            }
            let Ok(mut input) = std::fs::File::open(&path) else {
                continue;
            };
            zip.start_file(name, options).map_err(|e| e.to_string())?;
            std::io::copy(&mut input, &mut zip).map_err(|e| e.to_string())?;
        }

        zip.finish().map_err(|e| e.to_string())?;
        Ok(())
    })
    .await;

    match built {
        Ok(Ok(())) => {
            state.retention.record(&id, std::slice::from_ref(&zip_name));
            let mut response = files::serve(
                &state.download_root,
                &zip_name,
                headers,
                files::Disposition::Attachment,
                Some(files::DeleteAfter {
                    retention: state.retention.clone(),
                    job_id: id,
                }),
            )
            .await;
            if response.status().is_success() {
                response.headers_mut().insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/zip"),
                );
            }
            response
        }
        Ok(Err(e)) => error(StatusCode::INTERNAL_SERVER_ERROR, "archive_failed", e),
        Err(e) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "archive_failed",
            e.to_string(),
        ),
    }
}

/// Replays the stored request of a paused job. Settings, cookies and the URL
/// are the ones the job was created with.
async fn resume(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response {
    let Some(request) = state.jobs.take_for_resume(&id) else {
        return match state.jobs.summary(&id) {
            Some(_) => error(
                StatusCode::CONFLICT,
                "not_paused",
                format!("download {id} is not paused"),
            ),
            None => error(
                StatusCode::NOT_FOUND,
                "no_such_download",
                format!("no download with id: {id}"),
            ),
        };
    };

    let engine = state.engine.clone();
    tokio::spawn(async move {
        engine.run_job(request).await;
    });

    (StatusCode::ACCEPTED, Json(json!({ "id": id }))).into_response()
}

fn new_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!(
        "srv-{nanos:x}-{:x}",
        COUNTER.fetch_add(1, Ordering::Relaxed)
    )
}

#[doc(hidden)]
pub fn valid_job_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= MAX_ID_LEN
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}

async fn pause(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response {
    match state.engine.pause(&id).await {
        Ok(()) => {
            state.jobs.set_state(&id, "paused");
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => error(StatusCode::NOT_FOUND, "no_such_download", e.to_string()),
    }
}

async fn cancel(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response {
    // `Engine::cancel` reports success for unknown ids but still broadcasts a
    // "cancelled" error to every SSE client, so unknown ids stop here.
    if !state.engine.active_ids().iter().any(|active| active == &id)
        && !state.engine.is_retained(&id)
    {
        return error(
            StatusCode::NOT_FOUND,
            "no_such_download",
            format!("no active download with id: {id}"),
        );
    }
    match state.engine.cancel(&id).await {
        Ok(()) => {
            state.jobs.set_state(&id, "cancelled");
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => error(StatusCode::NOT_FOUND, "no_such_download", e.to_string()),
    }
}

async fn cancel_all(State(state): State<Arc<AppState>>) -> Response {
    let active = state.engine.active_ids();
    match state.engine.cancel_all().await {
        Ok(()) => {
            for id in active {
                state.jobs.set_state(&id, "cancelled");
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "cancel_failed",
            e.to_string(),
        ),
    }
}

async fn list_tools(State(state): State<Arc<AppState>>) -> Json<Value> {
    let runtime = state.engine.runtime();
    let mut tools = Vec::with_capacity(Tool::all().len());
    for tool in Tool::all() {
        tools.push(runtime.status(tool).await);
    }

    Json(json!({
        "tools": tools,
        "deno": runtime.deno().map(|path| path.display().to_string()),
        "ffmpeg": runtime.ffmpeg().map(|path| path.display().to_string()),
    }))
}

/// Installs the pinned Python tools into the writable overlay. Fails when the
/// host has no bundled or system interpreter (for example the Docker image,
/// where the tools are preinstalled).
///
/// `Accept: application/x-ndjson` streams output as it arrives, ending with a
/// `done` record; any other request keeps the original JSON response.
async fn update_tools(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let streaming = headers
        .get(header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.contains("application/x-ndjson"));

    if !streaming {
        return match state
            .engine
            .runtime()
            .reinstall_python_deps(&Tool::all())
            .await
        {
            Ok(log) => {
                let updated: Vec<&str> = Tool::all().into_iter().map(Tool::name).collect();
                Json(json!({ "updated": updated, "output": log })).into_response()
            }
            Err(e) => error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "tool_update_failed",
                e.to_string(),
            ),
        };
    }

    let engine = state.engine.clone();
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    tokio::spawn(async move {
        let chunks = tx.clone();
        let result = engine
            .runtime()
            .reinstall_python_deps_streaming(&Tool::all(), move |chunk| {
                let line = json!({ "event": "output", "data": chunk }).to_string();
                let _ = chunks.send(format!("{line}\n"));
            })
            .await;

        let line = match result {
            Ok(log) => {
                let updated: Vec<&str> = Tool::all().into_iter().map(Tool::name).collect();
                json!({ "event": "done", "ok": true, "updated": updated, "output": log })
            }
            Err(e) => json!({ "event": "done", "ok": false, "error": e.to_string() }),
        };
        let _ = tx.send(format!("{line}\n"));
    });

    let stream = futures_util::stream::unfold(rx, |mut rx| async move {
        rx.recv().await.map(|line| (Ok::<_, Infallible>(line), rx))
    });

    Response::builder()
        .header(header::CONTENT_TYPE, "application/x-ndjson")
        .header(header::CACHE_CONTROL, "no-store")
        .body(Body::from_stream(stream))
        .unwrap_or_else(|_| {
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "tool_update_failed",
                "could not build update stream",
            )
        })
}

/// Torrent support is not implemented; the audit records this as a later
/// engine behind the same routes.
async fn torrents_unavailable() -> Response {
    error(
        StatusCode::NOT_IMPLEMENTED,
        "torrents_unavailable",
        "no torrent engine is configured on this server",
    )
}

async fn get_settings(State(state): State<Arc<AppState>>) -> Json<Value> {
    Json(state.engine.settings().server_view())
}

async fn update_settings(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Response {
    let mut next = match Settings::from_value(&body) {
        Ok(next) => next,
        Err(e) => return error(StatusCode::BAD_REQUEST, "invalid_settings", e),
    };

    // Desktop-only fields are not writable over HTTP, even if a client sends them.
    let current = state.engine.settings();
    mm_core::force_desktop_fields(&mut next, &current);

    match state.engine.save_settings(&next) {
        Ok(()) => Json(next.server_view()).into_response(),
        Err(e) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "save_failed",
            e.to_string(),
        ),
    }
}

async fn reset_settings(State(state): State<Arc<AppState>>) -> Response {
    let defaults = Settings::default();
    let current = state.engine.settings();
    let mut next = defaults;
    mm_core::force_desktop_fields(&mut next, &current);

    match state.engine.save_settings(&next) {
        Ok(()) => Json(next.server_view()).into_response(),
        Err(e) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "save_failed",
            e.to_string(),
        ),
    }
}

async fn cookies_unavailable() -> Response {
    error(
        StatusCode::NOT_IMPLEMENTED,
        "cookies_unavailable",
        "this server stores no cookies; send `cookies` with each download",
    )
}

async fn download_root(State(state): State<Arc<AppState>>) -> Json<Value> {
    Json(json!({ "path": state.download_root.to_string_lossy() }))
}

async fn disk(State(state): State<Arc<AppState>>) -> Response {
    match disk_usage(&state.download_root) {
        Ok(usage) => Json(json!({ "used_percent": usage.used_percent })).into_response(),
        Err(e) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "disk_usage_failed",
            e.to_string(),
        ),
    }
}

async fn events(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.events.subscribe();

    // One subscription for the life of the connection. A lagged receiver skips
    // the events it missed rather than ending the stream.
    let stream = futures_util::stream::unfold(rx, |mut rx| async move {
        loop {
            match rx.recv().await {
                Ok(value) => {
                    let kind = value
                        .get("type")
                        .and_then(Value::as_str)
                        .unwrap_or("message")
                        .to_string();
                    let event = Event::default().event(kind).data(value.to_string());
                    return Some((Ok(event), rx));
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => return None,
            }
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(20)))
}

/// Serves `index.html` for anything that is not an API route, so client-side
/// navigation works on a hard reload.
async fn spa(State(state): State<Arc<AppState>>, request: axum::extract::Request) -> Response {
    let path = request.uri().path().trim_start_matches('/');

    // Unknown API paths are not SPA routes: an HTML 200 would look like a
    // successful call to an API client that only checks the status.
    if path.starts_with("api/") {
        return error(StatusCode::NOT_FOUND, "not_found", "no such API route");
    }

    // A traversal attempt must not fall through to `index.html`.
    if path.contains("..") {
        return error(StatusCode::NOT_FOUND, "not_found", "no such file");
    }

    if !path.is_empty() {
        if let Some(bytes) = state.static_assets.read(path).await {
            // Vite hashes `assets/`; the entry HTML must be revalidated.
            let cache = if path.starts_with("assets/") {
                "public, max-age=31536000, immutable"
            } else {
                "no-cache"
            };
            return (
                [
                    (header::CONTENT_TYPE, mime_for(std::path::Path::new(path))),
                    (header::CACHE_CONTROL, cache),
                ],
                axum::body::Body::from(bytes),
            )
                .into_response();
        }
    }

    // A missing asset is a broken build, not a route: hand back the SPA only
    // for paths that look like pages.
    let looks_like_asset = path.starts_with("assets/") || path.contains('.');
    if looks_like_asset {
        return error(StatusCode::NOT_FOUND, "not_found", "no such file");
    }

    match state.static_assets.read("index.html").await {
        Some(bytes) => (
            [
                (header::CONTENT_TYPE, "text/html; charset=utf-8"),
                (header::CACHE_CONTROL, "no-cache"),
            ],
            axum::body::Body::from(bytes),
        )
            .into_response(),
        None => error(
            StatusCode::NOT_FOUND,
            "not_found",
            format!(
                "no route and no index.html in {}",
                state.static_assets.describe()
            ),
        ),
    }
}

fn mime_for(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("ico") => "image/x-icon",
        Some("woff2") => "font/woff2",
        Some("wasm") => "application/wasm",
        _ => "application/octet-stream",
    }
}
