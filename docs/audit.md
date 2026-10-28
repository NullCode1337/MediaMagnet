# MediaMagnet: audit of the current implementation

Source: `/home/nullcode1337/Documents/MediaMagnet-old` (read-only), version `0.6.1`.
Everything below is derived from reading every file under `src-tauri/src`, `src/`, the Tauri
config/capabilities, `PKGBUILD`, `package.json` and `.github/workflows/release.yml`.

The OLD tree is ~13 MB and 211 files. Excluding Android scaffolding (`src-tauri/gen/android`,
which is generated and unused by the desktop app) and icons, the code that matters is:

| Area | Files | Size |
|---|---|---|
| Rust | `src-tauri/src/{main.rs,lib.rs}`, `commands/{mod,download,settings,cookie,utils,init}.rs` | 1 872 lines |
| Svelte 5 UI | `src/App.svelte`, `src/lib/components/*`, `src/lib/components/ui/*` (shadcn-svelte) | ~3 300 lines |
| Stores | `src/lib/stores/{store,settings}.svelte.ts`, `settings-schema.ts`, `src/lib/color.ts` | 749 lines |
| Build/CI | `package.json`, `vite.config.ts`, `tauri.conf.json`, `release.yml`, `PKGBUILD` | — |

---

## 1. Tauri commands

All commands are registered in `src-tauri/src/lib.rs:42-58`. There is no state
(`tauri::State`); all shared state is process-global (`OnceLock<Mutex<…>>`) inside
`commands/download.rs`.

| Command | Args | Returns | Emits | Called by |
|---|---|---|---|---|
| `downloader` | `url: String`, `downloadId: String` | `()` (fire-and-forget) | `download-status`, `download-started`, `download-progress`, `download-error`, `download-finished` | `src/App.svelte:137` (`startDownload`, both new and resume) |
| `pause_download` | `downloadId: String` | `Result<(), String>` | `download-status` (`"Paused"`) | `src/App.svelte:171` |
| `cancel_download` | `downloadId: String`, `url: String` | `Result<(), String>` | `download-error` (`"Download cancelled and files permanently deleted"`) | `src/App.svelte:186` |
| `cancel_all_downloads` | — | `Result<(), String>` | one `download-error` per active id (`"Download cancelled"`) | `src/App.svelte:196` |
| `settings` | `action: "check" \| "reset"` | `Result<Settings, String>` | — | `src/lib/stores/settings.svelte.ts:56` (`check`), `:165` (`reset`) |
| `update_settings` | `settings: Value` (entire config object) | `Result<Settings, String>` (canonicalised) | — | `src/lib/stores/settings.svelte.ts:102` (every debounced save) |
| `save_cookie` | `domain: String`, `input: { type: "FilePath"\|"Content", value: String }` | `Result<String, String>` (destination path) | — | `src/lib/components/Settings/Cookies.svelte:73` |
| `get_cookies` | — | `Result<HashMap<String, String>, String>` (stem → path) | — | `src/lib/components/Settings/Cookies.svelte:18` |
| `clear_cookies` | — | `()` | `notification` (plain `String`) | `src/lib/components/Settings/Cookies.svelte:106` |
| `delete_cookie` | `path: String` | `()` | `notification` (plain `String`) | `src/lib/components/Settings/Cookies.svelte:93` |
| `check_download_path` | `path: String` | `Result<(), String>` | — | `settings.svelte.ts:203` (import), `:329` (`setDownloadPath`) |
| `get_free_space` | — | `Result<f64, String>` (used %) | — | `src/App.svelte:224`, polled every 60 s |
| `notify` | `body: String` | `Result<(), String>` | `notification` (plain `String`) or OS notification | `src/App.svelte:96,103,156,173,188,201` |
| `overwrite_json` | `links: Vec<String>` | `()` | — | **no caller** |
| `check_links` | — | `()` | `link-event { links, message }` | **no caller** |

Rust-side helpers that are not commands: `set_download_path(app)` (called from
`download.rs:366,681`), `init_config(app)` (called once from `lib.rs:32`),
`Settings::load/save/apply`, `is_netscape`, `convert_json`.

