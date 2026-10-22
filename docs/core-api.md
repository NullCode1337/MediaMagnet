# mm-core public API

`mm-core` holds everything that used to live in `src-tauri/src/commands` minus the
Tauri types. It has no dependency on Tauri, a window, or the desktop app.

## Entry points

```rust
pub struct Engine; // Arc<Engine>
impl Engine {
    pub fn new(runtime: Arc<ToolRuntime>, config_dir: PathBuf, app_data_dir: PathBuf,
               policy: Policy, sink: SharedEventSink) -> Arc<Engine>;

    pub async fn run_job(self: &Arc<Self>, request: DownloadRequest);

    pub async fn pause(&self, id: &str) -> Result<()>;
    pub async fn cancel(&self, id: &str) -> Result<()>;
    pub async fn cancel_all(&self) -> Result<()>;
    pub fn active_ids(&self) -> Vec<String>;

    pub fn settings(&self) -> Settings;
    pub fn save_settings(&self, settings: &Settings) -> Result<()>;
    pub fn cookies_dir(&self) -> PathBuf;
    pub fn policy(&self) -> Policy;
    pub fn notify(&self, message: impl Into<String>);
}
```

`run_job` never returns an error: every failure is reported as a
`CoreEvent::DownloadError`, which is what the OLD `downloader` command did. Per-job
concurrency is still capped at five. `cancel`/`pause` look the job up by id;
`cancel` runs temp-file cleanup with the recorded URL and base directory.

```rust
pub struct DownloadRequest {
    pub id: String,
    pub url: String,
    pub backend: Option<Backend>,          // None = auto-detect, as in OLD
    pub base_dir: PathBuf,                 // chosen by the host, never by a client
    pub settings: Option<Settings>,        // job-local config; None = host settings
    pub cookies: Option<Vec<JobCookie>>,   // None = host store; Some = these only
}

pub struct JobCookie { pub domain: String, pub content: String }
```

`settings: Some(..)` replaces the host's settings for that job; the policy still
filters custom arguments. `cookies: Some(..)` never falls back to the host's
cookie directory: the longest matching domain is written to
`<app_data>/job-cookies/` (JSON converted to Netscape) and removed when the job
ends. The web SPA uses both fields, so its browser-local store drives downloads
without writing anything into the server's data directory.

## Events

```rust
pub trait EventSink: Send + Sync { fn emit(&self, event: CoreEvent); }
pub type SharedEventSink = Arc<dyn EventSink>;

pub enum CoreEvent {
    DownloadStarted  { id },
    DownloadState    { id, state: DownloadState }, // paused | cancelled | failed | finished
    DownloadProgress { id, value: f64 },
    DownloadStatus   { id, value: String },
    DownloadError    { id, value: String },
    DownloadFinished { id },
    Notification     { level: NotificationLevel, message: String }, // info|success|warning|error
}
```

The desktop crate maps these to its existing IPC events (`download-progress`,
`download-status`, …) in `apps/desktop/src-tauri/src/events.rs`; the server will map
the same enum onto SSE in Phase 3. `DownloadState` is additive — it lets a client
learn that a job was paused/cancelled without parsing status text.

## Settings

```rust
pub struct Settings { /* same 23 fields as OLD, same defaults */ }
impl Settings {
    pub fn save_to(&self, config_dir: &Path) -> Result<(), String>; // atomic, fsync, dir fsync
    pub fn load_from(config_dir: &Path) -> Settings;                // sanitises, backs up corrupt
    pub fn from_value(value: &Value) -> Result<Settings, String>;   // sanitises like OLD
    pub fn to_value(&self) -> Value;
    pub fn server_view(&self) -> Value;                             // desktop-only fields removed
}
pub fn force_desktop_fields(next: &mut Settings, current: &Settings);
pub fn sanitize_settings(value: &Value) -> (Value, SanitizeReport);
pub const DESKTOP_ONLY_KEYS: &[&str];
```

`server_view` drops `always_on_top`, `custom_titlebar`, `custom_titlebar_type`,
`native_notifications`, `clear_cookies_on_exit`, `download_path`. Server code uses it
for GET, and `force_desktop_fields` after PUT so HTTP clients cannot change local
window behaviour.

## Paths and host services

```rust
pub trait Host: Send + Sync {
    fn default_download_dir(&self) -> PathBuf;
    fn bundled_tools_dir(&self) -> Option<PathBuf> { None }
    fn notify(&self, message: &str) -> bool { false }
    fn open_path(&self, path: &Path) -> Result<()>;
}
pub fn resolve_download_path(settings: &Settings, host: &dyn Host) -> PathBuf;
pub fn disk_usage(path: &Path) -> Result<DiskUsage>;
```

The desktop `Host` wraps `app.path().download_dir()`, `notify-send`/the notification
plugin, and `opener`. Phase 3's server `Host` returns `MM_DOWNLOAD_ROOT` and no-ops
notifications.

## Cookies

