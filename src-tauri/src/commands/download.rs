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

fn get_downloads() -> &'static Arc<Mutex<HashMap<String, tokio::process::Child>>> {
    static DOWNLOADS: OnceLock<Arc<Mutex<HashMap<String, tokio::process::Child>>>> =
        OnceLock::new();
    DOWNLOADS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

#[inline]
fn configure_cmd_flags(_cmd: &mut Command) {
    #[cfg(target_os = "windows")]
    _cmd.creation_flags(0x08000000);
}

fn apply_cookies(cmd: &mut Command, app: &tauri::AppHandle, link: &str) -> Result<()> {
    let cookies_dir = app.path().app_data_dir().unwrap().join("cookies");
    if !cookies_dir.exists() {
        println!("[MediaMagnet][Download] No cookies directory found, ignoring...");
        return Ok(());
    }

    if let Ok(entries) = std::fs::read_dir(&cookies_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if link.to_lowercase().contains(&file_stem.to_lowercase()) {
                        if let Some(path_str) = path.to_str() {
                            cmd.args(["--cookies", path_str]);
                            println!("[MediaMagnet] Applied cookies from: {}", path_str);
                            return Ok(());
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

async fn base_command(app: &tauri::AppHandle, command: &str) -> Result<Command> {
    static VERSION: OnceLock<Arc<Mutex<HashSet<String>>>> = OnceLock::new();
    let version = VERSION.get_or_init(|| Arc::new(Mutex::new(HashSet::new())));

    let mut check_cmd = Command::new(command);
    configure_cmd_flags(&mut check_cmd);

    let check_result = check_cmd
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await;

    match check_result {
        Ok(output) if output.status.success() => {
            let mut checked_set = version.lock().map_err(|_| "Lock failed")?;
            if !checked_set.contains(command) {
                println!(
                    "[MediaMagnet][Download] Backend: Local {}, Version: {}",
                    command,
                    String::from_utf8_lossy(&output.stdout).trim()
                );
                checked_set.insert(command.to_string());
            }

            let mut cmd = Command::new(command);
            configure_cmd_flags(&mut cmd);
            Ok(cmd)
        }
        _ => match app.shell().sidecar(command) {
            Ok(sidecar) => {
                let mut checked_set = version.lock().map_err(|_| "Lock failed")?;
                if !checked_set.contains(command) {
                    println!("\n[MediaMagnet][Download] Backend: Prebuilt {}", command);
                    println!("  -> This is usually out-of-date, please download the latest version and put in PATH!");
                    checked_set.insert(command.to_string());
                }

                let std_cmd: std::process::Command = sidecar.into();
                let mut cmd: Command = std_cmd.into();
                configure_cmd_flags(&mut cmd);
                Ok(cmd)
            }
            Err(e) => Err(format!(
                "{} is not installed and no sidecar available: {}",
                command, e
            )
            .into()),
        },
    }
}

async fn is_youtube(app: &tauri::AppHandle, url: &str) -> bool {
    if let Ok(mut check_cmd) = base_command(app, "yt-dlp").await {
        check_cmd
            .args(["--print", "title", "--no-download", url])
            .stderr(Stdio::piped());
        if let Ok(output) = check_cmd.output().await {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return output.status.success() && !stderr.contains("Falling back on generic");
        }
    }
    false
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

async fn run_downloader(
    app: tauri::AppHandle,
    mut cmd: Command,
    link: &str,
    ytdlp: bool,
    id: String,
) -> Result<()> {
    let app_stdout = app.clone();
    let app_stderr = app.clone();
    let id_out = id.clone();
    let id_err = id.clone();

    let base_download_dir = set_download_path(app.clone()).await;

    let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let total_urls = if !ytdlp {
        println!("\n[MediaMagnet][gallery-dl] Counting total urls...");
        let mut count_cmd = base_command(&app, "gallery-dl").await?;
        let _ = apply_cookies(&mut count_cmd, &app, link);

        let out = count_cmd.args(["-g", link]).output().await?;
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('|'))
            .count()
    } else {
        0
    };

    get_downloads().lock().unwrap().insert(id.clone(), child);

    let mut out_reader = BufReader::new(stdout).lines();
    let mut err_reader = BufReader::new(stderr).lines();

    let mut downloaded = 0usize;

    if !ytdlp {
        println!(
            "[MediaMagnet][Download] Found {} total items to download",
            total_urls
        );
    }

    let out_handle = tokio::spawn(async move {
        while let Ok(Some(line)) = out_reader.next_line().await {
            if ytdlp {
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
            } else {
                let _ = app_stdout.emit(
                    "download-status",
                    StatusPayload {
                        id: id_out.clone(),
                        value: line,
                    },
                );
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
                        value: pct,
                    },
                );
            }
        }
    });

    let err_handle = tokio::spawn(async move {
        while let Ok(Some(line)) = err_reader.next_line().await {
            println!("{:#}", line);
            let event = if line.contains("[error") {
                "download-error"
            } else if !ytdlp
                || line.to_lowercase().contains("downloaded")
                || line.to_lowercase().contains("merged")
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

    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let mut guard = get_downloads().lock().map_err(|_| "Lock failed")?;

        if !guard.contains_key(&id) {
            break;
        }

        let done = match guard.get_mut(&id).unwrap().try_wait() {
            Ok(Some(_)) | Err(_) => true,
            Ok(None) => false,
        };

        if done {
            guard.remove(&id);
            break;
        }
    }

    let _ = tokio::join!(out_handle, err_handle);

    if !ytdlp {
        // === directlink file cleanup ===
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

    Ok(())
}

async fn clean_downloaded(app: &tauri::AppHandle, url: &str, is_youtube: bool) -> Result<()> {
    let base = set_download_path(app.clone()).await;

    if is_youtube {
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
    } else {
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

    if !lc.contains("http") {
        emit_status("download-error", "Invalid URL".into());
        return;
    }

    let is_youtube = is_youtube(&app, &url).await;
    let _ = app.emit(
        "download-started",
        IdPayload {
            id: download_id.clone(),
        },
    );

    let settings = Settings::load(&app);
    let mut cmd = match base_command(&app, if is_youtube { "yt-dlp" } else { "gallery-dl" }).await {
        Ok(c) => c,
        Err(_) => return,
    };

    if is_youtube {
        cmd.args([
            "-o",
            &settings.yt_output_template,
            "--progress-template",
            "%(progress)j",
            "--newline",
        ]);

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

        if let Err(e) = apply_args(
            &mut cmd,
            &url,
            &settings.yt_global_args,
            &settings.yt_site_args,
        ) {
            emit_status("download-error", e);
            let _ = app.emit("download-finished", IdPayload { id: download_id });
            return;
        }
        if settings.user_agent != "None" {
            cmd.args(["--user-agent", &settings.user_agent]);
        }
    } else {
        cmd.args(["-d", "."]);
        if let Err(e) = apply_args(
            &mut cmd,
            &url,
            &settings.gdl_global_args,
            &settings.gdl_site_args,
        ) {
            emit_status("download-error", e);
            let _ = app.emit("download-finished", IdPayload { id: download_id });
            return;
        }
        if settings.user_agent != "None" {
            cmd.args(["-a", &settings.user_agent]);
        }
    }

    let _ = apply_cookies(&mut cmd, &app, &url);
    cmd.arg(&url);

    if let Err(e) = run_downloader(app.clone(), cmd, &url, is_youtube, download_id.clone()).await {
        println!("[MediaMagnet] Download failed ({}): {}", download_id, e);
        emit_status("download-error", format!("Download failed: {}", e));
    }

    let _ = app.emit("download-finished", IdPayload { id: download_id });
}

#[tauri::command]
pub async fn pause_download(
    app: tauri::AppHandle,
    download_id: String,
) -> std::result::Result<(), String> {
    let child = {
        get_downloads()
            .lock()
            .map_err(|_| "Lock failed")?
            .remove(&download_id)
    };

    if let Some(mut child) = child {
        child
            .kill()
            .await
            .map_err(|e| format!("Kill failed: {}", e))?;
        child
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
    let child = {
        get_downloads()
            .lock()
            .map_err(|_| "Lock failed")?
            .remove(&download_id)
    };

    if let Some(mut child) = child {
        let _ = child.kill().await;
        let _ = child.wait().await;
    }

    let is_youtube = is_youtube(&app, &url).await;
    if let Err(e) = clean_downloaded(&app, &url, is_youtube).await {
        println!("[MediaMagnet] Cleanup failed during cancellation: {}", e);
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
    let children: Vec<(String, tokio::process::Child)> = {
        get_downloads()
            .lock()
            .map_err(|_| "Lock failed")?
            .drain()
            .collect()
    };

    for (id, mut child) in children {
        if let Err(e) = child.kill().await {
            eprintln!("[MediaMagnet] Failed to kill {}: {}", id, e);
        }
        let _ = child.wait().await;

        let _ = app.emit(
            "download-error",
            StatusPayload {
                id: id,
                value: "Download cancelled".into(),
            },
        );
    }
    Ok(())
}
