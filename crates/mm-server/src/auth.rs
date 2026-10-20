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
    // `EventSource` cannot set headers, so the SSE endpoint accepts `?token=`.
    // Pair `MM_TOKEN` with a TLS proxy that does not log query strings.
    request.uri().query().and_then(|query| {
        query
            .split('&')
            .find_map(|pair| pair.strip_prefix("token="))
            .map(|v| v.to_string())
    })
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
    use super::same;

    #[test]
    fn comparison_is_exact() {
        assert!(same(b"secret", b"secret"));
        assert!(!same(b"secret", b"secreT"));
        assert!(!same(b"secret", b"secret "));
        assert!(!same(b"", b"s"));
        assert!(same(b"", b""));
    }
}
