use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path};
use std::process::Stdio;
use std::sync::{Arc, Mutex, OnceLock};
use tauri::{Emitter, Manager};
use tauri_plugin_shell::ShellExt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use super::settings::Settings;
use super::utils::set_download_path;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Serialize, Clone)]
pub struct ProgressPayload {
    pub id: String,
    pub value: f64,
}

#[derive(Serialize, Clone)]
pub struct StatusPayload {
    pub id: String,
    pub value: String,
}

#[derive(Serialize, Clone)]
pub struct IdPayload {
    pub id: String,
}

#[derive(Clone, PartialEq, Copy)]
enum Backend {
    SpotDL,
    YtDlp,
    GalleryDl,
}

impl Backend {
    async fn identify(app: &tauri::AppHandle, url: &str) -> Self {
        if url.to_lowercase().contains("spotify.com") {
            return Self::SpotDL;
        }

        if let Ok(mut check_cmd) = base_command(app, "yt-dlp").await {
            check_cmd
                .args(["--print", "title", "--no-download", url])
                .stderr(Stdio::piped());

            if let Ok(output) = check_cmd.output().await {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if output.status.success() && !stderr.contains("Falling back on generic") {
                    return Self::YtDlp;
                }
            }
        }

        Self::GalleryDl
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::SpotDL => "spotdl",
            Self::YtDlp => "yt-dlp",
            Self::GalleryDl => "gallery-dl",
        }
    }
}

struct ActiveDownload {
    child: tokio::process::Child,
    backend: Backend,
}

fn get_downloads() -> &'static Arc<Mutex<HashMap<String, ActiveDownload>>> {
    static DOWNLOADS: OnceLock<Arc<Mutex<HashMap<String, ActiveDownload>>>> = OnceLock::new();
    DOWNLOADS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

fn get_semaphore() -> &'static Arc<tokio::sync::Semaphore> {
    static SEMAPHORE: OnceLock<Arc<tokio::sync::Semaphore>> = OnceLock::new();
    SEMAPHORE.get_or_init(|| Arc::new(tokio::sync::Semaphore::new(5)))
}

#[inline]
fn hide_cmd(_cmd: &mut Command) {
    #[cfg(target_os = "windows")]
    _cmd.creation_flags(0x08000000);
}

fn apply_cookies(
    cmd: &mut Command,
    app: &tauri::AppHandle,
    link: &str,
    backend: Backend,
) -> Result<bool> {
    let cookies_dir = app.path().app_data_dir().unwrap().join("cookies");
    if !cookies_dir.exists() {
        println!("[MediaMagnet][Download] No cookies directory found, ignoring...");
        return Ok(false);
    }

    if let Ok(entries) = std::fs::read_dir(&cookies_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if link.to_lowercase().contains(&file_stem.to_lowercase()) {
                        if let Some(path_str) = path.to_str() {
                            if backend == Backend::SpotDL {
                                cmd.args(["--cookie-file", path_str]);
                            } else {
                                cmd.args(["--cookies", path_str]);
                            }
                            println!("[MediaMagnet] Applied cookies from: {}", path_str);
                            return Ok(true);
                        }
                    }
                }
            }
        }
    }
    Ok(false)
}

async fn base_command(app: &tauri::AppHandle, command: &str) -> Result<Command> {
    static VERSION: OnceLock<Arc<Mutex<HashSet<String>>>> = OnceLock::new();
    let version = VERSION.get_or_init(|| Arc::new(Mutex::new(HashSet::new())));

    if let Ok(paths) = which::which_all(command) {
        let filtered = paths.filter(|path| {
            // no sidecar
            !path
                .to_string_lossy()
                .to_ascii_lowercase()
                .contains("mediamagnet")
        });

        for bin_path in filtered {
            let mut check_cmd = Command::new(&bin_path);
            hide_cmd(&mut check_cmd);

            let check_result = check_cmd
                .arg("--version")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .await;

            if let Ok(output) = check_result {
                if output.status.success() {
                    let mut checked_set = version.lock().map_err(|_| "Lock failed")?;
                    if !checked_set.contains(command) {
                        println!(
                            "[MediaMagnet][Download] Backend: Local {}, Version: {}",
                            bin_path.display(),
                            String::from_utf8_lossy(&output.stdout).trim()
                        );
                        checked_set.insert(command.to_string());
                    }

                    let mut cmd = Command::new(bin_path);
                    hide_cmd(&mut cmd);
                    return Ok(cmd);
                }
            }

            println!(
                "[MediaMagnet][Download] Backend: Local {} [INVALID]. Continuing...",
                bin_path.display()
            );
        }
    }

    match app.shell().sidecar(command) {
        Ok(sidecar) => {
            let mut checked_set = version.lock().map_err(|_| "Lock failed")?;
            if !checked_set.contains(command) {
                println!("\n[MediaMagnet][Download] Backend: Prebuilt {}", command);
                println!("  -> This is usually out-of-date, please download the latest version and put in PATH!");
                checked_set.insert(command.to_string());
            }

            let std_cmd: std::process::Command = sidecar.into();
            let mut cmd: Command = std_cmd.into();
            hide_cmd(&mut cmd);
            Ok(cmd)
        }
        Err(e) => Err(format!(
            "{} is not installed and no sidecar available: {}",
            command, e
        )
        .into()),
    }
}

