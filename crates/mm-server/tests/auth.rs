use std::net::{IpAddr, Ipv4Addr};
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use axum::middleware;
use axum::routing::get;
use axum::Router;
use tower::ServiceExt;

use mm_server::testing::{
    query_token_allowed, require_token, same, token_is_weak, AuthGuard, AuthLimiter, AuthPolicy,
};

const TOKEN: &str = "correct-horse-battery-staple";

fn policy() -> AuthPolicy {
    AuthPolicy {
        max_failures: 10,
        window: Duration::from_secs(60),
        lockout: Duration::from_secs(30),
        trust_proxy: false,
    }
}

fn guard(token: Option<&str>, policy: AuthPolicy) -> AuthGuard {
    AuthGuard::new(token.map(str::to_string), policy)
}

fn app(auth: AuthGuard) -> Router {
    // Mirrors production: the auth layer is inside `nest`, so it sees paths
    // with `/api/v1` already stripped.
    let api = Router::new()
        .route("/health", get(|| async { "health" }))
        .route("/settings", get(|| async { "settings" }))
        .route("/events", get(|| async { "events" }))
        .route("/files", get(|| async { "listing" }))
        .route("/files/{*path}", get(|| async { "file" }))
        .route("/downloads/{id}/file", get(|| async { "job file" }))
        .route("/downloads/{id}/archive", get(|| async { "archive" }))
        .route(
            "/downloads",
            get(|| async { "downloads" }).post(|| async { "start" }),
        )
        .layer(middleware::from_fn_with_state(auth, require_token));
    Router::new().nest("/api/v1", api)
}

