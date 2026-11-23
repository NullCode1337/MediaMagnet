# Security audit

Scope: the self-hostable server (`crates/mm-server`), the shared engine
(`crates/mm-core`), and how the web UI talks to them. The desktop app runs
`mm-core` with `Policy::desktop()`, where custom arguments are unrestricted by
design; that is a local-trust model and is out of scope here.

Method: manual review of every server route, the auth middleware, path and
cookie handling, the argument policy, process spawning, static assets, job
lifecycle, and the browser transport. Findings were reproduced against a live
debug build where practical.

## Fixed

| # | Severity | Finding | Fix |
|---|---|---|---|
| 1 | High | `/api/v1/files/{path}` served downloaded files `inline`. An HTML/SVG/JS file pulled from an untrusted site ran on the app origin, where the bearer token lives in `localStorage`, so opening it in the file browser handed over the token. | Active content (HTML, SVG, XML, JS, CSS, WASM) is served as `application/octet-stream` with `Content-Disposition: attachment`. Every file response also carries `X-Content-Type-Options: nosniff` and `Content-Security-Policy: sandbox`, so even an unknown active type cannot run with origin privileges. |
| 2 | High | Stored cookie profiles were attached to a URL when the profile name appeared *anywhere* in the URL. A caller could make the server send the operator's stored cookies to `https://attacker.example/?x=youtube`. | The server no longer has a cookie store (finding 14). The desktop's local store, which still auto-matches jars, now matches on domain boundaries via `url::Url`: exact host, `.`-suffix, or the site label before the public suffix. |
| 3 | Medium | The custom-argument deny-list matched long options exactly. `--exec-before-download`, `--ffmpeg-location`, `--downloader-args`, gallery-dl's `--filter` (a Python expression), and spotdl's `--ffmpeg` slipped through, so `MM_ALLOW_CUSTOM_ARGS=1` could lead to arbitrary command execution. | Long options are matched by family (`--exec` covers `--exec-before-download`), and the exec-capable flags above were added, along with `--cookies`, `--postprocessor-args` for yt-dlp and the gallery-dl filter/write-log options. |
| 4 | Medium | Before the store was removed, `POST /api/v1/cookies` with `{"input":{"type":"FilePath",...}}` made the server read an arbitrary path on the host and copy it into the cookie store. | The store is gone (finding 14), so the endpoint returns `501`; `FilePath` remains available only to the desktop app, where the path is local to the user. |
| 5 | Medium | `GET`/`HEAD` accepted `?token=` on every endpoint, not just the ones that cannot set headers, putting the token into access logs and browser history for ordinary API calls. | Query tokens are accepted only on `/events`, `/files/{path}` and `/downloads/{id}/file`, and only for `GET`/`HEAD`. All other requests require the header. |
| 6 | Medium | `Range: bytes=-0` produced `start > end`; `ByteRange::len()` underflowed (panic in debug, wrapped length in release). | Suffix length zero is rejected as unsatisfiable; the length calculation saturates. |
| 7 | Low | Queueing was unbounded: each accepted job holds a settings blob, cookies and a waiting task, so a token holder could exhaust memory before the 5 download permits ever ran. | At most 100 queued/running/paused jobs; further submissions get `429 too_many_downloads`. Job ids are capped at 128 characters from `[A-Za-z0-9._:-]`, URLs at 8 KiB with no control characters, and API bodies at 1 MiB. |
| 8 | Low | Directory listings followed symlinks, reporting the name, size and type of targets outside the download root. | Symlinks are skipped in `list_dir`. Serving them was already rejected by `safe_join`. |
| 9 | Low | `MM_STATIC_DIR` joined raw request paths without resolving them. | Reads are canonicalised and must stay inside the configured directory. |
| 10 | Low | Per-job cookie files (live session credentials) were written with the process umask, typically `0644`; the data and config directories were world-traversable. | Per-job cookie files are written `0600` (including `set_permissions` for files that already existed). The config and data directories are set to `0700` on Unix. |
| 11 | Low | gallery-dl direct links are renamed into a directory named after the remote filename. A name like `.._x` produced `..` as that component; only the later `create_dir` failure prevented the rename. | The derived component is validated explicitly; `.`/`..`, separators and NUL are skipped. |
| 12 | Low | The server bound `0.0.0.0:8080` with no token and only an informational line. | A prominent warning is printed when the bind address is not loopback and `MM_TOKEN` is unset. Exposing without a token remains a configuration choice, not a silent default. |
| 13 | Low | The server-side output-template guard rejected `/` and `\\` but not a Windows drive-relative prefix (`C:name`), which can resolve outside the job directory on Windows. | The guard also rejects a drive-letter prefix and falls back to the default template. |
| 14 | High | The server persisted cookie profiles in its data directory and exposed them over `GET`/`POST`/`DELETE /cookies`. That is a shared credential store on a host used by several clients, and a caller could attach the operator's jar to a URL of their choosing. | The store is gone: the server creates no cookie directory, removes one left by an older version at startup, the endpoints return `501 cookies_unavailable`, and `apply_cookie` only uses cookies that arrived with the job (`Policy::cookie_store` is false on the server). Per-request jars are temporary `0600` files removed when the job ends, is paused or cancelled, with a startup sweep for crashes. |
| 15 | High | The query-token allowlist used substring matching (`path.contains("/files")`), so `GET /api/v1/files` (the directory listing) and look-alike paths accepted `?token=`, widening URL and access-log exposure beyond the documented three routes. Separately, `/api/v1/downloads/{id}/archive` was missing from the allowlist, so the web UI's multi-file archive links returned `401` whenever `MM_TOKEN` was set. | Query tokens are matched structurally against the exact routes; `/events`, `/files/{path}`, `/downloads/{id}/file` and `/downloads/{id}/archive`; with or without the `/api/v1` prefix that `nest` strips before the middleware runs. Tests cover every allowed and denied shape, including the nested-router path. |
| 16 | Medium | There was no brute-force protection: a client could guess tokens against `/health` indefinitely, and the UI misreported a throttled response as an invalid token. | Failed attempts are counted per client address in a sliding window (`MM_AUTH_MAX_FAILURES`, default 10 per 300s). Reaching the limit locks the address out (`MM_AUTH_LOCKOUT_SECS`, default 300s) with `429` and `Retry-After`; even the correct token is refused until it expires. A success clears the count, addresses are independent, and the tracked-address map is bounded. `GET`/`HEAD /events` is exempt from the count so `EventSource`'s automatic retries cannot lock the address out. `MM_TRUST_PROXY=1` keys on the last `X-Forwarded-For` hop. |
| 17 | Medium | `same()` returned immediately when lengths differed, leaking the expected token's length and comparing variable-length input byte by byte. | Both values are hashed with SHA-256 and compared with `subtle::ConstantTimeEq`, so the comparison covers a fixed 32-byte digest in constant time. Presented tokens over 4 KiB are rejected before hashing. |
| 18 | Low | The `Bearer` scheme was matched case-sensitively, and query tokens were parsed by splitting on `&`, so percent-encoded bytes and tokens containing `+`, `%`, `&` or non-ASCII text could not authenticate. | The scheme check is ASCII case-insensitive; query parameters are percent-decoded with `form_urlencoded`. A duplicate `token` parameter resolves to the first value and cannot rescue a wrong one. |
| 19 | Low | `MM_TOKEN` accepted short or stock values such as `change-me` silently, and unauthenticated unknown `/api/*` paths returned the SPA with `200`. | Startup warns when the token is under 16 characters or a known placeholder. Unknown `/api/*` paths now return a JSON `404` instead of HTML. |