fn strip_ansi_codes(s: &str) -> String {
    static RE: std::sync::LazyLock<regex::Regex> =
        std::sync::LazyLock::new(|| regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap());
    RE.replace_all(s, "").to_string()
}

fn split_args(args_str: &str) -> std::result::Result<Vec<String>, String> {
    shell_words::split(args_str).map_err(|_| "Unable to split the arguments!".to_string())
}

fn apply_args(
    cmd: &mut Command,
    url: &str,
    global_args: &str,
    site_args: &[super::settings::SiteArguments],
) -> std::result::Result<(), String> {
    if !global_args.trim().is_empty() {
        let tokens = split_args(global_args)
            .map_err(|e| format!("Configuration Error (global arguments): {}", e))?;
        cmd.args(tokens);
    }

    for item in site_args {
        if !item.domain.trim().is_empty()
            && url.to_lowercase().contains(&item.domain.to_lowercase())
        {
            if !item.args.trim().is_empty() {
                let tokens = split_args(&item.args).map_err(|e| {
                    format!(
                        "Configuration Error (site [{}] arguments): {}",
                        item.domain, e
                    )
                })?;
                cmd.args(tokens);
                println!(
                    "[MediaMagnet] Applied custom arguments for domain [{}]: {}",
                    item.domain, item.args
                );
            }
        }
    }

    Ok(())
}