## 2. Tauri events

| Event | Payload | Emitted by | Listened by |
|---|---|---|---|
| `download-progress` | `{ id: String, value: f64 }` | `download.rs` (all three backends) | `App.svelte:273` |
| `download-status` | `{ id: String, value: String }` | `download.rs`, `pause_download` | `App.svelte:277` |
| `download-error` | `{ id: String, value: String }` | `download.rs`, `cancel_*` | `App.svelte:284` |
| `download-started` | `{ id: String }` | `download.rs:788` | `App.svelte:299` |
| `download-finished` | `{ id: String }` | `download.rs:767,847` | `App.svelte:307` |
| `notification` | **inconsistent**: `{ id, value }` from `download.rs:597`; bare `String` from `notify`, `clear_cookies`, `delete_cookie` | see left | `App.svelte:248` (typed as `{id,value}`) |
| `disk-update` | `number` | **never emitted** | `App.svelte:271` |
| `link-event` | `{ links: String[], message: String }` | `check_links` (**never called**) | no listener |

The frontend keeps all job state locally (`tasks` in `App.svelte:52`); Rust keeps only the
child-process table. Nothing survives an app restart except `links.json`, which is dead.

## 3. Feature inventory

| Feature | Where the logic lives | Tauri-coupled? | Target |
|---|---|---|---|
| Unified downloader (backend auto-selection) | `download.rs:41-70` (`Backend::identify`): host contains `spotify.com` → spotdl; else run `yt-dlp --print title --no-download <url>` and accept it when exit=0 and stderr lacks `"Falling back on generic"`; else gallery-dl | Yes: needs `AppHandle` only to resolve the binary (`base_command`) | all three |
| Download execution + progress parsing | `download.rs:354-678`; per-backend line parsers for spotdl (`Found N songs`, `k/n complete`), yt-dlp (`--progress-template "%(progress)j"` JSON lines), gallery-dl (line count vs. pre-count via `-g`) | Yes: `app.emit`, `app.path()`, shell sidecar | all three |
| Concurrency limit | `download.rs:88-91`, semaphore of **5** | No (process-global) | all three |
| Pause / resume | `pause_download` kills the child; resume is a fresh `downloader` call with the same id/URL (tools resume partial files) | Yes: `app.emit` | all three |
| Cancel (deletes temp files) | `cancel_download` + `clean_tempfiles` (`download.rs:680-750`): yt-dlp re-runs `--print filename --no-download`, deletes `.part`/`.ytdl`; gallery-dl asks extractor for `{_directory}` and removes first two path components | Yes | all three |
| Global custom args (per backend) | `settings.rs` fields `yt_global_args`, `gdl_global_args`, `spotdl_global_args`; applied by `apply_args` (`download.rs:232-265`) via `shell_words::split` | No | all three |
| Per-site custom args (yt-dlp, gallery-dl) | `yt_site_args`, `gdl_site_args: Vec<SiteArguments{id,domain,args}>`; case-insensitive substring match of `domain` against URL, args appended after global args | No | all three |
| Cookie manager (JSON + Netscape) | `cookie.rs`, `utils.rs:is_netscape/convert_json`; cookies stored as `<app_data>/cookies/<domain>.txt`; auto-applied by lowercase substring match of file stem against URL (`download.rs:99-135`); one retry without cookies on failure (`download.rs:808-844`) | Yes: `app.path()`, `app.emit` | all three |
| Settings persistence / sanitisation / migration | `settings.rs`: JSON at `<app_config>/settings.json`, atomic write (tmp + fsync + rename), corrupt-file backup, strict per-key sanitiser, `SETTINGS_VERSION = 1`, no-op `migrate` | Partly: path resolution and `Settings::apply` (window flags) | all three |
| Settings import/export | Frontend only: `settings.svelte.ts:172-319` (clipboard/file via dialog+fs plugins), JSON shown read-only in `ImportExport.svelte` | Yes: dialog/fs/clipboard plugins | desktop (web/API get equivalent endpoints) |
| Window behaviour from settings | `settings.rs:333-341`: `always_on_top`, `decorations = !custom_titlebar` | Yes: `AppHandle`/`WebviewWindow` | desktop only |
| Headless mode | Frontend only: `uiState.headless` (`store.svelte.ts:13-19`, window < 250 px either side), `Titlebar.svelte:29-39` resizes to 200×200 + always-on-top, circular progress UI (`App.svelte:357-399`) | Yes: `@tauri-apps/api/window` | desktop only |
| Custom titlebar / macOS traffic lights | `Titlebar.svelte`, `custom_titlebar_type` setting | Yes: Tauri window API | desktop only |
| Native notifications | `utils.rs:244-274`: `notify-send` on Linux, `tauri-plugin-notification` elsewhere; falls back to `notification` event | Yes | desktop (server: log/event only) |
| Disk usage | `utils.rs:218-242` (`sysinfo::Disks`) | Yes (path from settings) | desktop UI; API may expose |
| "Clear cookies on exit" | Setting exists (`clear_cookies_on_exit`, Privacy tab) but **no code reads it** | — | broken in OLD |
| Startup link queue (`links.json`) | `init.rs`: created/validated at startup; `overwrite_json` writes; `check_links` emits `link-event` | Yes | dead in OLD (no caller) |
| Clipboard paste-to-download | `App.svelte:149-162` (`readText`) | Yes: plugin | desktop; web uses navigator.clipboard (needs gesture) |
| Theme (system/light/dark), AMOLED, accent hue/black/white | `mode-watcher` + `app.css` variables + `settings.svelte.ts:applyAccent` | No (localStorage for mode) | all three |
| Mobile/Android layout | `App.svelte` responsive branches, `android.json` capability, `tauri-plugin-m3` | Yes | out of scope (keep code paths harmless) |

