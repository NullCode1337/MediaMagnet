use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::{ConnectInfo, Request, State};
use axum::http::{header, HeaderMap, Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

const MAX_TOKEN_LEN: usize = 4096;
const MAX_TRACKED_ADDRESSES: usize = 4096;

/// Constant-time comparison over fixed-size digests, so neither the contents nor
/// the length of the expected token is revealed by the comparison.
#[doc(hidden)]
pub fn same(a: &[u8], b: &[u8]) -> bool {
    let left = Sha256::digest(a);
    let right = Sha256::digest(b);
    bool::from(left[..].ct_eq(&right[..]))
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let mut values = headers.get_all(header::AUTHORIZATION).iter();
    let value = values.next()?.to_str().ok()?;
    // Two Authorization headers are ambiguous: a proxy and the application can
    // disagree about which one is authoritative. Refuse both rather than pick.
    if values.next().is_some() {
        return None;
    }
    let mut parts = value.splitn(2, char::is_whitespace);
    let scheme = parts.next()?;
    let credentials = parts.next()?.trim();
    scheme.eq_ignore_ascii_case("bearer").then_some(credentials)
}

fn query_token(uri: &axum::http::Uri) -> Option<String> {
    let query = uri.query()?;
    url::form_urlencoded::parse(query.as_bytes())
        .find_map(|(key, value)| (key == "token").then(|| value.into_owned()))
}

fn presented_token(request: &Request) -> Option<String> {
    if let Some(value) = bearer_token(request.headers()) {
        return Some(value.to_string());
    }
    // `EventSource` and a plain download link cannot set headers, so those GET
    // routes also accept `?token=`. No other request may carry a token in the
    // URL, so it cannot end up in logs for ordinary API calls.
    if !query_token_allowed(request.method(), request.uri().path()) {
        return None;
    }
    query_token(request.uri())
}

fn valid_segment(value: &str) -> bool {
    !value.is_empty() && !value.contains('/')
}

fn job_tail<'a>(path: &'a str, suffix: &str) -> Option<&'a str> {
    path.strip_prefix("/downloads/")?.strip_suffix(suffix)
}

/// Middleware on the nested router sees the path with `/api/v1` already
/// stripped; the same helper also accepts the full path.
fn api_path(path: &str) -> &str {
    path.strip_prefix("/api/v1").unwrap_or(path)
}

/// `EventSource` retries a failed stream every few seconds, so counting those
/// failures would let one stale token in an open tab exhaust the budget and then
/// block the prompt that fixes it. Stream failures still need a valid token and
/// are still refused while the address is locked out for other reasons.
fn counts_towards_lockout(method: &Method, path: &str) -> bool {
    let stream = method == Method::GET || method == Method::HEAD;
    !(stream && api_path(path) == "/events")
}

/// Exact route matching. Substring checks would let a listing or a look-alike
/// path accept a token in the URL.
#[doc(hidden)]
pub fn query_token_allowed(method: &Method, path: &str) -> bool {
    if method != Method::GET && method != Method::HEAD {
        return false;
    }
    let path = api_path(path);
    if path == "/events" {
        return true;
    }
    if let Some(rest) = path.strip_prefix("/files/") {
        return !rest.is_empty();
    }
    if job_tail(path, "/file").is_some_and(valid_segment)
        || job_tail(path, "/archive").is_some_and(valid_segment)
    {
        return true;
    }
    false
}

/// Brute-force protection tuned from the environment.
#[derive(Debug, Clone, Copy)]
pub struct AuthPolicy {
    pub max_failures: u32,
    pub window: Duration,
    pub lockout: Duration,
    pub trust_proxy: bool,
}

impl Default for AuthPolicy {
    fn default() -> Self {
        Self {
            max_failures: 10,
            window: Duration::from_secs(300),
            lockout: Duration::from_secs(300),
            trust_proxy: false,
        }
    }
}

