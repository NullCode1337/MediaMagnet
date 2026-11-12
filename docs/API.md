# MediaMagnet API

The self-hosted server exposes a versioned REST API under `/api/v1` and serves
the web UI from the same origin. The desktop application does not listen on the
network.

- Specification: [openapi.yaml](./openapi.yaml) (OpenAPI 3.1)
- Typed client: [`@mediamagnet/client`](../packages/client) (TypeScript)

## Running the server

```sh
bun install
bun run build:server
./target/release/mediamagnet-server
```

`build:server` builds the web UI first; the server embeds it at compile time,
so the resulting binary is self-contained. Building `mm-server` with cargo
alone still works, but then the SPA must be supplied at runtime.

Pre-built standalone archives are attached to `server-v*` releases. Locally,
`bun run package:server` writes
`target/package/mediamagnet-server-<version>-<target>.tar.gz`.

| Variable | Default | Purpose |
| --- | --- | --- |
| `MM_BIND` | `0.0.0.0:8080` | Listen address. |
| `MM_DATA_DIR` | `$XDG_DATA_HOME/mediamagnet` | Settings, cookies and runtime data. |
| `MM_DOWNLOAD_ROOT` | `$MM_DATA_DIR/downloads` | Directory that `/files` serves from and downloads are written to. |
| `MM_FILE_TTL_SECS` | `3600` | Seconds an undelivered job output may remain before it is deleted. |
| `MM_STATIC_DIR` | embedded UI | Serve the web UI from this directory instead. |
| `MM_TOKEN` | unset | When set, every request needs this bearer token. |
| `MM_ALLOW_CUSTOM_ARGS` | off | Allows user-supplied tool arguments; see [Policy](#policy). |
| `MM_PYTHON` | unset | Interpreter for the system tool mode (used with the overlay). |
| `MM_BINARIES_DIR` | `binaries/` beside the executable, if present | Directory holding bundled `python/`, `deno`, `ffmpeg` and `ffprobe`. Without it the server uses tools on `PATH` or `MM_PYTHON`. |

Health check:

```sh
curl -s http://localhost:8080/api/v1/health
# {"status":"ok","version":"0.6.3"}
```

## Authentication

With `MM_TOKEN` set, send `Authorization: Bearer <token>` on every request.
A missing or wrong token returns `401` with `WWW-Authenticate: Bearer`.

Three `GET`/`HEAD` routes also accept `?token=<token>`, because `EventSource`
and a plain download link cannot set headers:

- `/events`
- `/files/{path}`
- `/downloads/{id}/file`

Every other endpoint rejects query tokens, so ordinary API calls never carry the
token in a URL or an access log. Those three still do: use a proxy that does not
log query strings, and prefer the header wherever the client can set one.

Without `MM_TOKEN` the API is open. The server warns on startup when it binds a
non-loopback address without a token. Do not expose it to an untrusted network.

Hardening notes and the full audit are in [security.md](./security.md).

## Errors

```json
{ "error": "invalid_url", "message": "url must be http or https" }
```

`error` is stable; `message` is for humans. Domain errors use `400`, unknown
downloads and files `404`, an unsatisfiable range `416`, and filesystem or
storage failures `500`.

## Downloads

```sh
curl -s -X POST http://localhost:8080/api/v1/downloads \
  -H 'Content-Type: application/json' \
  -d '{"url":"https://www.youtube.com/watch?v=...","id":"job-1"}'
# 202 {"id":"job-1"}
```

The job runs in the background. Choose the `id` yourself if you want to track
it; omit it and the server assigns one. Pause and cancel use that id:

```sh
curl -i -X POST   http://localhost:8080/api/v1/downloads/job-1/pause   # 204
curl -i -X DELETE http://localhost:8080/api/v1/downloads/job-1         # 204
curl -i -X DELETE http://localhost:8080/api/v1/downloads               # 204, cancels all
```

The backend (yt-dlp, gallery-dl, spotdl) is detected from the URL. Force one
with `"backend"`, and change individual settings for this job only with a
sparse `"overrides"` patch:

```sh
curl -s -X POST http://localhost:8080/api/v1/downloads \
  -H 'Content-Type: application/json' \
  -d '{"url":"https://...","id":"job-2","backend":"yt-dlp","overrides":{"yt_format":"bv*[height<=720]+ba/b[height<=720]"}}'
# 202 {"id":"job-2"}
```

Overrides apply over `settings` (or the server's settings when `settings` is
absent) and are stored with the job, so resume reuses them. A forced backend
that is not installed returns `503 backend_unavailable`.

### Tracking and resuming

```sh
curl -s http://localhost:8080/api/v1/downloads
# {"jobs":[{"id":"job-1","url":"https://...","state":"running","created_at_ms":1729000000000}]}

curl -s http://localhost:8080/api/v1/downloads/job-1
# {"id":"job-1","url":"https://...","state":"paused","created_at_ms":...}

curl -i -X POST http://localhost:8080/api/v1/downloads/job-1/resume   # 202
```

Records live in memory, oldest first, and nothing survives a restart. At most
100 are kept; finished records are pruned first. `state` is `running`,
`paused`, `cancelled`, `failed` or `finished`.

Resuming replays the URL, backend, overrides, settings and cookies the job was
created with. A
second request with the id of a running or paused job returns `409
duplicate_download`; finished and cancelled ids may be reused.

Requests are bounded: `id` is at most 128 characters of `A-Z`, `a-z`, `0-9`,
`-`, `_`, `.` or `:` (an invalid one returns `400 invalid_id`), a URL is at most
8 KiB with no control characters, a JSON body is at most 1 MiB, and at most
100 jobs may be queued, running or paused at once (`429 too_many_downloads`).

### Output file

When a job finishes, `download-finished` carries the files it wrote and the job
record lists the same paths:

```sh
curl -s http://localhost:8080/api/v1/downloads/job-1
# {"id":"job-1","url":"https://...","state":"finished",
#  "files":["Video.mp4"],"created_at_ms":...}

# Stream the file as an attachment (202 before the job finishes has no file).
curl -OJ http://localhost:8080/api/v1/downloads/job-1/file
```

A job that wrote several files (a playlist or gallery) answers `409
multiple_files` with the full `files` list; pass one back as `?path=`. The byte
range and `Content-Disposition` behaviour is the same as
[`/files`](#files), except the disposition is `attachment`.

Server job outputs are not kept in the download root. Once a file has been
streamed here in full (a request without a `Range` header), it is deleted. A
file that nobody fetches is deleted `MM_FILE_TTL_SECS` after the job finished
(one hour by default). A range request does not trigger deletion, because it may
be one seek in a longer session; the expiry sweep still removes the file. Files
recorded by a previous server process are removed at startup, since their job
records did not survive the restart.

The web UI uses this: the tab that created a job saves its output through the
browser when `download-finished` arrives. API clients should fetch through this
endpoint rather than `/files`, because the file disappears after delivery.

### Per-job settings and cookies

A job may carry its own settings and cookies. They are used for that job only
and never written to the server's configuration.

```sh
curl -s -X POST http://localhost:8080/api/v1/downloads \
  -H 'Content-Type: application/json' \
  -d '{
    "url": "https://example.com/gallery",
    "settings": { "gdl_global_args": "--range 1-20", "user_agent": "MyAgent/1.0" },
    "cookies": [{ "domain": "example.com", "content": "<netscape cookie text>" }]
  }'
```

Omitted settings fields take defaults, not the server's current values: send
the complete settings object. When `cookies` is present it is the only cookie
source for the job (`domain` matching is case-insensitive, on a host boundary,
and the longest matching entry wins); when absent the job runs without cookies.

Custom arguments in per-job settings are subject to [Policy](#policy).

The web UI uses this to keep its configuration in browser `localStorage`
without needing a server-side settings store.

## Events

```sh
curl -N http://localhost:8080/api/v1/events
# event: download-progress
# data: {"type":"download-progress","id":"job-1","value":42.5}
```

The SSE `event:` name equals the JSON `type`. Payloads are `TransportEvent` in
the OpenAPI spec:

| Event | Data |
| --- | --- |
| `download-started` | `{id}` |
| `download-finished` | `{id, files}` — `files` are paths relative to the download root |
| `download-progress` | `{id, value}` — percentage |
| `download-status` | `{id, value}` — status line |
| `download-error` | `{id, value}` — message |
| `download-state` | `{id, value}` — `paused`, `cancelled`, `failed` or `finished` |
| `notification` | `{level, message}` — `info`, `success`, `warning` or `error` |

Keep-alive comments are sent every 20 seconds. A slow consumer skips events it
missed rather than being disconnected; there is no replay.

## Tools

```sh
curl -s http://localhost:8080/api/v1/tools
# {"tools":[{"name":"yt-dlp","available":true,"source":"preinstalled",...}],"deno":null,"ffmpeg":"/usr/bin/ffmpeg"}

curl -s -X POST http://localhost:8080/api/v1/tools/update
# {"updated":["yt-dlp","gallery-dl","spotdl"],"output":"... pip log ..."}
```

`source` is `preinstalled` (on `PATH`), `system` (`MM_PYTHON`), `bundled` or
`overlay`. An unavailable tool has `"available": false` and an `error`.
`/tools/update` reinstalls the latest Python tools into the writable overlay,
returns the installer's console output in `output`, and needs a bundled or
`MM_PYTHON` interpreter; the Docker image preinstalls them and answers
`500 tool_update_failed`.

## Torrents

Torrent support is not implemented. `GET`/`POST /torrents` and `DELETE
/torrents/{id}` return `501 torrents_unavailable` so clients can detect the
absence rather than guess.

## Settings

`GET /settings` returns the server view: the desktop-only fields
(`always_on_top`, `custom_titlebar`, `custom_titlebar_type`,
`native_notifications`, `clear_cookies_on_exit`, `download_path`) are omitted
and cannot be written over HTTP, even if a request includes them.

`PUT /settings` replaces the server settings. Unknown keys and wrong types are
dropped, `accent_hue` is rounded and clamped to 0–360, and site-argument
entries need string `domain` and `args`. Missing fields revert to defaults.
`POST /settings/reset` writes the defaults.

```sh
curl -s http://localhost:8080/api/v1/settings

# Replaces the whole object: fields not listed here reset to their defaults.
curl -s -X PUT http://localhost:8080/api/v1/settings \
  -H 'Content-Type: application/json' \
  -d '{"dark_mode":false,"yt_format":"best"}'
```

### Policy

By default the server refuses user-supplied tool arguments
(`yt_global_args`, `yt_site_args[].args`, `gdl_*`, `spotdl_*`). If a job
carries non-empty arguments for the backend that handles it, the job fails
with a `Configuration Error`; the values are still stored and returned by the
settings endpoints. Even with custom arguments enabled, an output template
containing a path separator is replaced with the default template. This is what
makes it safe to accept settings from a browser client.

Start the server with `MM_ALLOW_CUSTOM_ARGS=1` to accept the arguments. They
are still checked against a per-tool deny-list, matched by option family so
`--exec` also covers `--exec-before-download` and `--output` covers
`--output-na-placeholder`. The list blocks output paths, config loading,
arbitrary executables (`--ffmpeg-location`, `--downloader`, spotdl's
`--ffmpeg`) and command execution (`--exec`, gallery-dl's filter expressions).
An output template that could leave the job directory (a path separator or a
Windows drive prefix) is replaced with the default template. Only enable it when
every client is trusted: the deny-list tracks the tools' CLIs and a new
exec-capable option is a gap until it is added.

The failure is asynchronous: `POST /downloads` still returns `202`, and the
`download-error` event carries the message.

## Cookies

The server keeps no cookie store. Cookies belong to the client, which sends the
ones it wants used with each job:

```sh
curl -s -X POST http://localhost:8080/api/v1/downloads \
  -H 'Content-Type: application/json' \
  -d '{
    "url": "https://example.com/gallery",
    "cookies": [{ "domain": "example.com", "content": "<netscape cookie text>" }]
  }'
```

`content` is Netscape text or JSON, converted to Netscape for the job. When
`cookies` is present it is the only cookie source; when absent the job runs
without cookies. The server writes the matched jar to a private temporary file
for the lifetime of the job and deletes it when the job ends, is paused or is
cancelled. A restart sweeps anything a crash left behind.

The desktop app manages a local store on the machine it runs on; that store is
not part of this API and is never consulted by the server. A `cookies` directory
left in the server data directory by an older version is removed at startup.

`GET`/`POST /cookies` and `DELETE /cookies[/{domain}]` return
`501 cookies_unavailable`. They stay in the API so a client can detect the
absence rather than guess.

## Files

```sh
curl -s 'http://localhost:8080/api/v1/files?path='
# {"path":"","entries":[{"path":"video.mp4","name":"video.mp4","kind":"file","size":123,"url":"/api/v1/files/video.mp4"}]}
```

Each entry's `url` is ready to use. `GET /files/{path}` serves the file with
`Accept-Ranges: bytes` and `Content-Disposition: inline`, honouring one `Range`
header (`206`); an invalid range returns `416` with `Content-Range: bytes */N`.
Add `?download=1` for `Content-Disposition: attachment`. Paths are resolved
inside the download root; traversal and symlink escapes are rejected, and
listings omit symlinks.

Downloaded files are untrusted. Types a browser can execute (HTML, SVG, XML,
JavaScript, CSS, WebAssembly) are always sent as `application/octet-stream` with
an attachment disposition, and every file response carries
`X-Content-Type-Options: nosniff` and `Content-Security-Policy: sandbox`, so a
downloaded page can never run on the API's origin.

`GET /paths/download-root` returns the root path, and `GET /paths/disk-usage`
returns `{"used_percent": 63.2}` for its filesystem.

## TypeScript client

`@mediamagnet/client` is a workspace package shipped with this repository (it is
not published to a registry). With `bun install` done, other workspaces can
import it directly:

```ts
import { createMediaMagnetClient, subscribeEvents } from "@mediamagnet/client";

const api = createMediaMagnetClient({ baseUrl: "http://localhost:8080", token });

const { data, error } = await api.POST("/downloads", {
  body: { url, id, settings: { user_agent: "MyAgent/1.0" } },
});

const stop = subscribeEvents({
  baseUrl: "http://localhost:8080",
  token,
  onEvent: (event) => console.log(event.type, event),
});
```

The client wraps [`openapi-fetch`](https://github.com/openapi-ts/openapi-typescript/tree/main/packages/openapi-fetch),
so every path, request body and response is typed from the spec. `subscribeEvents`
is browser-only (`EventSource`).

The client targets JSON responses. For file bytes, fetch the `url` returned by
`GET /files` directly (it already percent-encodes every segment), adding the
bearer header. Passing `parseAs: "stream"` to the typed call also works when
you want to pipe `response.body`.

Regenerate the types after changing the API:

```sh
bun run generate:api
```

## Versioning

The path is versioned (`/api/v1`). Within `v1`, new optional fields and new
event types may be added; existing fields and event names keep their meaning.
Breaking changes get a new path version.