## 4. Sidecar discovery and invocation, and why arm64 fails

`tauri.conf.json` declares `bundle.externalBin: ["binaries/yt-dlp", "binaries/gallery-dl"]`.
The Tauri bundler expects, at build time, one file per target triple named
`binaries/<name>-<triple>[.exe]`. At runtime `app.shell().sidecar(name)` resolves the file next
to the executable in the resource directory.

`base_command` (`download.rs:137-220`), per tool name:

1. process-global path cache (`OnceLock<Mutex<HashMap<String, PathBuf>>>`);
2. `which::which_all(name)`, **discarding any path whose lowercase form contains
   `"mediamagnet"`**, then validating each candidate with `--version` (first success wins,
   cached);
3. otherwise `app.shell().sidecar(name)`; otherwise error
   `"<name> is not installed and no sidecar available: …"`.

What is actually shipped in the repo:

| File | Present in OLD | Notes |
|---|---|---|
| `yt-dlp-x86_64-unknown-linux-gnu` | yes | x86_64 only |
| `gallery-dl-x86_64-unknown-linux-gnu` | yes | x86_64 only |
| `yt-dlp-x86_64-pc-windows-msvc.exe` | yes | x86_64 only |
| `gallery-dl-x86_64-pc-windows-msvc.exe` | yes | x86_64 only |
| `yt-dlp-aarch64-linux-android`, `gallery-dl-aarch64-linux-android` | yes | Android, not usable on Linux/macOS |
| spotdl | **never** | must already be on `PATH` |
| macOS, any arch | **never** | so `bundle.externalBin` has no file for `aarch64-apple-darwin`/`x86_64-apple-darwin` |

Failure modes:

* **Linux arm64**: no sidecar for `aarch64-unknown-linux-gnu`; relies entirely on `PATH`.
  The AppImage/deb does not put distro packages on PATH, so nothing resolves.
* **Windows arm64**: no `aarch64-pc-windows-msvc.exe`; the x86_64 binaries would run under
  emulation but Tauri looks for the exact triple and does not find it.
* **macOS arm64**: no sidecar at all; a GUI-launched `.app` has a minimal `PATH`
  (`/usr/bin:/bin:/usr/sbin:/sbin`), so Homebrew's `/opt/homebrew/bin` is invisible. `fix_path_env`
  (`main.rs:5`) tries to recover the login-shell PATH but only if the shell starts successfully.