async fn status(app: &Router, method: Method, uri: &str, token: Option<&str>) -> StatusCode {
    let mut request = Request::builder().method(method).uri(uri);
    if let Some(token) = token {
        request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    app.clone()
        .oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap()
        .status()
}

#[test]
fn comparison_is_exact_and_length_hiding() {
    assert!(same(b"secret", b"secret"));
    assert!(!same(b"secret", b"secreT"));
    assert!(!same(b"secret", b"secret "));
    assert!(!same(b"", b"s"));
    assert!(same(b"", b""));
    // Different lengths are compared as fixed-size digests, not byte by byte.
    assert!(!same(b"a", b"aaaaaaaaaaaaaaaa"));
}

#[test]
fn query_tokens_are_limited_to_headerless_get_routes() {
    // Allowed: the SSE stream and plain download links, with or without the
    // `/api/v1` prefix the nested router strips.
    for path in [
        "/events",
        "/api/v1/events",
        "/files/a.mp4",
        "/api/v1/files/a.mp4",
        "/files/set/one.jpg",
        "/downloads/job-1/file",
        "/api/v1/downloads/job-1/file",
        "/downloads/job-1/archive",
        "/api/v1/downloads/job-1/archive",
    ] {
        assert!(
            query_token_allowed(&Method::GET, path),
            "{path} should allow a query token"
        );
    }
    assert!(query_token_allowed(&Method::HEAD, "/api/v1/events"));

    // Rejected: the listing, look-alike paths and everything mutating.
    for path in [
        "/files",
        "/api/v1/files",
        "/files/",
        "/api/v1/files/",
        "/files-list",
        "/api/v1/files-list",
        "/settings?files",
        "/api/v1/settings?files",
        "/health",
        "/api/v1/health",
        "/cookies",
        "/downloads/job-1",
        "/api/v1/downloads/job-1",
        "/downloads/a/b/file",
        "/api/v1/downloads/a/b/file",
    ] {
        assert!(
            !query_token_allowed(&Method::GET, path),
            "{path} must not allow a query token"
        );
    }
    for method in [Method::POST, Method::PUT, Method::DELETE, Method::PATCH] {
        assert!(!query_token_allowed(&method, "/api/v1/downloads"));
        assert!(!query_token_allowed(
            &method,
            "/api/v1/downloads/job-1/file"
        ));
        assert!(!query_token_allowed(
            &method,
            "/api/v1/downloads/job-1/archive"
        ));
        assert!(!query_token_allowed(&method, "/api/v1/events"));
    }
}

#[tokio::test]
async fn header_token_is_required_and_parsed_strictly() {
    let app = app(guard(Some(TOKEN), policy()));

    assert_eq!(
        status(&app, Method::GET, "/api/v1/health", None).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        status(&app, Method::GET, "/api/v1/health", Some("wrong")).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        status(&app, Method::GET, "/api/v1/health", Some(TOKEN)).await,
        StatusCode::OK
    );

    // The scheme is case-insensitive, and extra whitespace is harmless.
    for value in [
        format!("bearer {TOKEN}"),
        format!("BEARER {TOKEN}"),
        format!("Bearer    {TOKEN}"),
    ] {
        let request = Request::builder()
            .uri("/api/v1/health")
            .header(header::AUTHORIZATION, value)
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            app.clone().oneshot(request).await.unwrap().status(),
            StatusCode::OK
        );
    }

    // Other schemes, incomplete credentials and oversized values never
    // authenticate.
    let oversized = format!("Bearer {}", "a".repeat(5000));
    for value in [
        format!("Basic {TOKEN}"),
        "Bearer".to_string(),
        String::new(),
        format!("Bearer {TOKEN} extra"),
        TOKEN.to_string(),
        oversized,
    ] {
        let request = Request::builder()
            .uri("/api/v1/health")
            .header(header::AUTHORIZATION, value)
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    // Duplicate Authorization headers are ambiguous and must never authenticate.
    let mut request = Request::builder()
        .uri("/api/v1/health")
        .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
        .body(Body::empty())
        .unwrap();
    request.headers_mut().append(
        header::AUTHORIZATION,
        axum::http::HeaderValue::from_str(&format!("Bearer {TOKEN}")).unwrap(),
    );
    assert_eq!(
        app.clone().oneshot(request).await.unwrap().status(),
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn query_token_only_works_on_allowed_routes() {
    let app = app(guard(Some(TOKEN), policy()));

    for uri in [
        format!("/api/v1/events?token={TOKEN}"),
        format!("/api/v1/files/a.mp4?token={TOKEN}"),
        format!("/api/v1/downloads/job-1/file?token={TOKEN}"),
        format!("/api/v1/downloads/job-1/archive?token={TOKEN}"),
    ] {
        assert_eq!(
            status(&app, Method::GET, &uri, None).await,
            StatusCode::OK,
            "{uri} should accept the query token"
        );
    }

    for uri in [
        format!("/api/v1/files?token={TOKEN}"),
        format!("/api/v1/health?token={TOKEN}"),
        format!("/api/v1/settings?token={TOKEN}"),
        format!("/api/v1/downloads?token={TOKEN}"),
        "/api/v1/files/a.mp4?token=wrong".to_string(),
        // A second parameter must not rescue a wrong first one.
        format!("/api/v1/files/a.mp4?token=wrong&token={TOKEN}"),
    ] {
        assert_eq!(
            status(&app, Method::GET, &uri, None).await,
            StatusCode::UNAUTHORIZED,
            "{uri} must not accept the query token"
        );
    }

    // Mutating methods cannot carry a token in the URL.
    assert_eq!(
        status(
            &app,
            Method::POST,
            &format!("/api/v1/downloads?token={TOKEN}"),
            None
        )
        .await,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn query_token_is_percent_decoded() {
    let token = "s3cr3t+/=";
    let app = app(guard(Some(token), policy()));
    assert_eq!(
        status(
            &app,
            Method::GET,
            "/api/v1/files/a.mp4?token=s3cr3t%2B%2F%3D",
            None
        )
        .await,
        StatusCode::OK
    );
    // A literal `+` is a space in form encoding, not part of this token.
    assert_eq!(
        status(
            &app,
            Method::GET,
            "/api/v1/files/a.mp4?token=s3cr3t+%2F%3D",
            None
        )
        .await,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn repeated_failures_lock_out_the_client() {
    let mut policy = policy();
    policy.max_failures = 2;
    let app = app(guard(Some(TOKEN), policy));

    assert_eq!(
        status(&app, Method::GET, "/api/v1/health", Some("wrong")).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        status(&app, Method::GET, "/api/v1/health", Some("wrong")).await,
        StatusCode::TOO_MANY_REQUESTS
    );
    // The correct token is refused while the address is locked out.
    assert_eq!(
        status(&app, Method::GET, "/api/v1/health", Some(TOKEN)).await,
        StatusCode::TOO_MANY_REQUESTS
    );
}

#[tokio::test]
async fn stream_failures_do_not_count_toward_the_lockout() {
    let mut policy = policy();
    policy.max_failures = 2;
    let app = app(guard(Some(TOKEN), policy));

    // `EventSource` retries a failed stream every few seconds; none of these
    // attempts may consume the budget, or a stale tab would lock out the address.
    for _ in 0..10 {
        assert_eq!(
            status(&app, Method::GET, "/api/v1/events", Some("stale")).await,
            StatusCode::UNAUTHORIZED
        );
    }

    // The budget is untouched, so the next real failures still trip it.
    assert_eq!(
        status(&app, Method::GET, "/api/v1/health", Some("wrong")).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        status(&app, Method::GET, "/api/v1/health", Some("wrong")).await,
        StatusCode::TOO_MANY_REQUESTS
    );

    // A locked-out address cannot stream either, even with the right token.
    assert_eq!(
        status(&app, Method::GET, "/api/v1/events", Some(TOKEN)).await,
        StatusCode::TOO_MANY_REQUESTS
    );
}

#[tokio::test]
async fn an_open_server_ignores_tokens_entirely() {
    let app = app(guard(None, policy()));
    assert_eq!(
        status(&app, Method::GET, "/api/v1/health", None).await,
        StatusCode::OK
    );
}

#[test]
fn limiter_blocks_then_recovers() {
    let mut policy = policy();
    policy.max_failures = 3;
    let limiter = AuthLimiter::new(policy);
    let ip = IpAddr::V4(Ipv4Addr::LOCALHOST);
    let start = Instant::now();

    assert!(limiter.check(ip, start).is_ok());
    assert_eq!(limiter.failure(ip, start).failures, 1);
    assert_eq!(limiter.failure(ip, start).failures, 2);
    let trip = limiter.failure(ip, start);
    assert_eq!(trip.blocked_for, Some(policy.lockout));

    assert_eq!(
        limiter.check(ip, start + Duration::from_secs(1)),
        Err(policy.lockout - Duration::from_secs(1))
    );
    // The lockout expires on its own.
    assert!(limiter
        .check(ip, start + policy.lockout + Duration::from_secs(1))
        .is_ok());
    assert_eq!(
        limiter
            .failure(ip, start + policy.lockout + Duration::from_secs(1))
            .failures,
        1
    );
}

#[test]
fn limiter_resets_on_success_and_window_expiry() {
    let mut policy = policy();
    policy.max_failures = 3;
    let limiter = AuthLimiter::new(policy);
    let ip = IpAddr::V4(Ipv4Addr::LOCALHOST);
    let start = Instant::now();

    limiter.failure(ip, start);
    limiter.failure(ip, start);
    limiter.success(ip);
    assert_eq!(limiter.failure(ip, start).failures, 1);

    // A window with no failures starts a new count.
    assert_eq!(
        limiter
            .failure(ip, start + policy.window + Duration::from_secs(1))
            .failures,
        1
    );
}

#[test]
fn another_address_is_not_penalised() {
    let mut policy = policy();
    policy.max_failures = 1;
    let limiter = AuthLimiter::new(policy);
    let attacker = IpAddr::V4(Ipv4Addr::new(203, 0, 113, 7));
    let other = IpAddr::V4(Ipv4Addr::new(198, 51, 100, 9));
    let now = Instant::now();

    assert!(limiter.failure(attacker, now).blocked_for.is_some());
    assert!(limiter.check(other, now).is_ok());
}

#[test]
fn weak_tokens_are_detected() {
    for weak in [
        "",
        "short",
        "change-me",
        "CHANGEME",
        "password",
        "secret",
        "test",
    ] {
        assert!(token_is_weak(weak), "{weak:?} should be weak");
    }
    for strong in [
        "correct-horse-battery-staple",
        "0123456789abcdef0123456789abcdef",
        "9f2c1e7a4b8d0f3c6a5e2d1b9c8f7a6e",
    ] {
        assert!(!token_is_weak(strong), "{strong:?} should be accepted");
    }
}