impl AuthPolicy {
    pub fn from_env() -> Self {
        let mut policy = Self::default();
        if let Some(value) = env_u64("MM_AUTH_MAX_FAILURES") {
            policy.max_failures = value.min(10_000) as u32;
        }
        if let Some(value) = env_u64("MM_AUTH_WINDOW_SECS").filter(|value| *value > 0) {
            policy.window = Duration::from_secs(value);
        }
        if let Some(value) = env_u64("MM_AUTH_LOCKOUT_SECS").filter(|value| *value > 0) {
            policy.lockout = Duration::from_secs(value);
        }
        policy.trust_proxy = env_flag("MM_TRUST_PROXY");
        policy
    }
}

fn env_u64(name: &str) -> Option<u64> {
    std::env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
}

fn env_flag(name: &str) -> bool {
    std::env::var(name).ok().is_some_and(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}

struct Attempts {
    failures: u32,
    window_start: Instant,
    blocked_until: Option<Instant>,
}

#[doc(hidden)]
pub struct Failure {
    pub failures: u32,
    pub blocked_for: Option<Duration>,
}

/// Sliding-window failure counter keyed by client address.
pub struct AuthLimiter {
    policy: AuthPolicy,
    attempts: Mutex<HashMap<IpAddr, Attempts>>,
}

impl AuthLimiter {
    pub fn new(policy: AuthPolicy) -> Self {
        Self {
            policy,
            attempts: Mutex::new(HashMap::new()),
        }
    }

    pub fn policy(&self) -> AuthPolicy {
        self.policy
    }

    /// `Err(retry_after)` means this address is currently locked out.
    pub fn check(&self, key: IpAddr, now: Instant) -> Result<(), Duration> {
        if self.policy.max_failures == 0 {
            return Ok(());
        }
        let Ok(mut attempts) = self.attempts.lock() else {
            return Ok(());
        };
        let Some(entry) = attempts.get_mut(&key) else {
            return Ok(());
        };
        if let Some(until) = entry.blocked_until {
            if now < until {
                return Err(until - now);
            }
            attempts.remove(&key);
            return Ok(());
        }
        if now.saturating_duration_since(entry.window_start) >= self.policy.window {
            attempts.remove(&key);
        }
        Ok(())
    }

    pub fn failure(&self, key: IpAddr, now: Instant) -> Failure {
        if self.policy.max_failures == 0 {
            return Failure {
                failures: 0,
                blocked_for: None,
            };
        }
        let Ok(mut attempts) = self.attempts.lock() else {
            return Failure {
                failures: 0,
                blocked_for: None,
            };
        };
        if attempts.len() >= MAX_TRACKED_ADDRESSES {
            prune(&mut attempts, now, self.policy);
        }
        let key = if attempts.len() >= MAX_TRACKED_ADDRESSES && !attempts.contains_key(&key) {
            IpAddr::V4(Ipv4Addr::UNSPECIFIED)
        } else {
            key
        };
        let entry = attempts.entry(key).or_insert(Attempts {
            failures: 0,
            window_start: now,
            blocked_until: None,
        });
        if let Some(until) = entry.blocked_until {
            if now < until {
                return Failure {
                    failures: entry.failures,
                    blocked_for: Some(until - now),
                };
            }
            entry.failures = 0;
            entry.window_start = now;
            entry.blocked_until = None;
        }
        if now.saturating_duration_since(entry.window_start) >= self.policy.window {
            entry.failures = 0;
            entry.window_start = now;
        }
        entry.failures = entry.failures.saturating_add(1);
        if entry.failures >= self.policy.max_failures {
            entry.failures = 0;
            entry.blocked_until = Some(now + self.policy.lockout);
            return Failure {
                failures: self.policy.max_failures,
                blocked_for: Some(self.policy.lockout),
            };
        }
        Failure {
            failures: entry.failures,
            blocked_for: None,
        }
    }

    pub fn success(&self, key: IpAddr) {
        if let Ok(mut attempts) = self.attempts.lock() {
            attempts.remove(&key);
        }
    }
}

fn prune(attempts: &mut HashMap<IpAddr, Attempts>, now: Instant, policy: AuthPolicy) {
    attempts.retain(|_, entry| match entry.blocked_until {
        Some(until) => now < until,
        None => now.saturating_duration_since(entry.window_start) < policy.window,
    });
}

/// Everything the middleware needs. Cloneable so the router can own a copy.
#[derive(Clone)]
pub struct AuthGuard {
    token: Option<Arc<str>>,
    limiter: Arc<AuthLimiter>,
}

impl AuthGuard {
    pub fn new(token: Option<String>, policy: AuthPolicy) -> Self {
        Self {
            token: token
                .map(|token| token.trim().to_string())
                .filter(|token| !token.is_empty())
                .map(Arc::from),
            limiter: Arc::new(AuthLimiter::new(policy)),
        }
    }

    pub fn limiter(&self) -> &AuthLimiter {
        &self.limiter
    }

    fn accepts(&self, presented: &str) -> bool {
        let Some(expected) = self.token.as_deref() else {
            return false;
        };
        presented.len() <= MAX_TOKEN_LEN && same(presented.as_bytes(), expected.as_bytes())
    }
}

fn client_key(request: &Request, trust_proxy: bool) -> IpAddr {
    if trust_proxy {
        // The nearest proxy appends the peer to the end of the list, so the last
        // entry is the one a client cannot forge.
        if let Some(ip) = request
            .headers()
            .get("x-forwarded-for")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.rsplit(',').next())
            .and_then(|value| value.trim().parse::<IpAddr>().ok())
        {
            return ip;
        }
    }
    request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|info| info.0.ip())
        .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
}