## Response headers

Every response now carries:

- `X-Content-Type-Options: nosniff`
- `Referrer-Policy: no-referrer` (tokens travel in query strings on three routes)
- `X-Frame-Options: DENY`

HTML responses (the SPA) get a CSP of `default-src 'self'`, `script-src 'self'`,
`style-src 'self' 'unsafe-inline'`, `connect-src 'self'`, `object-src 'none'`,
`frame-ancestors 'none'`. API responses are `Cache-Control: no-store`. File
responses get `Content-Security-Policy: sandbox`, which gives any rendered
document an opaque origin.

## Accepted risks

- **SSRF is inherent.** The server downloads URLs supplied by authenticated
  clients; that is the product. A token holder can make the host fetch internal
  addresses and cloud metadata endpoints, and yt-dlp/gallery-dl can reach
  whatever the host can. Run it as an unprivileged user with no cloud instance
  role, and do not expose it to untrusted callers.
- **One shared download root.** Every token holder can list, download and delete
  using the same root, and job records are global. There is no per-user
  isolation; deploy one instance per trust boundary.
- **The token is a bearer capability stored in the browser.** CSP and the file
  hardening remove the easy path to script execution on the origin, but any
  future XSS is still a token compromise. Treat tokens as passwords: one per
  deployment, rotated when someone leaves, and never in a URL you paste around.
