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
   the retry now really omits cookies. This includes the gallery-dl `-g` pre-count
   and the cancellation cleanup, which used to apply the cookie file regardless of
   the attempt (they now follow the `with_cookies` flag of the running attempt).
8. **URL is appended by core.** OLD appended `url` in two places around the
   cookie retry; `apply_download_args` now always appends it exactly once.
9. **The `didg` + `ls.i` skip predicate is copied verbatim** (including its
   oddity) and has its own comment.
10. **Tool discovery cache is per-`ToolRuntime`**, not process-global, so tests and
    future server/multi-engine embeddings do not share state.
11. **`identify()` no longer needs an `AppHandle`**; it resolves tools through
    `ToolRuntime`. The yt-dlp probe now runs with the same arguments and cookies
    the real download uses (OLD probed with none, so a cookie-protected or
    format-restricted link could still pass the probe and then fail). A failed
    probe is logged with its reason (yt-dlp's last stderr line, or "no dedicated
    extractor" for the generic fallback) instead of falling back silently. The
    reason is not shown as a warning by itself — gallery-dl sites fail the probe
    by design — it is appended to the error if gallery-dl also fails
    (`... (yt-dlp: <reason>)`), so a YouTube bot-check is visible as such.

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

## Phase 2 — frontend split into api / ui / web

24. **Three packages, one direction of dependency.** `packages/api` holds only
    types (`Config`, `Transport`, `TransportEvent`, …) and no runtime dependency
    beyond itself. `packages/ui` holds components and stores and imports
    `@mediamagnet/api`. `apps/web` is the shell: it owns the CSS entry, picks a
    transport, calls `bindTransport()` and mounts `App`. `packages/ui` imports
    no `@tauri-apps/*` and issues no HTTP, so the server shell (Phase 3) reuses
    it unchanged.
25. **`transport` is a Proxy, not a factory argument.** Components import the
    `transport` object directly, as they imported `invoke` before. The shell
    binds the real implementation before `mount()`. Using it unbound throws, so a
    missing `bindTransport()` fails loudly instead of silently no-op'ing.
26. **`App` is exported from `@mediamagnet/ui`; the titlebar is conditional on
    the transport.** OLD hid it with `currentPlatform !== "android"`. It is now
    `transport.kind === "desktop"`, which is also correct for the browser build.
    Same for the window/desktop sections in General, Download and Privacy.
27. **`settings.init()` is called by the shell, not in a constructor.** OLD
    started loading in `Settings`'s constructor, which fired an `invoke` before
    the transport could be bound. The shell now awaits it before mounting.
28. **`$lib` is an alias into the UI package.** Components keep OLD's `$lib/...`
    import style; `apps/web` maps `$lib` to `../../packages/ui/src/lib` in both
    Vite and tsconfig. Tailwind needs `@source "../../../packages/ui/src"` in
    `apps/web/src/app.css`, otherwise no utility used only in the package is
    emitted.
29. **Tauri JS packages moved from the workspace root into `apps/web`.** Only the
    desktop shell uses them. They are pinned to the Rust crate versions resolved
    in `Cargo.lock` (2.12.4 / 2.8.3 / …) because `tauri dev` refuses to start on a
    major/minor mismatch — the previous `~2` ranges resolved to older minors.
30. **`settings-schema` keeps Rust-assigned site-arg ids.** OLD regenerated a
    `crypto.randomUUID()` for every entry on every normalize, so ids changed on
    each load and the UI's `{#each}` keys churned. An existing `id` is now
    preserved; one is only invented when the source has none.
31. **Desktop-only keys are dropped on save for HTTP.** `settings.outgoing()`
    strips `DESKTOP_ONLY_KEYS` when `transport.kind !== "desktop"`, matching
    `Settings::server_view()`. The UI still edits a full `Config`; missing fields
    from a server answer are filled with defaults by `hydrate()`.
32. **Unsubscription is settled in one place.** Tauri's `listen()` and
    `onBackButtonPress()` return promises that resolve late, and two of them
    resolve to different shapes (`UnlistenFn` vs `PluginListener`). The desktop
    transport wraps both in a `pending()` helper that returns a synchronous
    unsubscribe, unregisters immediately if already disposed, and gates event
    delivery so a handler cannot fire after `unsubscribe()`.

## Phase 3 — server binary and HTTP transport

33. **One engine, two hosts.** `mm-server` builds the same `Engine` as the
    desktop crate and swaps only the `Host`/`Policy`. Everything the desktop
    does to a file (tempfile cleanup, cancellation, cookie matching) therefore
    behaves identically on the server; the server adds no download logic.
34. **`safe_join` produced a path with a trailing separator.** Joining an empty
    residual `PathBuf` appends `/`, so `/root/hello.txt` resolved to
    `/root/hello.txt/` and `metadata` failed with `NotADirectory` (errno 20) on
    Unix. Every file download 404'd while the directory listing worked. The tail
    is now joined only when it is non-empty; `safe_join_has_no_trailing_separator`
    asserts it. Phase 1's tests missed it because `Path` equality ignores the
    trailing separator.
35. **`download-state` travels as `value`.** `CoreEvent::DownloadState` serialises
    its state as `state`, while the UI expects `{ id, value }` like every other
    value event. `events::to_transport_event` renames it on the wire so both
    transports deliver an identical `TransportEvent`. A test pins the key.
36. **Unknown download ids are 404 for cancel as well as pause.** Core's
    `cancel()` returns `Ok` and broadcasts a cancellation for ids it never
    tracked, which would make `DELETE /downloads/<id>` claim success and push a
    spurious event to every SSE client. The route checks `active_ids()` first.
    Core is unchanged because the desktop app relies on cancel being
    idempotent.
37. **The SPA is served from `MM_STATIC_DIR`, defaulting to `public/` beside the
    binary.** Unknown paths return `index.html` so a hard reload works, except
    paths containing `..` and paths that look like assets (`assets/…` or a
    dotted segment), which 404 — otherwise a broken build is indistinguishable
    from a valid page.
38. **Transport selection happens at runtime, not at build time.** `main.ts`
    tests for `__TAURI_INTERNALS__` and dynamically imports the matching
    transport, so `apps/web/dist` is a single artefact used by both targets.
    The HTTP module is only fetched in a browser and the Tauri module only in
    the webview.
39. **Token entry is a prompt, and a 401 retries once.** The server may be
    started with `MM_TOKEN`; the first 401 offers a token prompt, stores it, and
    replays the request. `?token=` on a link is read, stripped from the URL and
    remembered, because `EventSource` cannot set an `Authorization` header — the
    SSE endpoint accepts both.
40. **Browser-only limitations are explicit, not hidden.** The server owns the
    download root, so `checkDownloadPath` is a no-op. `openPath` throws with a
    reason instead of silently doing nothing, the Cookies list only keeps its
    delete action, and the sidebar's "Open folder" is hidden. Local picks are
    tracked in memory by a synthetic `file://` id, and exports become a Blob
    download because the browser has no save-picker path to hand back.
41. **Vite serves `static/`, not `public/`.** The app's files lived in `static/`
    while Vite's default `publicDir` is `public/`, so `favicon.png` was never
    copied into `dist`. `publicDir: 'static'` fixes it and `index.html` points at
    the real icon rather than OLD's `vite.svg` reference.

## Phase 4 — tool runtime, packaging and CI

42. **No frozen sidecars.** yt-dlp and gallery-dl are no longer shipped as
    PyInstaller blobs. The desktop bundle carries a python-build-standalone
    interpreter per target triple with `yt-dlp`, `gallery-dl` and `spotdl`
    installed into its site-packages at build time; the tools run as
    `python -m <module>`.
43. **Four runtime modes.** `ToolSource` is `Overlay`, `Preinstalled`, `System` or
    `Bundled`, in that resolution order. The overlay is checked first because it
    only exists after the user explicitly ran `reinstall_python_deps`. The
    preinstalled check keeps OLD's `which` + `--version` validation, so a stale
    system binary is still skipped.
44. **System mode is `MM_PYTHON`, not a settings key.** OLD had no Python path
    field and adding one would change the settings UI. The environment variable
    is read by both the desktop and the server.
45. **Deno is found on PATH, not through `--js-runtimes`.** The bundled tools
    directory is prepended to `PATH` for every child, so yt-dlp's default deno
    probe finds the bundled binary. Passing `--js-runtimes deno:<path>` would
    break against older user-installed yt-dlp builds that lack the flag.
46. **The overlay lives in app data.** macOS `.app`, AppImage and Program Files
    are read-only, so `reinstall_python_deps` installs into
    `<app_data>/py-overlay` with `pip install --target`, and that directory is
    prepended to `PYTHONPATH`. `reset_python_deps` (new command) deletes it and
    clears the resolution cache. Both are desktop commands only; the server's
    tools are part of its image.
47. **ffmpeg is bundled on Windows and macOS, system on Linux and the server.**
    A missing ffmpeg is not a preflight error: jobs that do not need merging
    still work, and yt-dlp reports the failure itself for those that do.
48. **CI builds the bundled Python on each target runner.** A single Python
    standalone release is downloaded per triple and the pinned tool versions are
    installed with that interpreter, so wheels match the runner's OS and arch.
    arm64 jobs use `ubuntu-24.04-arm`, `windows-11-arm` and `macos-latest`;
    x86_64 macOS uses `macos-15-intel` (the `macos-13` runner is retired).
49. **Packaging switches from `externalBin` to `resources`.** `tauri.conf.json`
    ships the `binaries/` directory, where CI places `python/`, `deno`,
    `ffmpeg` and `ffprobe`. The checked-in 92-byte x86_64 stub files are no
    longer referenced by any config; CI removes them from the bundle. They can
    be deleted from the repository when convenient.
50. **The server image is `python:3.12-slim-bookworm` + Debian's ffmpeg.** Python
    3.12 is the audit's pinned interpreter (widest cp312/abi3 wheel coverage);
    Debian bookworm provides glibc ≥ 2.28. Deno is copied from the official
    `denoland/deno:bin` image. The same Dockerfile is built for
    `linux/amd64` and `linux/arm64` by buildx in the release workflow.

## Browser storage for the web SPA

51. **Settings and cookies live in the browser.** The HTTP transport reads and
    writes `mm:settings:v1` and `mm:cookies:v1` in `localStorage`; the SPA no
    longer calls `GET/PUT /settings`, `POST /settings/reset` or the `/cookies`
    routes. Those routes stay for API clients that drive the server directly.
    First run shows the shared defaults from `@mediamagnet/api`, not the
    server's saved config.
52. **Jobs carry their settings.** `POST /downloads` accepts optional `settings`
    and `cookies`; core applies them to that job only. Without them the job uses
    the host's saved settings and cookie directory, so the desktop app and
    non-SPA API clients are unchanged. The server policy still filters custom
    arguments and output templates, so a client cannot gain argv it was denied.
53. **Client cookies never fall back to the server store.** When `cookies` is
    present, core matches the URL against the longest domain, converts JSON to
    Netscape if needed, writes one file under `<app_data>/job-cookies/`, and
    deletes it when the job (or a concurrent cancellation) releases it. An empty
    or non-matching list means the job runs without cookies — it does not pick
    up the server operator's cookies.
54. **Import/export already round-trips the local config.** The panel reads
    `settings.config`, which is the localStorage copy; import writes through
    `updateSettings` and therefore persists there. Export still goes through the
    clipboard or the browser's download prompt, not the backend.