* `gallery-dl.bin` from Codeberg is a PyInstaller blob built for one OS/arch; on any other
  triple it is simply wrong, and the CI workflow downloads that same file for both macOS and
  Linux (`release.yml:60-75,77-92`).
* `spotdl` has no sidecar path at all, so Spotify downloads only ever work if spotdl is on PATH.
* The Android `aarch64-linux-android` files are dead weight in desktop bundles.
* The name filter `"mediamagnet"` is a heuristic to avoid picking the sidecar through `which`
  on Windows; it also hides legitimately renamed copies.

## 5. Rust dependencies on Tauri, window, or plugins

Everything below must be abstracted before moving into `mm-core` (no `tauri` dependency):

| Location | Dependency | Needed value | Abstraction |
|---|---|---|---|
| all commands | `tauri::AppHandle` | context object | `AppPaths` + `EventSink` injected into core |
| `settings.rs:333` | `app.get_webview_window("main")`, `set_always_on_top`, `set_decorations` | window flags | remove from core; desktop adapter applies after `update_settings` |
| `settings.rs:343,384`, `cookie.rs:21,87,110`, `utils.rs:134,185,219`, `init.rs:15-49` | `app.path().app_data_dir()` / `app_config_dir()` / `download_dir()` | filesystem roots | `AppPaths { data, config, downloads }` (server uses env-configurable dirs) |
| `download.rs:199` | `app.shell().sidecar()` | executable resolution | `ToolRuntime::resolve(Tool)` returning a `Command` spec |
| `download.rs` everywhere | `app.emit(...)` | event delivery | `EventSink` (Tauri emitter / tokio broadcast for SSE / callback in tests) |
| `utils.rs:245-269` | `tauri_plugin_notification`, `notify-send` | OS toast | `Notifier` trait; desktop impl keeps plugin, core emits an event |
| `utils.rs:203-209`, `init.rs` | `app.emit("notification", …)` as control flow | notifications | event sink |
| `main.rs:5` | `fix_path_env` | PATH repair | desktop `main.rs` only |
| `lib.rs:6-19` | plugins: `fs`, `single-instance`, `m3`, `os`, `notification`, `clipboard-manager`, `dialog`, `opener`, `process`, `shell` | desktop runtime | stay in `apps/desktop`; frontend plugins are reached through `Transport` |
| `utils.rs:185-216` | `std::env::set_current_dir` | global CWD for downloads | replace with per-process `Command::current_dir` (documented deviation) |
| `download.rs:94-97` | `creation_flags(0x08000000)` (CREATE_NO_WINDOW) | hide console | keep in core behind `#[cfg(windows)]`; harmless outside Tauri |
| `lib.rs:42-58` | `#[tauri::command]` wrappers | IPC | thin adapters in `apps/desktop`, core exposes async methods |

Non-Tauri dependencies that move as-is: `tokio` (process/time/sync), `serde`/`serde_json`,
`regex`, `shell-words`, `which`, `sysinfo` (desktop-only consumer), `dirs`, `thiserror`.

## 6. Behaviours that are surprising, broken, or ambiguous

These are preserved where harmless; each intentional fix is recorded in `docs/decisions.md`.

1. **`notification` payload shape is inconsistent.** `notify`, `clear_cookies`,
   `delete_cookie` emit a bare string; the UI reads `event.payload.value` → blank toasts. In
   the new event model every notification carries `{ value }` (and an optional `level`).
2. **Cookie domain sanitisation is broken.** Cookies.svelte saves
   `domain.toLowerCase().replace(/[./]/g, "")`, so `youtube.com` becomes `youtubecom`, while
   matching requires the URL to contain `youtubecom`. Decision: keep the on-disk filename
   convention but normalise both the URL and the stem the same way when matching.
3. **`clear_cookies_on_exit` does nothing.** No implementation exists. Decision: implement it
   in the desktop runtime on exit; the API equivalent is an explicit cookie wipe endpoint.
