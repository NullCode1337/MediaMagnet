# Decisions

Deviations from `MediaMagnet-old` and calls made where OLD was ambiguous. Each entry
says what changed and why.

## Phase 1 — core extraction

### Downloading

1. **Per-process working directory.** OLD called `std::env::set_current_dir()` in
   `set_download_path`, so concurrent downloads to different folders raced on the
   process CWD and the path stayed changed after a download. Core now passes
   `Command::current_dir(base_dir)` to every child (download, gallery-dl pre-count,
   temp-file cleanup) and never mutates the process CWD.
2. **Base directory is chosen by the host.** OLD computed it inside
   `download_url`/`clean_tempfiles` from settings. `DownloadRequest` now carries
   `base_dir`; desktop fills it with `host::resolve_download_path(settings, host)`,
   the same rule OLD used (default `~/Downloads/MediaMagnet`, a path containing
   `mediamagnet` used verbatim, any other path gets `/MediaMagnet` appended). This
   is what lets server mode ignore client paths later.
3. **Cookie filtering bug fixed.** OLD's cookie UI stored `google.com` as
   `googlecom` (`replace(/[./]/g, "")`), while `apply_cookies` matched the file
   stem against the URL, so files saved through the UI never matched. The UI now
   only strips path separators and core validates the domain (no `..`, `/`, `\`,
   NUL, whitespace, single path component).
4. **`delete_cookie` is confined to the cookie store.** OLD deleted any path the
   frontend passed. Core canonicalises and requires the target to live inside the
   cookie directory.
5. **Temp JSON cookie file removed.** `save_cookie(Content)` wrote
   `cookies/temp_json.json` and then deleted it; conversion now happens in memory.
6. **`cancel_download` no longer needs `url`.** Cleanup uses the URL the engine
   recorded for the job. The desktop command keeps accepting `url` (now optional,
   ignored) so the existing frontend call keeps working.
7. **Cookies now retried correctly.** OLD's "retry without cookies" rebuilt the
   command through `apply_download_args`, which re-applied the same cookie file;
   the retry now really omits cookies.
8. **URL is appended by core.** OLD appended `url` in two places around the
   cookie retry; `apply_download_args` now always appends it exactly once.
9. **The `didg` + `ls.i` skip predicate is copied verbatim** (including its
   oddity) and has its own comment.
10. **Tool discovery cache is per-`ToolRuntime`**, not process-global, so tests and
    future server/multi-engine embeddings do not share state.
11. **`identify()` no longer needs an `AppHandle`**; it resolves tools through
    `ToolRuntime`. Behaviour (spotify host → spotdl, yt-dlp probe, gallery-dl
    fallback) is unchanged.

### Notifications

12. **One payload shape.** OLD emitted `notification` as `{id,value}` from the
    downloader and as a bare string from `notify`/cookie commands; the frontend
    read `.value`, so bare-string notifications rendered empty toasts. All
    notifications are now `{level, message}` (`CoreEvent::Notification`), and the
    frontend toast handler reads `message`.
13. **Desktop toast path.** `notify` keeps OLD's semantics: native notification
    when `native_notifications` is on, otherwise an in-app `notification` event.
    Core's `Host::notify` returns whether it delivered natively.

### Settings

14. **Settings are addressed by directory.** `Settings::save_to(dir)` /
    `load_from(dir)` replace the `AppHandle`-based methods so core does not depend
    on Tauri. Desktop passes `app_config_dir()`, exactly as before.
15. **Window behaviour stays in the desktop crate.** `always_on_top` and
    `decorations = !custom_titlebar` are applied by `app::apply_window_settings`.
16. **`dark_mode` behaviour unchanged** (persisted by the frontend, not applied on
    startup); `mode-watcher` remains the source of truth.
17. **`clear_cookies_on_exit` remains a stored no-op**, as in OLD.
18. **Desktop-only settings have a server projection.** `Settings::server_view()`
    drops `always_on_top`, `custom_titlebar`, `custom_titlebar_type`,
    `native_notifications`, `clear_cookies_on_exit`, `download_path`;
    `force_desktop_fields()` restores them from local state after an HTTP update.

### Dead code

19. **Removed** (no caller in OLD's frontend): `overwrite_json`, `check_links`,
    the `link-event` event, and the never-emitted `disk-update` listener. The
    `links.json` file is still created/validated at startup to keep OLD's data
    layout.

### Policy (server mode groundwork)

20. **`Policy` in core.** `Policy::desktop()` is fully permissive and preserves
    OLD behaviour exactly (no argument filtering). `Policy::server()` refuses all
    custom args; `Policy::server_with_custom_args()` (used when
    `MM_ALLOW_CUSTOM_ARGS` is on) allows them but applies a per-tool deny-list
    (`--exec*`, `--load-info-json`, `--batch-file/-a`, config files, plugin dirs,
    `--netrc-cmd`, `--cookies-from-browser`, output/path overrides,
    gallery-dl `--postprocessor*`, spotdl `--output`/`--cookie-file`/`--config`),
    matching both `--flag value` and `--flag=value`, plus attached short flags.
    Server policy also replaces output templates containing path separators with
    the default `%(title)s.%(ext)s`.
21. **`files::safe_join` / `parse_range`** added for the future HTTP layer, with
    tests (traversal, symlink escape, range forms). Nothing in Phase 1 uses them.

### Frontend

22. **`tauri-plugin-m3` dropped from the web workspace.** It is Android-only in
    OLD (`setBarColor`) and the rebuild does not target Android (see audit §4).
    The call was removed from `App.svelte`. If Android returns, it comes back as
    a mobile-only import in the desktop crate.
23. **Frontend copied verbatim in Phase 1**, except the two changes above and the
    notification payload type. Rewrite happens in Phase 2.