fn unauthorized() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        [(header::WWW_AUTHENTICATE, "Bearer realm=\"MediaMagnet\"")],
        Json(json!({
            "error": "unauthorized",
            "message": "This server requires a bearer token",
        })),
    )
        .into_response()
}

fn locked_out(retry_after: Duration) -> Response {
    let seconds = retry_after.as_secs().max(1);
    (
        StatusCode::TOO_MANY_REQUESTS,
        [
            (
                header::WWW_AUTHENTICATE,
                "Bearer realm=\"MediaMagnet\"".to_string(),
            ),
            (header::RETRY_AFTER, seconds.to_string()),
        ],
        Json(json!({
            "error": "too_many_attempts",
            "message": format!(
                "Too many failed authentication attempts; retry in {seconds} seconds"
            ),
        })),
    )
        .into_response()
}

pub async fn require_token(
    State(guard): State<AuthGuard>,
    request: Request,
    next: Next,
) -> Response {
    if guard.token.is_none() {
        return next.run(request).await;
    }

    let policy = guard.limiter.policy();
    let key = client_key(&request, policy.trust_proxy);
    let now = Instant::now();

    if let Err(retry_after) = guard.limiter.check(key, now) {
        return locked_out(retry_after);
    }

    let presented = presented_token(&request);
    if presented
        .as_deref()
        .is_some_and(|token| guard.accepts(token))
    {
        guard.limiter.success(key);
        return next.run(request).await;
    }

    if !counts_towards_lockout(request.method(), request.uri().path()) {
        return unauthorized();
    }

    let failure = guard.limiter.failure(key, now);
    if let Some(retry_after) = failure.blocked_for {
        eprintln!(
            "[MediaMagnet][Server] repeated authentication failures from {key}; \
             blocked for {}s",
            retry_after.as_secs()
        );
        return locked_out(retry_after);
    }
    unauthorized()
}

/// A token that is too short or is a stock placeholder is guessable even with
/// throttling. The server warns instead of refusing to start.
#[doc(hidden)]
pub fn token_is_weak(token: &str) -> bool {
    const COMMON: &[&str] = &[
        "change-me",
        "changeme",
        "change_me",
        "password",
        "secret",
        "admin",
        "test",
        "token",
        "mediamagnet",
        "letmein",
        "1234567890",
    ];
    let lower = token.trim().to_ascii_lowercase();
    lower.len() < 16 || COMMON.contains(&lower.as_str())
}