4. **`dark_mode` is persisted but never applied on start.** `mode-watcher` uses
   `localStorage`; `settings.dark_mode` is written from `mode.current` but never read back.
   Decision: keep `mode-watcher` as source of truth for desktop/web; the API stores the flag.
5. **`disk-update` is listened to but never emitted**, and `get_free_space` is polled.
   Decision: drop the listener; keep polling (desktop) and expose disk info in the API status.
6. **`links.json` / `overwrite_json` / `check_links` / `link-event` are dead.** Nothing writes
   links from the UI and nothing listens for `link-event`. Decision: keep the file for
   compatibility with an existing data directory, but the feature is not exposed in the UI.
   Server jobs are in-memory only (same durability as OLD).
7. **Global `set_current_dir`** makes concurrent downloads with different `download_path`
   values race, and leaves the process CWD elsewhere. Decision: `current_dir` per child only.
8. **Weird skip rule** `download.rs:765`: skip URLs containing `didg` and `ls.i` unless they
   also contain `original`. No comment or issue explains it. Decision: preserve verbatim,
   isolated in one named predicate so it can be removed once the origin is known.
9. **`get_free_space`** ignores the `MediaMagnet` subdirectory and treats the literal
   `"default"` as a path; `set_download_path` appends `MediaMagnet` unless the configured path
   already contains that word, case-insensitively. Decision: factor the path rule into one
   function in core and use it everywhere (disk usage included).
10. **Site-arg IDs** are regenerated with `crypto.randomUUID()` in the frontend on every load;
    Rust preserves an existing string `id`. Decision: core assigns stable IDs when missing and
    the frontend no longer rewrites them.
11. **`Backend::identify` costs a full yt-dlp extraction** per URL (still does) and ignored
    cookies and custom args — so a YouTube link that only works with a cookie file (bot
    check) or a custom arg was probed without them, reported as "not yt-dlp", and handed to
    gallery-dl, which has no YouTube extractor ("Unsupported URL"). Decision: probe with the
    real download arguments plus cookies, and report the probe's failure reason instead of
    falling back silently.
12. **Pause is really "kill"**, resume is a new process. Preserved.
13. **gallery-dl `-g` pre-count doubles requests** and can fail on extractors that do not
    support listing; on failure the process output is ignored and progress stays at 0.
    Preserved, error surfaced as a log line.
14. **spotdl never receives cookie or site args** (it gets `--cookie-file` for matched
    profiles only). Preserved.
15. **Concurrency is a hard-coded 5.** Exposed as a core constant for now; server config may
    override later (not in scope).
16. **Android scaffolding** under `src-tauri/gen/android` and the `android.json` capability are
    not part of the three targets. Not carried over.
17. **Settings sanitiser defaults missing keys silently**; import UI warns via its own
    normaliser. Both implementations agree today; the new TS client will reuse the Rust
    output instead of duplicating validation.
18. **`sysinfo` "used %"** is computed as `(total - available) / total`, not the filesystem's
    own `used`. Preserved.

## 7. Toolchain verification (arm64 / x86_64)

Verified on 2026-10-19 against PyPI and GitHub releases.

### Python

* `python-build-standalone` (`astral-sh/python-build-standalone`, release `20261013`) ships
  `install_only` builds for every needed triple: `aarch64-apple-darwin`, `x86_64-apple-darwin`,
  `aarch64-pc-windows-msvc`, `x86_64-pc-windows-msvc`, `aarch64-unknown-linux-gnu`,
  `x86_64-unknown-linux-gnu` (and musl variants). Sizes 25–76 MB compressed. These bundles
  include `pip`.
* Console entry points exist for all three tools, so `python -m <module>` is valid:
  `yt_dlp`, `gallery_dl`, `spotdl` (all three wheels contain `__main__.py`).

### Wheel availability (CPython 3.12, `pip download --only-binary=:all:` with the full
platform-tag set; 64 wheels resolve for every target):

The versions below are the ones checked on the date above. Builds install the tools
unpinned (decision 69), so the dependency closure can move ahead of this table.

