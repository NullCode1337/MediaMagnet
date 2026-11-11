use axum::extract::{Request, State};
use axum::http::{header, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use std::sync::Arc;

use crate::state::AppState;

/// Constant-time comparison, so a wrong token cannot be found byte by byte.
fn same(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn presented_token(request: &Request) -> Option<String> {
    if let Some(value) = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        return Some(value.trim().to_string());
    }
    // `EventSource` and a plain download link cannot set headers, so those GET
    // routes also accept `?token=`. No other request may carry a token in the
    // URL, so it cannot end up in logs for ordinary API calls.
    if !query_token_allowed(request.method(), request.uri().path()) {
        return None;
    }
    request.uri().query().and_then(|query| {
        query
            .split('&')
            .find_map(|pair| pair.strip_prefix("token="))
            .map(|v| v.to_string())
    })
}

fn query_token_allowed(method: &axum::http::Method, path: &str) -> bool {
    if method != axum::http::Method::GET && method != axum::http::Method::HEAD {
        return false;
    }
    path.ends_with("/events") || path.contains("/files") || path.ends_with("/file")
}

pub async fn require_token(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    let Some(expected) = state.token.as_deref() else {
        return next.run(request).await;
    };

    match presented_token(&request) {
        Some(token) if same(token.as_bytes(), expected.as_bytes()) => next.run(request).await,
        _ => (
            StatusCode::UNAUTHORIZED,
            [(header::WWW_AUTHENTICATE, "Bearer")],
            Json(json!({
                "error": "unauthorized",
                "message": "This server requires a bearer token",
            })),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Method;

    #[test]
    fn comparison_is_exact() {
        assert!(same(b"secret", b"secret"));
        assert!(!same(b"secret", b"secreT"));
        assert!(!same(b"secret", b"secret "));
        assert!(!same(b"", b"s"));
        assert!(same(b"", b""));
    }

    #[test]
    fn query_tokens_are_limited_to_headerless_get_routes() {
        assert!(query_token_allowed(&Method::GET, "/api/v1/events"));
        assert!(query_token_allowed(&Method::GET, "/events"));
        assert!(query_token_allowed(&Method::GET, "/api/v1/files/a.mp4"));
        assert!(query_token_allowed(
            &Method::GET,
            "/api/v1/downloads/job/file"
        ));

        // Mutating endpoints and everything else require the header.
        assert!(!query_token_allowed(&Method::POST, "/api/v1/downloads"));
        assert!(!query_token_allowed(&Method::GET, "/api/v1/settings"));
        assert!(!query_token_allowed(&Method::DELETE, "/api/v1/downloads"));
        assert!(!query_token_allowed(&Method::GET, "/api/v1/cookies"));
    }
}