async fn apply_download_args(
    app: &tauri::AppHandle,
    url: &str,
    backend: Backend,
    settings: &Settings,
) -> std::result::Result<Command, String> {
    let mut cmd = base_command(app, backend.as_str())
        .await
        .map_err(|e| format!("{} backend error: {}", backend.as_str(), e))?;

    let lc = url.to_lowercase();

    match backend {
        Backend::SpotDL => {
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

            apply_args(&mut cmd, url, &settings.spotdl_global_args, &Vec::new())?;
        }
        Backend::YtDlp => {
            cmd.args([
                "-o",
                &settings.yt_output_template,
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

            apply_args(
                &mut cmd,
                url,
                &settings.yt_global_args,
                &settings.yt_site_args,
            )?;

            if settings.user_agent != "None" {
                cmd.args(["--user-agent", &settings.user_agent]);
            }
        }
        Backend::GalleryDl => {
            cmd.args(["-d", "."]);
            apply_args(
                &mut cmd,
                url,
                &settings.gdl_global_args,
                &settings.gdl_site_args,
            )?;
            if settings.user_agent != "None" {
                cmd.args(["-a", &settings.user_agent]);
            }
        }
    }

    Ok(cmd)
}

async fn download_url(
    app: tauri::AppHandle,
    mut cmd: Command,
    link: &str,
    backend: Backend,
    id: String,
) -> Result<()> {
    let app_stdout = app.clone();
    let app_stderr = app.clone();
    let id_out = id.clone();
    let id_err = id.clone();

    let base_download_dir = set_download_path(app.clone()).await;

    cmd.current_dir(&base_download_dir);

    let total_urls = if backend == Backend::GalleryDl {
        println!("\n[MediaMagnet][gallery-dl] Counting total urls...");
        let mut count_cmd = base_command(&app, "gallery-dl").await?;
        let _ = apply_cookies(&mut count_cmd, &app, link, backend);

        let out = count_cmd.args(["-g", link]).output().await?;
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('|'))
            .count()
    } else {
        0
    };

    let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    get_downloads().lock().unwrap().insert(
        id.clone(),
        ActiveDownload {
            child,
            backend: backend.clone(),
        },
    );

    let mut out_reader = BufReader::new(stdout).lines();
    let mut err_reader = BufReader::new(stderr).lines();

    let mut downloaded = 0usize;

    if backend == Backend::GalleryDl {
        println!(
            "[MediaMagnet][Download] Found {} total items to download",
            total_urls
        );
    }

    let out_handle = tokio::spawn(async move {
        if backend == Backend::SpotDL {
            let mut total_songs: i32 = 1;

            while let Ok(Some(line)) = out_reader.next_line().await {
                println!("[spotdl] {}", line);

                // "Found 8 songs in [Date] 04 - 2026 (Playlist)"
                if let Some(rest) = line.strip_prefix("Found ") {
                    if let Some(count_str) = rest.split_whitespace().next() {
                        if let Ok(n) = count_str.parse::<i32>() {
                            total_songs = n;
                        }
                    }
                    continue;
                }

                // "3/8 complete"
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

                            let _ = app_stdout.emit(
                                "download-progress",
                                ProgressPayload {
                                    id: id_out.clone(),
                                    value: pct,
                                },
                            );

                            let _ = app_stdout.emit(
                                "download-status",
                                StatusPayload {
                                    id: id_out.clone(),
                                    value: format!("{}/{} complete", done as u64, total as u64),
                                },
                            );
                            continue;
                        }
                    }
                }

                if !line.contains("other audio") || !line.contains("http") {
                    let _ = app_stdout.emit(
                        "download-status",
                        StatusPayload {
                            id: id_out.clone(),
                            value: line.clone(),
                        },
                    );
                }
            }

            let _ = app_stdout.emit(
                "download-progress",
                ProgressPayload {
                    id: id_out.clone(),
                    value: 100.0,
                },
            );
        } else {
            while let Ok(Some(line)) = out_reader.next_line().await {
                if backend == Backend::YtDlp {
                    if !line.contains('{') {
                        println!("{:#}", line);
                    }
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                        if let (Some("downloading"), Some(percent)) = (
                            json.get("status").and_then(|s| s.as_str()),
                            json.get("_percent").and_then(|p| p.as_f64()),
                        ) {
                            let _ = app_stdout.emit(
                                "download-progress",
                                ProgressPayload {
                                    id: id_out.clone(),
                                    value: percent,
                                },
                            );

                            let mut msg = String::new();
                            if let Some(p) = json.get("_percent_str").and_then(|p| p.as_str()) {
                                msg.push_str(&format!("Downloading: {}", strip_ansi_codes(p)));
                            }
                            if let Some(s) = json.get("_speed_str").and_then(|s| s.as_str()) {
                                msg.push_str(&format!(" Speed: {}", strip_ansi_codes(s)));
                            }
                            if let Some(e) = json.get("_eta_str").and_then(|e| e.as_str()) {
                                msg.push_str(&format!(" ETA: {}", strip_ansi_codes(e)));
                            }

                            let _ = app_stdout.emit(
                                "download-status",
                                StatusPayload {
                                    id: id_out.clone(),
                                    value: msg,
                                },
                            );
                        } else if json.get("status").and_then(|s| s.as_str()) == Some("finished") {
                            let _ = app_stdout.emit(
                                "download-progress",
                                ProgressPayload {
                                    id: id_out.clone(),
                                    value: 100.0,
                                },
                            );
                            let _ = app_stdout.emit(
                                "download-status",
                                StatusPayload {
                                    id: id_out.clone(),
                                    value: "Download finished".into(),
                                },
                            );
                        }
                    } else {
                        let _ = app_stdout.emit(
                            "download-status",
                            StatusPayload {
                                id: id_out.clone(),
                                value: line,
                            },
                        );
                    }
                } else if backend == Backend::GalleryDl {
                    let _ = app_stdout.emit(
                        "download-status",
                        StatusPayload {
                            id: id_out.clone(),
                            value: line.clone(),
                        },
                    );

                    if !line.trim_start().starts_with('#') && !line.trim().is_empty() {
                        downloaded += 1;
                        let pct = if total_urls > 0 {
                            (downloaded as f64 / total_urls as f64) * 100.0
                        } else {
                            0.0
                        };
                        let _ = app_stdout.emit(
                            "download-progress",
                            ProgressPayload {
                                id: id_out.clone(),
                                value: pct.min(100.0),
                            },
                        );
                    }
                } else {
                    println!("{:#}", line);
                    let _ = app_stdout.emit(
                        "download-status",
                        StatusPayload {
                            id: id_out.clone(),
                            value: line,
                        },
                    );
                }
            }
        }
    });

    let err_handle = tokio::spawn(async move {
        while let Ok(Some(line)) = err_reader.next_line().await {
            println!("{:#}", line);
            let line = line.to_lowercase();
            let event = if line.contains("[error") || line.contains("error:") {
                "download-error"
            } else if !(backend == Backend::YtDlp)
                || line.contains("downloaded")
                || line.contains("merged")
            {
                "download-status"
            } else {
                "notification"
            };
            let _ = app_stderr.emit(
                event,
                StatusPayload {
                    id: id_err.clone(),
                    value: line,
                },
            );
        }
    });

    let mut exit_status: Option<std::process::ExitStatus> = None;

    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let mut guard = get_downloads().lock().map_err(|_| "Lock failed")?;

        if !guard.contains_key(&id) {
            break;
        }

        let done = match guard.get_mut(&id).unwrap().child.try_wait() {
            Ok(Some(status)) => {
                exit_status = Some(status);
                true
            }
            Err(_) => true,
            Ok(None) => false,
        };

        if done {
            guard.remove(&id);
            break;
        }
    }

    let _ = tokio::join!(out_handle, err_handle);

    if backend == Backend::GalleryDl {
        // === create folder from directlink domain name ===
        let dl_path = base_download_dir.join("directlink");
        if dl_path.exists() && dl_path.is_dir() {
            for entry in std::fs::read_dir(&dl_path)? {
                let path = entry?.path();
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.to_lowercase().contains("part") {
                            continue;
                        }
                        let domain_dir =
                            base_download_dir.join(name.split('_').next().unwrap_or("unknown"));
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
    }

    if let Some(status) = exit_status {
        if !status.success() {
            return Err(format!(
                "{} process exited with error ({})",
                backend.as_str(),
                status
            )
            .into());
        }
    }

    Ok(())
}

async fn clean_tempfiles(app: &tauri::AppHandle, url: &str, backend: Backend) -> Result<()> {
    let base = set_download_path(app.clone()).await;

    match backend {
        Backend::SpotDL => {
            return Ok(()); // no temp files in download path
        }
        Backend::YtDlp => {
            let settings = Settings::load(app);
            let mut cmd = base_command(app, "yt-dlp").await?;
            cmd.args([
                "-o",
                &settings.yt_output_template,
                "--print",
                "filename",
                "--no-download",
                url,
            ])
            .current_dir(&base);

            let out = cmd.output().await?;
            for line in String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                let target = base.join(Path::new(line).file_name().unwrap_or_default());
                for suffix in ["part", "ytdl"] {
                    let path = match target.extension() {
                        Some(ext) => {
                            target.with_extension(format!("{}.{}", ext.to_string_lossy(), suffix))
                        }
                        None => target.with_extension(suffix),
                    };
                    if path.is_file() {
                        let _ = std::fs::remove_file(&path);
                    }
                }
            }
        }
        Backend::GalleryDl => {
            let mut cmd = base_command(app, "gallery-dl").await?;
            cmd.args(["-s", "-d", ".", "--print", "{_directory}", url])
                .current_dir(&base);

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
                    if deleted.insert(sub.clone()) && sub.starts_with(&base) && sub.is_dir() {
                        let _ = std::fs::remove_dir_all(&sub);
                    }
                }
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn downloader(app: tauri::AppHandle, url: String, download_id: String) {
    let lc = url.to_lowercase();
    let emit_status = |event: &str, msg: String| {
        let _ = app.emit(
            event,
            StatusPayload {
                id: download_id.clone(),
                value: msg,
            },
        );
    };

    if lc.contains("didg") && lc.contains("ls.i") && !lc.contains("original") {
        emit_status("download-status", "Skipped: Not original version".into());
        let _ = app.emit("download-finished", IdPayload { id: download_id });
        return;
    }

    if !lc.starts_with("http://") && !lc.starts_with("https://") {
        emit_status("download-error", "Invalid URL".into());
        return;
    }

    let _permit = match get_semaphore().acquire().await {
        Ok(p) => p,
        Err(_) => {
            emit_status("download-error", "Failed to acquire download permit".into());
            return;
        }
    };
    emit_status("download-status", "Starting...".into());

    let backend = Backend::identify(&app, &url).await;
    let settings = Settings::load(&app);

    let _ = app.emit(
        "download-started",
        IdPayload {
            id: download_id.clone(),
        },
    );

    let mut cmd = match apply_download_args(&app, &url, backend, &settings).await {
        Ok(c) => c,
        Err(e) => {
            emit_status("download-error", e);
            return;
        }
    };

    let cookies_applied = apply_cookies(&mut cmd, &app, &url, backend).unwrap_or(false);
    cmd.arg(&url);

    if let Err(e) = download_url(app.clone(), cmd, &url, backend, download_id.clone()).await {
        if cookies_applied {
            println!(
                "[MediaMagnet] Download failed ({}), retrying without cookies...",
                download_id
            );
            emit_status(
                "download-status",
                "FAILED: Retrying without cookies...".into(),
            );

            match apply_download_args(&app, &url, backend, &settings).await {
                Ok(mut retry_cmd) => {
                    retry_cmd.arg(&url);
                    if let Err(e) =
                        download_url(app.clone(), retry_cmd, &url, backend, download_id.clone())
                            .await
                    {
                        println!(
                            "[MediaMagnet] Download failed without cookies ({}): {}",
                            download_id, e
                        );
                        emit_status("download-error", format!("Download failed: {}", e));
                    }
                }
                Err(e) => {
                    emit_status("download-error", e);
                }
            }
        } else {
            println!("[MediaMagnet] Download failed ({}): {}", download_id, e);
            emit_status("download-error", format!("Download failed: {}", e));
        }
    }

    let _ = app.emit("download-finished", IdPayload { id: download_id });
}

#[tauri::command]
pub async fn pause_download(
    app: tauri::AppHandle,
    download_id: String,
) -> std::result::Result<(), String> {
    let entry = {
        get_downloads()
            .lock()
            .map_err(|_| "Lock failed")?
            .remove(&download_id)
    };

    if let Some(mut entry) = entry {
        entry
            .child
            .kill()
            .await
            .map_err(|e| format!("Kill failed: {}", e))?;
        entry
            .child
            .wait()
            .await
            .map_err(|e| format!("Wait failed: {}", e))?;

        let _ = app.emit(
            "download-status",
            StatusPayload {
                id: download_id,
                value: "Paused".into(),
            },
        );
        Ok(())
    } else {
        Err(format!("No active download with id: {}", download_id))
    }
}

#[tauri::command]
pub async fn cancel_download(
    app: tauri::AppHandle,
    download_id: String,
    url: String,
) -> std::result::Result<(), String> {
    let entry = {
        get_downloads()
            .lock()
            .map_err(|_| "Lock failed")?
            .remove(&download_id)
    };

    if let Some(mut entry) = entry {
        let _ = entry.child.kill().await;
        let _ = entry.child.wait().await;

        if let Err(e) = clean_tempfiles(&app, &url, entry.backend).await {
            println!("[MediaMagnet] Cleanup failed during cancellation: {}", e);
        }
    }

    let _ = app.emit(
        "download-error",
        StatusPayload {
            id: download_id,
            value: "Download cancelled and files permanently deleted".into(),
        },
    );
    Ok(())
}

#[tauri::command]
pub async fn cancel_all_downloads(app: tauri::AppHandle) -> std::result::Result<(), String> {
    let children: Vec<(String, ActiveDownload)> = {
        get_downloads()
            .lock()
            .map_err(|_| "Lock failed")?
            .drain()
            .collect()
    };

    for (id, mut entry) in children {
        if let Err(e) = entry.child.kill().await {
            eprintln!("[MediaMagnet] Failed to kill {}: {}", id, e);
        }
        let _ = entry.child.wait().await;

        let _ = app.emit(
            "download-error",
            StatusPayload {
                id,
                value: "Download cancelled".into(),
            },
        );
    }

    get_semaphore().close();
    Ok(())
}