| Target | spotdl 4.5.2 | gallery-dl 1.32.17 | yt-dlp 2026.8.19 + EJS |
|---|---|---|---|
| linux x86_64 | ✅ | ✅ | ✅ |
| linux aarch64 | ✅ | ✅ | ✅ |
| macOS arm64 | ✅ | ✅ | ✅ |
| macOS x86_64 | ✅ | ✅ | ✅ |
| Windows x64 | ✅ | ✅ | ✅ |
| Windows arm64 | ✅ | ✅ | ✅ |

Binary (non-pure) wheels in the closure and their lowest platform requirement:

| Package | linux x64 | linux arm64 | mac arm64 | mac x64 | win x64 | win arm64 |
|---|---|---|---|---|---|---|
| `rapidfuzz 3.14.6` | manylinux_2_28 | manylinux_2_28 | 11.0 arm64 | 10.13 | amd64 | arm64 |
| `pydantic-core 2.50.1` | manylinux_2_17 | manylinux_2_17 | 11.0 arm64 | 10.12 | amd64 | arm64 |
| `curl_cffi 0.15.0` (abi3) | manylinux2014 | manylinux2014 | 11.0 arm64 | 10.9 | amd64 | arm64 |
| `pillow 13.0.0` (via spotapi) | manylinux_2_28 | manylinux_2_28 | 11.0 arm64 | 10.13 | amd64 | arm64 |
| `pycryptodomex` (abi3) | manylinux2014 | manylinux2014 | universal2 | universal2 | amd64 | arm64 |
| `markupsafe`, `pymongo`, `websockets`, `brotli`, `charset-normalizer`, `pyyaml` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

**Conclusion: nothing needs to be compiled.** Two consequences to record:

* `rapidfuzz` and `pillow 13` only publish `manylinux_2_28` wheels: the bundled Python needs
  glibc ≥ 2.28 on Linux (Debian 10+, Ubuntu 20.04+, Fedora 29+). Fine for the Docker image
  (Debian bookworm) and for current distros; old AppImage hosts would need an overlay build.
* All binary wheels are `cp312`/`abi3`, which is why the bundled interpreter is pinned to
  CPython 3.12 (not 3.13/3.14) to keep the widest wheel coverage.

### JavaScript runtime for YouTube

