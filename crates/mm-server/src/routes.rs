use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{middleware, Json, Router};
use futures_util::Stream;
use serde::Deserialize;
use serde_json::{json, Value};

use mm_core::cookies::CookieInput;
use mm_core::host::disk_usage;
use mm_core::{DownloadRequest, JobCookie, Settings, Tool, ToolSource};

use crate::auth::require_token;
use crate::files;
use crate::state::AppState;

pub fn router(state: Arc<AppState>) -> Router {
    let api = Router::new()
        .route("/health", get(health))
        .route(
            "/downloads",
            get(list_downloads).post(start_download).delete(cancel_all),
        )
        .route("/downloads/{id}", get(get_download).delete(cancel))
        .route("/downloads/{id}/pause", post(pause))
        .route("/downloads/{id}/resume", post(resume))
        .route("/settings", get(get_settings).put(update_settings))
        .route("/settings/reset", post(reset_settings))
        .route("/cookies", get(list_cookies).post(save_cookie))
        .route("/cookies", delete(clear_cookies))
        .route("/cookies/{domain}", delete(delete_cookie))
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
        .layer(middleware::from_fn_with_state(state.clone(), require_token))
        .with_state(state.clone());

    Router::new()
        .nest("/api/v1", api)
        .fallback(get(crate::routes::spa))
        .with_state(state)
}

fn error(status: StatusCode, code: &str, message: impl Into<String>) -> Response {
    (
        status,
        Json(json!({ "error": code, "message": message.into() })),
    )
        .into_response()
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

    let id = body
        .id
        .filter(|id| !id.trim().is_empty())
        .unwrap_or_else(new_id);

    let request = DownloadRequest {
        id: id.clone(),
        url,
        backend: None,
        base_dir: state.download_root.clone(),
        settings,
        cookies: body.cookies,
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
    if !state.engine.active_ids().iter().any(|active| active == &id) {
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

fn source_name(source: ToolSource) -> &'static str {
    match source {
        ToolSource::Preinstalled => "preinstalled",
        ToolSource::System => "system",
        ToolSource::Bundled => "bundled",
        ToolSource::Overlay => "overlay",
    }
}

async fn list_tools(State(state): State<Arc<AppState>>) -> Json<Value> {
    let runtime = state.engine.runtime();
    let mut tools = Vec::new();
    for tool in Tool::all() {
        let status = match runtime.resolve(tool).await {
            Ok(resolved) => json!({
                "name": tool.name(),
                "available": true,
                "source": source_name(resolved.source),
                "program": resolved.program.display().to_string(),
                "args": resolved.args,
            }),
            Err(e) => json!({
                "name": tool.name(),
                "available": false,
                "source": Value::Null,
                "program": Value::Null,
                "args": [],
                "error": e.to_string(),
            }),
        };
        tools.push(status);
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
async fn update_tools(State(state): State<Arc<AppState>>) -> Response {
    match state
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
    }
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

async fn list_cookies(State(state): State<Arc<AppState>>) -> Response {
    match mm_core::cookies::list_cookies(&state.engine.cookies_dir()) {
        Ok(map) => Json(map).into_response(),
        Err(e) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "cookie_read_failed",
            e.to_string(),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct SaveCookieBody {
    pub domain: String,
    pub input: CookieInput,
}

async fn save_cookie(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SaveCookieBody>,
) -> Response {
    match mm_core::cookies::save_cookie(&state.engine.cookies_dir(), &body.domain, &body.input) {
        Ok(path) => Json(json!({ "domain": body.domain, "path": path })).into_response(),
        Err(e) => error(StatusCode::BAD_REQUEST, "invalid_cookie", e.to_string()),
    }
}

async fn clear_cookies(State(state): State<Arc<AppState>>) -> Response {
    match mm_core::cookies::clear_cookies(&state.engine.cookies_dir()) {
        Ok(deleted) => Json(json!({ "deleted": deleted })).into_response(),
        Err(e) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "cookie_clear_failed",
            e.to_string(),
        ),
    }
}

async fn delete_cookie(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(domain): axum::extract::Path<String>,
) -> Response {
    let dir = state.engine.cookies_dir();
    let path = match mm_core::cookies::cookie_path(&dir, &domain) {
        Ok(path) => path,
        Err(e) => return error(StatusCode::BAD_REQUEST, "invalid_cookie", e.to_string()),
    };
    match mm_core::cookies::delete_cookie(&dir, &path) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => error(StatusCode::NOT_FOUND, "no_such_cookie", e.to_string()),
    }
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

    // A traversal attempt must not fall through to `index.html`.
    if path.contains("..") {
        return error(StatusCode::NOT_FOUND, "not_found", "no such file");
    }

    if !path.is_empty() {
        if let Some(bytes) = state.static_assets.read(path).await {
            return (
                [(header::CONTENT_TYPE, mime_for(std::path::Path::new(path)))],
                bytes,
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
        Some(bytes) => {
            ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], bytes).into_response()
        }
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