- **Custom arguments run with the server's privileges.** They are unrestricted
  by default, so a client that can reach the API can pass options that run
  commands (`--exec`, gallery-dl filters). `MM_ALLOW_CUSTOM_ARGS=filtered`
  enables the deny-list, which tracks the tools' CLIs (a new exec-capable option
  is a gap until it is added); `off` refuses arguments entirely. Bind to
  loopback or require `MM_TOKEN` when others can reach the port.
- **Brute-force throttling is per address and in memory.** Ten failures in five
  minutes lock an address out for five minutes (`MM_AUTH_MAX_FAILURES`,
  `MM_AUTH_WINDOW_SECS`, `MM_AUTH_LOCKOUT_SECS`). A distributed attack from many
  addresses is slowed, not stopped, and counters reset on restart. Behind a
  reverse proxy set `MM_TRUST_PROXY=1` so clients are keyed by the last
  `X-Forwarded-For` hop; never enable it on a directly exposed server, where the
  header can be spoofed. Expensive unauthenticated endpoints are still not rate
  limited.
- **The events stream does not count toward the lockout.** `EventSource` retries
  a failed stream every few seconds, so one stale token in an open tab would
  otherwise lock the address out and then block the prompt that fixes it. `GET`/
  `HEAD /events` still needs a valid token and is still refused while the
  address is locked out, but its failures are not counted. The consequence is
  that a client probing only `/events` is not throttled, so the lockout is
  defence in depth, not a substitute for a strong token.
- **Token in query strings.** The SSE stream and plain download links cannot set
  headers, so four GET routes accept `?token=` (events, files, a job's file and
  a job's archive). Configure the proxy not to log query strings, and prefer the
  header wherever the client can set one.

## Verification

- `cargo test -p mm-core -p mm-server` covers host matching, argument families,
  range parsing, active-content classification, query-token paths, the exact
  query-token allowlist (nested and full paths), header parsing, length-hiding
  constant-time comparison, per-address lockout, the events-stream exemption,
  and job limits.
- Live checks against a server with `MM_TOKEN` set: missing, wrong, `Basic`,
  incomplete and over-long `Bearer` credentials return `401`; the scheme is
  case-insensitive; `?token=` works on `/events`, `/files/{path}`, a job's `file`
  and its `archive`, but returns `401` on `/health`, `/files` (the listing),
  `/settings` and every mutating method; a query token on a missing job returns
  `404`, proving the request passed auth; the tenth wrong token returns `429`
  with `Retry-After`, and a correct token is refused until the lockout expires;
  repeated stale tokens on `/events` never lock the address out.
- File checks: an HTML file in the download root is returned as
  `application/octet-stream` + `attachment` + `sandbox`; `bytes=-0` returns
  `416`; an id containing a newline is rejected with `400 invalid_id`; the SPA
  is served with its own CSP and an SPA route named `/files` is not sandboxed;
  `GET /api/v1/cookies` returns `501 cookies_unavailable` and the server
  creates no `cookies` directory; unknown `/api/*` paths return JSON `404`.
- Dependency advisories were not scanned in this environment (`cargo audit` is
  not installed). The new direct dependencies (`sha2`, `subtle`, `url`) were
  already in the lockfile transitively.