`yt-dlp 2026.8.19` declares `yt-dlp-ejs==0.8.0` in its `default` extra and needs an external
JS runtime; Deno is the only one enabled by default (`--js-runtimes`). Deno `v2.9.7` publishes
official `deno` zips for `aarch64-apple-darwin`, `x86_64-apple-darwin`,
`aarch64-pc-windows-msvc`, `x86_64-pc-windows-msvc`, `aarch64-unknown-linux-gnu`,
`x86_64-unknown-linux-gnu` (≈38–43 MB each). Plan: ship one `deno` binary per target as a
Tauri resource, pass `--js-runtimes deno:<path>`; Docker installs Deno from the official image.
(`denort`, ~29 MB, was not chosen: yt-dlp's runtime probing expects the `deno` CLI.)

### ffmpeg

* Windows x64 + arm64: `BtbN/FFmpeg-Builds` `ffmpeg-n9.0-latest-win{64,arm64}-lgpl-9.0.zip`
  (≈170 MB zip; only `ffmpeg.exe`/`ffprobe.exe` are copied out of it).
* macOS arm64 + x64: `eugeneware/ffmpeg-static` release `b6.1.1` has `ffmpeg-darwin-arm64`
  (46 MB) and `ffmpeg-darwin-x64` (79 MB) plus matching `ffprobe` binaries. (evermeet.cx now
  returns HTTP 410 and was x86-only anyway.)
* Linux: the desktop bundle does not ship `ffmpeg` (it uses the system one); the server's
  `-tools` archive bundles the static Linux build (decisions 66–67). Without `ffmpeg`,
  job creation is allowed but the first run returns a clear "ffmpeg not found" error
  (tools that do not need merging still work).

## 8. Extraction plan for `mm-core`

Proposed modules, all `#[tauri]`-free:

```
mm-core/
  model/        Job, JobState, JobEvent, Backend, DownloadRequest, Settings, SiteArgs,
                CookieProfile, AppPaths, ToolStatus, Error
  settings/     Settings load/save/sanitize/migrate/import/export (ports settings.rs 1:1)
  cookies/      Netscape/JSON conversion + profile store (cookie.rs, utils.rs)
  paths/        download-root rule, path traversal guard, write test
  runtime/      ToolRuntime (bundled/system/preinstalled) + overlay management + ffmpeg probe
  downloader/   backend identification, arg building, process supervisor, line parsers,
                pause/cancel/cleanup, semaphore
  events/       EventSink trait + broadcast bus (tokio::sync::broadcast<JobEvent>)
  jobs/         JobRegistry: create/get/list/cancel/pause/resume; maps ids ↔ processes
```

Desktop `apps/desktop` keeps: Tauri setup, plugins, command wrappers that call core and apply
window flags, and an `EventSink` that forwards `JobEvent` to the legacy event names so the
existing UI contract is unchanged. `mm-server` builds the same core with an `AppPaths` from
env/CLI, an SSE `EventSink`, token auth, static SPA hosting and the `/api/v1` routes.

Deviations to record in `docs/decisions.md` when implemented:

* per-process `current_dir` instead of `set_current_dir`;
* cookie/URL normalisation applied symmetrically;
* notifications always `{ value }`;
* site-arg IDs assigned in core;
* `clear_cookies_on_exit` implemented on the desktop;
* `disk-update` listener removed;
* server jobs are in-memory (same as OLD desktop jobs);
* `paths/download-root` shared by download execution, file serving and disk usage.

## 9. Open items deferred to later phases

Status after the four procedures (server release artifacts, remaining API surface, portable
bundle, final verification). Each item is closed or explicitly accepted:

* **Torrent endpoints — closed.** OLD has **no** torrent code (it is a README roadmap item).
  `/api/v1/torrents` exists and returns `501 torrents_unavailable`; no engine trait was added
  because there is nothing to plug into it yet (decision 65). A qBittorrent-compatible RPC
  adapter is still the intended first engine. This is new behaviour, not a port.
* **Public-API file serving and Range support — closed.** Implemented in Phase 3
  (`GET /files/{path}`, one `Range` header, `416` with `Content-Range: bytes */N`); there is no
  OLD equivalent.
* **Server job durability — accepted.** Job records are in memory and capped at 100
  (decision 64). A restart clears them; OLD has the same durability because the frontend held
  the job list.
* **Portable tool bundling — closed.** The `-tools` archive bundles Python, the three Python
  tools, Deno and ffmpeg; the server finds them in `binaries/` beside the executable
  (decision 66). Bundled ffmpeg sources and the Linux desktop's dependence on the system
  ffmpeg are recorded in section 7.
* **`spotdl` config/temp directory — open, OLD behaviour kept.** The child process runs with
  `current_dir` set to the download directory (`download.rs:623`), so spotdl still creates
  `.spotdl` there. Redirecting its config needs a version-specific flag and would be a
  behavioural change; until then the download directory contains that folder.
* **Sidecars — closed.** The six tracked stub files were deleted, and the last pieces of the
  shell sidecar path are gone with them: the `tauri-plugin-shell` crate, its plugin
  registration, the `shell:default` capability and the `@tauri-apps/plugin-shell` JS
  dependency. Nothing in core, the desktop host or the web UI ever mentions a
  `binaries/<name>-<triple>` file; tools resolve only through `ToolRuntime` (overlay,
  `PATH`, `MM_PYTHON` or the bundled interpreter). `binaries/` stays as the resource
  directory CI fills with `python/`, `deno`, `ffmpeg` and `ffprobe`, which is a different
  thing from a Tauri sidecar.
* **Tool dependencies — closed.** The bundle installs `yt-dlp[default]` and
  `gallery-dl[extra]` (decision 68), so the interpreter carries the YAML/TOML/SOCKS/
  truststore/secretstorage modules and yt-dlp's `default` set explicitly rather than
  through spotdl. Verified by importing every module and running `pip check` against the
  rebuilt `-tools` archive.