```rust
pub fn save_cookie(dir: &Path, domain: &str, input: &CookieInput) -> Result<String>;
pub fn list_cookies(dir: &Path) -> Result<BTreeMap<String, String>>;
pub fn clear_cookies(dir: &Path) -> Result<usize>;
pub fn delete_cookie(dir: &Path, path: &Path) -> Result<()>;   // confined to `dir`
pub fn validate_domain(domain: &str) -> Result<String>;
pub fn match_cookie(dir: &Path, link: &str) -> Option<PathBuf>;
pub fn is_netscape(path: &Path) -> Result<bool>;
pub fn netscape_from_json(content: &str) -> Result<String>;
pub fn convert_json(src: &Path, dest: &Path) -> Result<()>;
pub struct CookieInput { FilePath(String) | Content(String) } // {type,value} tagged
```

## Policy

```rust
pub struct Policy { pub allow_custom_args: bool, pub allow_client_paths: bool }
impl Policy {
    pub fn desktop() -> Self;                 // permissive — OLD behaviour
    pub fn server() -> Self;                  // no custom args, no client paths
    pub fn server_with_custom_args() -> Self; // deny-list filtering only
    pub fn filter_custom_args(&self, tool: ArgTool, args: &str) -> Result<Vec<String>>;
}
```

## Tool runtime

```rust
pub struct ToolRuntime; // Arc<ToolRuntime>
impl ToolRuntime {
    pub fn new(config: RuntimeConfig) -> Self;
    pub async fn resolve(&self, tool: Tool) -> Result<ResolvedTool, ToolError>;
    pub async fn reinstall_python_deps(&self, tools: &[Tool]) -> Result<(), ToolError>;
    pub fn reset_python_overlay(&self) -> std::io::Result<()>;
    pub fn deno(&self) -> Option<PathBuf>;
    pub fn ffmpeg(&self) -> Option<PathBuf>;
}
pub enum Tool { YtDlp, GalleryDl, SpotDl } // name()/module()/package()
pub enum ToolSource { Preinstalled, System, Bundled, Overlay }
pub struct ResolvedTool {
    pub tool: Tool,
    pub program: PathBuf,        // tool binary or Python interpreter
    pub args: Vec<String>,       // ["-m", "<module>"] for Python modes
    pub env: Vec<(String, String)>, // PYTHONPATH, PYTHONNOUSERSITE, PATH
    pub source: ToolSource,
}
pub struct RuntimeConfig {
    pub bundled_tools_dir: Option<PathBuf>, // resources/binaries
    pub overlay_dir: Option<PathBuf>,       // <app_data>/py-overlay
    pub python_path: Option<PathBuf>,       // MM_PYTHON
}
pub const TOOL_TARGET: &str; // build target triple, e.g. x86_64-unknown-linux-gnu
```

Resolution order (cached per tool):

1. **Overlay** — `<py-overlay>/<module>` exists; the bundled interpreter runs
   `<python> -m <module>` with the overlay first on `PYTHONPATH`.
2. **Preinstalled / System / Bundled** — every PATH hit for `<name>`,
   `<name>-dev` and `<name>-git` (skipping paths containing `mediamagnet`) is
   validated with `--version`, as is `<python> -m <module> --version` for
   `MM_PYTHON` and the bundled interpreter. The highest parsed version wins,
   a dev/git/nightly build wins a tie, and PATH wins a tie with an interpreter.
   `bundled_tools_dir/python` is a python-build-standalone install with the
   three tools baked into its site-packages; PATH alone is the Docker/server
   mode.

   If no candidate responds, the first available interpreter runs the module
   (bundled first, then `MM_PYTHON`).

Every resolved tool also gets `bundled_tools_dir` prepended to `PATH`, so a bundled
`deno`/`ffmpeg`/`ffprobe` is visible to yt-dlp and spotdl. `reinstall_python_deps`
runs `pip install --target <overlay> --upgrade <packages>` with the first available
interpreter (bundled, then system); `reset_python_overlay` deletes the directory.

## Files and ranges (for the HTTP layer)

```rust
pub fn safe_join(root: &Path, relative: &str) -> Result<PathBuf>;   // traversal + symlink guard
pub fn list_dir(root: &Path, relative: &str) -> Result<Vec<DirEntry>>;
pub fn parse_range(header: &str, len: u64) -> Result<Option<ByteRange>>;
```

Both have unit tests. Phase 3's download-file endpoints build on them.

## Desktop command mapping

| OLD command | Now |
|---|---|
| `downloader` | builds a `DownloadRequest`, spawns `engine.run_job` |
| `pause_download` / `cancel_download` / `cancel_all_downloads` | `engine.pause` / `cancel` / `cancel_all` |
| `settings` (check/reset) | `Settings::load_from` / `default().save_to` + `apply_window_settings` |
| `update_settings` | `Settings::from_value` + save + `apply_window_settings` |
| `save_cookie` / `get_cookies` / `clear_cookies` / `delete_cookie` | `cookies::*` |
| `check_download_path` | unchanged (UI-driven directory probe) |
| `get_free_space` | `disk_usage` |
| `notify` | `Host::notify`, falling back to `engine.notify` |
| `overwrite_json`, `check_links` | removed (no callers) |
