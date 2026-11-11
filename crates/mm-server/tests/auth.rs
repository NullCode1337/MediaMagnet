use axum::http::Method;

use mm_server::testing::{query_token_allowed, same};

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
