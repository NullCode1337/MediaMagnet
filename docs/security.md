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
- **Custom arguments are best-effort.** With `MM_ALLOW_CUSTOM_ARGS=1` the
  deny-list tracks the tools' CLIs; a new exec-capable option that is not in the
  list is a gap until it is added. Leave the setting off unless every client is
  trusted, and keep the tools updated.
- **No brute-force throttling or request rate limiting.** The token comparison
  is constant time, but authentication attempts and expensive endpoints are not
  rate limited. Put Cloudflare, an authenticated reverse proxy, or Zero Trust
  Access in front of a public deployment.
- **Token in query strings.** The SSE stream and plain download links cannot set
  headers, so those three routes accept `?token=`. Configure the proxy not to
  log query strings.

## Verification

- `cargo test -p mm-core -p mm-server` covers host matching, argument families,
  range parsing, active-content classification, query-token paths and job limits.
- Live checks: an HTML file in the download root is returned as
  `application/octet-stream` + `attachment` + `sandbox`; `bytes=-0` returns
  `416`; `?token=` is rejected on `/health` and accepted on `/events`; an id
  containing a newline is rejected with `400 invalid_id`; the SPA is served
  with its own CSP and an SPA route named `/files` is not sandboxed;
  `GET /api/v1/cookies` returns `501 cookies_unavailable` and the server
  creates no `cookies` directory.
- Dependency advisories were not scanned in this environment (`cargo audit` is
  not installed). The lockfile only gained the `url` dependency, which was
  already present transitively.
