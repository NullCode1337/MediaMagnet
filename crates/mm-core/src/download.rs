use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::path::{Component, Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::cookies::match_cookie;
use crate::error::{Error, Result};
use crate::events::{CoreEvent, DownloadState, NotificationLevel, SharedEventSink};
use crate::policy::{ArgTool, Policy};
use crate::runtime::{hide_console, Tool, ToolRuntime};
use crate::settings::{Settings, SiteArguments};

const MAX_CONCURRENT_DOWNLOADS: usize = 5;
const DEFAULT_OUTPUT_TEMPLATE: &str = "%(title)s.%(ext)s";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    #[serde(rename = "spotdl")]
    SpotDl,
    #[serde(rename = "yt-dlp")]
    YtDlp,
    #[serde(rename = "gallery-dl")]
    GalleryDl,
}

impl Backend {
    pub fn as_str(self) -> &'static str {
        match self {
            Backend::SpotDl => "spotdl",
            Backend::YtDlp => "yt-dlp",
            Backend::GalleryDl => "gallery-dl",
        }
    }

    fn tool(self) -> Tool {
        match self {
            Backend::SpotDl => Tool::SpotDl,
            Backend::YtDlp => Tool::YtDlp,
            Backend::GalleryDl => Tool::GalleryDl,
        }
    }

    fn arg_tool(self) -> ArgTool {
        match self {
            Backend::SpotDl => ArgTool::SpotDl,
            Backend::YtDlp => ArgTool::YtDlp,
            Backend::GalleryDl => ArgTool::GalleryDl,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub id: String,
    pub url: String,
    pub backend: Option<Backend>,
    /// Directory this job writes into. Chosen by the host (settings on desktop,
    /// account/job root on the server); core never reads a client path itself.
    pub base_dir: PathBuf,
}

struct ActiveDownload {
    child: tokio::process::Child,
    backend: Backend,
    base_dir: PathBuf,
    url: String,
    with_cookies: bool,
}

pub struct Engine {
    runtime: Arc<ToolRuntime>,
    config_dir: PathBuf,
    app_data_dir: PathBuf,
    policy: Policy,
    sink: SharedEventSink,
    downloads: Mutex<HashMap<String, ActiveDownload>>,
    semaphore: tokio::sync::Semaphore,
}

impl Engine {
    pub fn new(
        runtime: Arc<ToolRuntime>,
        config_dir: PathBuf,
        app_data_dir: PathBuf,
        policy: Policy,
        sink: SharedEventSink,
    ) -> Arc<Self> {
        Arc::new(Self {
            runtime,
            config_dir,
            app_data_dir,
            policy,
            sink,
            downloads: Mutex::new(HashMap::new()),
            semaphore: tokio::sync::Semaphore::new(MAX_CONCURRENT_DOWNLOADS),
        })
    }

    pub fn policy(&self) -> Policy {
        self.policy
    }

    pub fn settings(&self) -> Settings {
        Settings::load_from(&self.config_dir)
    }

    pub fn save_settings(&self, settings: &Settings) -> Result<()> {
        settings.save_to(&self.config_dir).map_err(Error::Msg)
    }

    pub fn cookies_dir(&self) -> PathBuf {
        self.app_data_dir.join("cookies")
    }

    fn emit(&self, event: CoreEvent) {
        self.sink.emit(event);
    }

    pub fn notify(&self, message: impl Into<String>) {
        self.emit(CoreEvent::Notification {
            level: NotificationLevel::Info,
            message: message.into(),
        });
    }

    fn status(&self, id: &str, value: impl Into<String>) {
        self.emit(CoreEvent::DownloadStatus {
            id: id.to_string(),
            value: value.into(),
        });
    }

    async fn base_command(&self, tool: Tool) -> Result<Command> {
        let resolved = self.runtime.resolve(tool).await?;
        let mut cmd = Command::new(resolved.path);
        hide_console(&mut cmd);
        Ok(cmd)
    }

    /// Returns the backend and, when yt-dlp was rejected, why. The reason is kept
    /// for the error message if the fallback also fails; a normal gallery-dl link
    /// ("no dedicated extractor") is expected and must not surface as a warning.
    async fn identify(&self, url: &str, settings: &Settings) -> (Backend, Option<String>) {
        if url.to_lowercase().contains("spotify.com") {
            return (Backend::SpotDl, None);
        }

        match self.probe_ytdlp(url, settings).await {
            Ok(()) => (Backend::YtDlp, None),
            Err(reason) => {
                println!(
                    "[MediaMagnet] yt-dlp probe failed ({}), falling back to gallery-dl",
                    reason
                );
                (Backend::GalleryDl, Some(reason))
            }
        }
    }

    /// Probes yt-dlp with the same arguments (and cookies) the real download
    /// would use, so the probe cannot fail for a reason the download would not hit.
    async fn probe_ytdlp(&self, url: &str, settings: &Settings) -> std::result::Result<(), String> {
        let (mut cmd, _) = self
            .apply_download_args(url, Backend::YtDlp, settings, true)
            .await
            .map_err(|e| e.to_string())?;
        cmd.args(["--print", "title", "--no-download"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = cmd
            .output()
            .await
            .map_err(|e| format!("could not run yt-dlp: {}", e))?;
        let stderr = String::from_utf8_lossy(&output.stderr);

        if stderr.contains("Falling back on generic") {
            return Err("no dedicated extractor".into());
        }

        if output.status.success() {
            return Ok(());
        }

        let reason = stderr
            .lines()
            .rev()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .unwrap_or("unknown error")
            .to_string();
        Err(reason)
    }

    fn cookie_dir_apply(&self, cmd: &mut Command, link: &str, backend: Backend) -> bool {
        let cookies_dir = self.cookies_dir();
        let Some(path) = match_cookie(&cookies_dir, link) else {
            println!("[MediaMagnet][Download] No cookies directory found, ignoring...");
            return false;
        };
        let Some(path_str) = path.to_str() else {
            return false;
        };
        if backend == Backend::SpotDl {
            cmd.args(["--cookie-file", path_str]);
        } else {
            cmd.args(["--cookies", path_str]);
        }
        println!("[MediaMagnet] Applied cookies from: {}", path_str);
        true
    }

    fn apply_args(
        &self,
        cmd: &mut Command,
        url: &str,
        global_args: &str,
        site_args: &[SiteArguments],
        tool: ArgTool,
    ) -> Result<()> {
        if !global_args.trim().is_empty() {
            let tokens = self
                .policy
                .filter_custom_args(tool, global_args)
                .map_err(|e| {
                    Error::Msg(format!("Configuration Error (global arguments): {}", e))
                })?;
            cmd.args(tokens);
        }

        for item in site_args {
            let applies = !item.domain.trim().is_empty()
                && url.to_lowercase().contains(&item.domain.to_lowercase());
            if applies && !item.args.trim().is_empty() {
                let tokens = self
                    .policy
                    .filter_custom_args(tool, &item.args)
                    .map_err(|e| {
                        Error::Msg(format!(
                            "Configuration Error (site [{}] arguments): {}",
                            item.domain, e
                        ))
                    })?;
                cmd.args(tokens);
                println!(
                    "[MediaMagnet] Applied custom arguments for domain [{}]: {}",
                    item.domain, item.args
                );
            }
        }

        Ok(())
    }

    fn output_template(&self, settings: &Settings) -> String {
        let template = &settings.yt_output_template;
        if self.policy.allow_client_paths || !contains_separator(template) {
            template.clone()
        } else {
            DEFAULT_OUTPUT_TEMPLATE.to_string()
        }
    }

    async fn apply_download_args(
        &self,
        url: &str,
        backend: Backend,
        settings: &Settings,
        with_cookies: bool,
    ) -> Result<(Command, bool)> {
        let mut cmd = self
            .base_command(backend.tool())
            .await
            .map_err(|e| Error::Msg(format!("{} backend error: {}", backend.as_str(), e)))?;

        let lc = url.to_lowercase();

        match backend {
            Backend::SpotDl => {
                cmd.args([
                    "--simple-tui",
                    "--log-level",
                    "INFO",
                    "--output",
                    "{list-name}/",
                ]);

                if !settings.spotdl_format.is_empty() {
                    cmd.args(["--format", &settings.spotdl_format]);
                }

                if !settings.spotdl_bitrate.is_empty() {
                    cmd.args(["--bitrate", &settings.spotdl_bitrate]);
                }

                self.apply_args(
                    &mut cmd,
                    url,
                    &settings.spotdl_global_args,
                    &[],
                    backend.arg_tool(),
                )?;
            }
            Backend::YtDlp => {
                let template = self.output_template(settings);
                cmd.args([
                    "-o",
                    &template,
                    "--progress-template",
                    "%(progress)j",
                    "--newline",
                ]);

                if lc.contains("youtu") || lc.contains("yt.be") {
                    if !settings.yt_format.is_empty() {
                        cmd.args(["-f", &settings.yt_format]);
                    }

                    if settings.yt_embed_thumbnail {
                        cmd.arg("--embed-thumbnail");
                    }

                    if settings.yt_embed_subs {
                        cmd.args(["--write-subs", "--write-auto-sub", "--embed-subs"]);
                    }

                    if settings.yt_restrict_filenames {
                        cmd.arg("--restrict-filenames");
                    }
                }

                self.apply_args(
                    &mut cmd,
                    url,
                    &settings.yt_global_args,
                    &settings.yt_site_args,
                    backend.arg_tool(),
                )?;

                if !settings.user_agent.is_empty() {
                    cmd.args(["--user-agent", &settings.user_agent]);
                }
            }
            Backend::GalleryDl => {
                cmd.args(["-d", "."]);
                self.apply_args(
                    &mut cmd,
                    url,
                    &settings.gdl_global_args,
                    &settings.gdl_site_args,
                    backend.arg_tool(),
                )?;
                if !settings.user_agent.is_empty() {
                    cmd.args(["-a", &settings.user_agent]);
                }
            }
        }

        let cookies_applied = with_cookies && self.cookie_dir_apply(&mut cmd, url, backend);
        cmd.arg(url);
        Ok((cmd, cookies_applied))
    }

    /// Runs a complete download job, emitting events as it goes. Never fails: failures
    /// are reported on the event stream, matching the desktop commands' contract.
    pub async fn run_job(self: &Arc<Self>, request: DownloadRequest) {
        let id = request.id.clone();
        let lc = request.url.to_lowercase();

        if lc.contains("didg") && lc.contains("ls.i") && !lc.contains("original") {
            self.status(&id, "Skipped: Not original version");
            self.emit(CoreEvent::DownloadFinished { id });
            return;
        }

        if !lc.starts_with("http://") && !lc.starts_with("https://") {
            self.emit(CoreEvent::DownloadError {
                id,
                value: "Invalid URL".into(),
            });
            return;
        }

        let _permit = match self.semaphore.acquire().await {
            Ok(permit) => permit,
            Err(_) => {
                self.emit(CoreEvent::DownloadError {
                    id,
                    value: "Failed to acquire download permit".into(),
                });
                return;
            }
        };
        self.status(&id, "Starting...");

        let settings = self.settings();

        let (backend, probe_failure) = match request.backend {
            Some(backend) => (backend, None),
            None => self.identify(&request.url, &settings).await,
        };

        // yt-dlp rejected the link; if gallery-dl cannot handle it either, the
        // yt-dlp reason is the one worth showing.
        let with_probe_reason = |e: String| match &probe_failure {
            Some(reason) => format!("{} (yt-dlp: {})", e, reason),
            None => e,
        };

        self.emit(CoreEvent::DownloadStarted { id: id.clone() });

        let prepared = match self
            .apply_download_args(&request.url, backend, &settings, true)
            .await
        {
            Ok(prepared) => prepared,
            Err(e) => {
                self.emit(CoreEvent::DownloadError {
                    id,
                    value: e.to_string(),
                });
                return;
            }
        };

        let mut succeeded = true;

        if let Err(e) = self
            .download_url(
                prepared.0,
                &request.url,
                backend,
                &request.base_dir,
                &id,
                prepared.1,
            )
            .await
        {
            if prepared.1 {
                println!(
                    "[MediaMagnet] Download failed ({}), retrying without cookies...",
                    id
                );
                self.status(&id, "FAILED: Retrying without cookies...");

                match self
                    .apply_download_args(&request.url, backend, &settings, false)
                    .await
                {
                    Ok((retry_cmd, _)) => {
                        if let Err(e) = self
                            .download_url(
                                retry_cmd,
                                &request.url,
                                backend,
                                &request.base_dir,
                                &id,
                                false,
                            )
                            .await
                        {
                            println!(
                                "[MediaMagnet] Download failed without cookies ({}): {}",
                                id, e
                            );
                            self.emit(CoreEvent::DownloadError {
                                id: id.clone(),
                                value: format!(
                                    "Download failed: {}",
                                    with_probe_reason(e.to_string())
                                ),
                            });
                            succeeded = false;
                        }
                    }
                    Err(e) => {
                        self.emit(CoreEvent::DownloadError {
                            id: id.clone(),
                            value: e.to_string(),
                        });
                        succeeded = false;
                    }
                }
            } else {
                println!("[MediaMagnet] Download failed ({}): {}", id, e);
                self.emit(CoreEvent::DownloadError {
                    id: id.clone(),
                    value: format!("Download failed: {}", with_probe_reason(e.to_string())),
                });
                succeeded = false;
            }
        }

        if succeeded {
            self.emit(CoreEvent::DownloadFinished { id });
        }
    }

    async fn download_url(
        self: &Arc<Self>,
        mut cmd: Command,
        link: &str,
        backend: Backend,
        base_dir: &Path,
        id: &str,
        with_cookies: bool,
    ) -> Result<()> {
        let id = id.to_string();
        std::fs::create_dir_all(base_dir)?;
        cmd.current_dir(base_dir);

        let total_urls = if backend == Backend::GalleryDl {
            println!("\n[MediaMagnet][gallery-dl] Counting total urls...");
            let mut count_cmd = self.base_command(Tool::GalleryDl).await?;
            if with_cookies {
                self.cookie_dir_apply(&mut count_cmd, link, backend);
            }
            count_cmd.current_dir(base_dir);

            let out = count_cmd.args(["-g", link]).output().await?;
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('|'))
                .count()
        } else {
            0
        };

        let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;

        let stdout = child.stdout.take().ok_or("missing stdout")?;
        let stderr = child.stderr.take().ok_or("missing stderr")?;

        self.downloads.lock().map_err(|_| Error::Lock)?.insert(
            id.clone(),
            ActiveDownload {
                child,
                backend,
                base_dir: base_dir.to_path_buf(),
                url: link.to_string(),
                with_cookies,
            },
        );

        let mut out_reader = BufReader::new(stdout).lines();
        let mut err_reader = BufReader::new(stderr).lines();

        if backend == Backend::GalleryDl {
            println!(
                "[MediaMagnet][Download] Found {} total items to download",
                total_urls
            );
        }

        let already_downloaded = Arc::new(AtomicBool::new(false));
        let ad_out = already_downloaded.clone();
        let sink_out = self.sink.clone();
        let id_out = id.clone();

        let out_handle = tokio::spawn(async move {
            match backend {
                Backend::SpotDl => {
                    let mut total_songs: i32 = 1;

                    while let Ok(Some(line)) = out_reader.next_line().await {
                        println!("[spotdl] {}", line);

                        if let Some(rest) = line.strip_prefix("Found ") {
                            if let Some(count_str) = rest.split_whitespace().next() {
                                if let Ok(n) = count_str.parse::<i32>() {
                                    total_songs = n;
                                }
                            }
                            continue;
                        }

                        if line.trim().ends_with("complete") {
                            if let Some((num, rest)) = line.trim().split_once('/') {
                                if let (Ok(done), Ok(total)) = (
                                    num.trim().parse::<i32>(),
                                    rest.split_whitespace().next().unwrap_or("0").parse::<i32>(),
                                ) {
                                    if total > 0 {
                                        total_songs = total;
                                    }
                                    let denom = if total_songs > 0 { total_songs } else { 1 };
                                    let pct = ((done as f64 / denom as f64) * 100.0).min(100.0);

                                    sink_out.emit(CoreEvent::DownloadProgress {
                                        id: id_out.clone(),
                                        value: pct,
                                    });
                                    sink_out.emit(CoreEvent::DownloadStatus {
                                        id: id_out.clone(),
                                        value: format!("{}/{} complete", done as u64, total as u64),
                                    });
                                    continue;
                                }
                            }
                        }

                        if !line.contains("other audio") || !line.contains("http") {
                            sink_out.emit(CoreEvent::DownloadStatus {
                                id: id_out.clone(),
                                value: line,
                            });
                        }
                    }

                    sink_out.emit(CoreEvent::DownloadProgress {
                        id: id_out,
                        value: 100.0,
                    });
                }

                Backend::YtDlp => {
                    while let Ok(Some(line)) = out_reader.next_line().await {
                        if !line.contains('{') {
                            println!("{:#}", line);
                        }
                        if line.contains("has already been downloaded") {
                            ad_out.store(true, Ordering::Relaxed);
                        }
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                            if let (Some("downloading"), Some(percent)) = (
                                json.get("status").and_then(|s| s.as_str()),
                                json.get("_percent").and_then(|p| p.as_f64()),
                            ) {
                                sink_out.emit(CoreEvent::DownloadProgress {
                                    id: id_out.clone(),
                                    value: percent,
                                });

                                let mut msg = String::new();
                                if let Some(p) = json.get("_percent_str").and_then(|p| p.as_str()) {
                                    write!(msg, "Downloading: {}", strip_ansi_codes(p)).ok();
                                }
                                if let Some(s) = json.get("_speed_str").and_then(|s| s.as_str()) {
                                    write!(msg, " Speed: {}", strip_ansi_codes(s)).ok();
                                }
                                if let Some(e) = json.get("_eta_str").and_then(|e| e.as_str()) {
                                    write!(msg, " ETA: {}", strip_ansi_codes(e)).ok();
                                }

                                sink_out.emit(CoreEvent::DownloadStatus {
                                    id: id_out.clone(),
                                    value: msg,
                                });
                            } else if json.get("status").and_then(|s| s.as_str())
                                == Some("finished")
                            {
                                sink_out.emit(CoreEvent::DownloadProgress {
                                    id: id_out.clone(),
                                    value: 100.0,
                                });
                                sink_out.emit(CoreEvent::DownloadStatus {
                                    id: id_out.clone(),
                                    value: "Download finished".into(),
                                });
                            }
                        } else {
                            sink_out.emit(CoreEvent::DownloadStatus {
                                id: id_out.clone(),
                                value: line,
                            });
                        }
                    }
                }

                Backend::GalleryDl => {
                    let mut downloaded = 0usize;

                    while let Ok(Some(line)) = out_reader.next_line().await {
                        sink_out.emit(CoreEvent::DownloadStatus {
                            id: id_out.clone(),
                            value: line.clone(),
                        });

                        if !line.trim_start().starts_with('#') && !line.trim().is_empty() {
                            downloaded += 1;
                            let pct = if total_urls > 0 {
                                (downloaded as f64 / total_urls as f64) * 100.0
                            } else {
                                0.0
                            };
                            sink_out.emit(CoreEvent::DownloadProgress {
                                id: id_out.clone(),
                                value: pct.min(100.0),
                            });
                        }
                    }
                }
            }
        });

        let sink_err = self.sink.clone();
        let id_err = id.clone();
        let err_handle = tokio::spawn(async move {
            let mut error = String::new();

            while let Ok(Some(line)) = err_reader.next_line().await {
                println!("{:#}", line);
                let line_l = line.to_lowercase();

                if line_l.contains("[error") || line_l.contains("error:") {
                    error = line.clone();
                }

                if backend != Backend::YtDlp
                    || line_l.contains("downloaded")
                    || line_l.contains("merged")
                {
                    sink_err.emit(CoreEvent::DownloadStatus {
                        id: id_err.clone(),
                        value: line,
                    });
                } else {
                    let level = if line_l.starts_with("error") {
                        NotificationLevel::Error
                    } else if line_l.starts_with("warning") {
                        NotificationLevel::Warning
                    } else {
                        NotificationLevel::Info
                    };
                    sink_err.emit(CoreEvent::Notification {
                        level,
                        message: line,
                    });
                }
            }
            error
        });

        let mut exit_status: Option<std::process::ExitStatus> = None;

        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            let mut guard = self.downloads.lock().map_err(|_| Error::Lock)?;

            if !guard.contains_key(&id) {
                break;
            }

            match guard.get_mut(&id).unwrap().child.try_wait() {
                Ok(Some(status)) => {
                    exit_status = Some(status);
                    guard.remove(&id);
                    break;
                }
                Err(_) => {
                    guard.remove(&id);
                    break;
                }
                Ok(None) => {}
            }
        }

        let (_, err) = tokio::join!(out_handle, err_handle);
        let error = err.unwrap_or_default();

        if backend == Backend::GalleryDl {
            self.postprocess_directlink(base_dir)?;
        }

        if let Some(status) = exit_status {
            if !status.success() && !already_downloaded.load(Ordering::Relaxed) {
                if !error.trim().is_empty() {
                    return Err(error.into());
                } else {
                    return Err(Error::Msg(format!(
                        "{} process exited with error ({})",
                        backend.as_str(),
                        status
                    )));
                }
            }
        }

        Ok(())
    }

    fn postprocess_directlink(&self, base_dir: &Path) -> Result<()> {
        let dl_path = base_dir.join("directlink");
        if dl_path.exists() && dl_path.is_dir() {
            for entry in std::fs::read_dir(&dl_path)? {
                let path = entry?.path();
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.to_lowercase().contains("part") {
                            continue;
                        }
                        let domain_dir = base_dir.join(name.split('_').next().unwrap_or("unknown"));
                        if !domain_dir.exists() {
                            std::fs::create_dir(&domain_dir)?;
                        }
                        std::fs::rename(&path, domain_dir.join(name))?;
                    }
                }
            }
            if dl_path.read_dir()?.next().is_none() {
                std::fs::remove_dir(dl_path)?;
            }
        }
        Ok(())
    }

    async fn clean_tempfiles(
        &self,
        base: &Path,
        url: &str,
        backend: Backend,
        with_cookies: bool,
    ) -> Result<()> {
        match backend {
            Backend::SpotDl => return Ok(()),
            Backend::YtDlp => {
                let mut cmd = self.base_command(Tool::YtDlp).await?;
                if with_cookies {
                    self.cookie_dir_apply(&mut cmd, url, backend);
                }
                let template = self.output_template(&self.settings());
                cmd.args(["-o", &template, "--print", "filename", "--no-download", url])
                    .current_dir(base);

                let out = cmd.output().await?;
                for line in String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                {
                    let target = base.join(Path::new(line).file_name().unwrap_or_default());
                    for suffix in ["part", "ytdl"] {
                        let path = match target.extension() {
                            Some(ext) => target.with_extension(format!(
                                "{}.{}",
                                ext.to_string_lossy(),
                                suffix
                            )),
                            None => target.with_extension(suffix),
                        };
                        if path.is_file() {
                            let _ = std::fs::remove_file(&path);
                        }
                    }
                }
            }
            Backend::GalleryDl => {
                let mut cmd = self.base_command(Tool::GalleryDl).await?;
                if with_cookies {
                    self.cookie_dir_apply(&mut cmd, url, backend);
                }
                cmd.args(["-s", "-d", ".", "--print", "{_directory}", url])
                    .current_dir(base);

                let out = cmd.output().await?;
                let mut deleted = HashSet::new();
                for line in String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                {
                    let mut parts = Path::new(line).components().filter_map(|c| {
                        if let Component::Normal(s) = c {
                            Some(s.to_owned())
                        } else {
                            None
                        }
                    });
                    if let (Some(a), Some(b)) = (parts.next(), parts.next()) {
                        let sub = base.join(a).join(b);
                        if deleted.insert(sub.clone()) && sub.starts_with(base) && sub.is_dir() {
                            let _ = std::fs::remove_dir_all(&sub);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn pause(&self, id: &str) -> Result<()> {
        let entry = self.downloads.lock().map_err(|_| Error::Lock)?.remove(id);

        match entry {
            Some(mut entry) => {
                entry
                    .child
                    .kill()
                    .await
                    .map_err(|e| Error::Msg(format!("Kill failed: {}", e)))?;
                entry
                    .child
                    .wait()
                    .await
                    .map_err(|e| Error::Msg(format!("Wait failed: {}", e)))?;
                self.status(id, "Paused");
                self.emit(CoreEvent::DownloadState {
                    id: id.to_string(),
                    state: DownloadState::Paused,
                });
                Ok(())
            }
            None => Err(Error::Msg(format!("No active download with id: {}", id))),
        }
    }

    pub async fn cancel(&self, id: &str) -> Result<()> {
        let entry = self.downloads.lock().map_err(|_| Error::Lock)?.remove(id);

        if let Some(mut entry) = entry {
            let _ = entry.child.kill().await;
            let _ = entry.child.wait().await;

            if let Err(e) = self
                .clean_tempfiles(
                    &entry.base_dir,
                    &entry.url,
                    entry.backend,
                    entry.with_cookies,
                )
                .await
            {
                println!("[MediaMagnet] Cleanup failed during cancellation: {}", e);
            }
        }

        self.emit(CoreEvent::DownloadError {
            id: id.to_string(),
            value: "Download cancelled and files permanently deleted".into(),
        });
        self.emit(CoreEvent::DownloadState {
            id: id.to_string(),
            state: DownloadState::Cancelled,
        });
        Ok(())
    }

    pub async fn cancel_all(&self) -> Result<()> {
        let children: Vec<(String, ActiveDownload)> = {
            let mut guard = self.downloads.lock().map_err(|_| Error::Lock)?;
            guard.drain().collect()
        };

        for (id, mut entry) in children {
            if let Err(e) = entry.child.kill().await {
                eprintln!("[MediaMagnet] Failed to kill {}: {}", id, e);
            }
            let _ = entry.child.wait().await;

            self.emit(CoreEvent::DownloadError {
                id: id.clone(),
                value: "Download cancelled".into(),
            });
            self.emit(CoreEvent::DownloadState {
                id,
                state: DownloadState::Cancelled,
            });
        }

        Ok(())
    }

    pub fn active_ids(&self) -> Vec<String> {
        self.downloads
            .lock()
            .map(|guard| guard.keys().cloned().collect())
            .unwrap_or_default()
    }
}

fn contains_separator(template: &str) -> bool {
    template.contains('/') || template.contains('\\')
}

fn strip_ansi_codes(s: &str) -> std::borrow::Cow<'_, str> {
    static RE: std::sync::LazyLock<regex::Regex> =
        std::sync::LazyLock::new(|| regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap());
    RE.replace_all(s, "")
}
