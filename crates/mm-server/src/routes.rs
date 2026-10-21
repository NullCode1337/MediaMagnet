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
use mm_core::{DownloadRequest, JobCookie, Settings};

use crate::auth::require_token;
use crate::files;
use crate::state::AppState;

pub fn router(state: Arc<AppState>) -> Router {
    let api = Router::new()
        .route("/health", get(health))
        .route("/downloads", post(start_download))
        .route("/downloads/{id}/pause", post(pause))
        .route("/downloads/{id}", delete(cancel))
        .route("/downloads", delete(cancel_all))
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

    let engine = state.engine.clone();
    let base_dir = state.download_root.clone();
    let job_id = id.clone();
    let job_settings = settings;
    let job_cookies = body.cookies;
    tokio::spawn(async move {
        engine
            .run_job(DownloadRequest {
                id: job_id,
                url,
                backend: None,
                base_dir,
                settings: job_settings,
                cookies: job_cookies,
            })
            .await;
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
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
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
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => error(StatusCode::NOT_FOUND, "no_such_download", e.to_string()),
    }
}

async fn cancel_all(State(state): State<Arc<AppState>>) -> Response {
    match state.engine.cancel_all().await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "cancel_failed",
            e.to_string(),
        ),
    }
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

    let candidate = if path.is_empty() {
        state.static_dir.join("index.html")
    } else {
        state.static_dir.join(path)
    };

    if !path.is_empty() && candidate.is_file() {
        let body = tokio::fs::read(&candidate).await;
        if let Ok(bytes) = body {
            return ([(header::CONTENT_TYPE, mime_for(&candidate))], bytes).into_response();
        }
    }

    // A missing asset is a broken build, not a route: hand back the SPA only
    // for paths that look like pages.
    let looks_like_asset = path.starts_with("assets/") || path.contains('.');
    if looks_like_asset {
        return error(StatusCode::NOT_FOUND, "not_found", "no such file");
    }

    match tokio::fs::read(state.static_dir.join("index.html")).await {
        Ok(bytes) => ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], bytes).into_response(),
        Err(_) => error(
            StatusCode::NOT_FOUND,
            "not_found",
            format!(
                "no route and no index.html under {}",
                state.static_dir.display()
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
