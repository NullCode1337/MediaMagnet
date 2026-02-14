use std::sync::{Arc, Mutex, OnceLock};
use tauri::{Emitter, Manager};
use tauri_plugin_shell::ShellExt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use super::settings::Settings;
use super::utils::set_download_path;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn get_current_download() -> &'static Arc<Mutex<Option<tokio::process::Child>>> { // no idea
    static CURRENT_DOWNLOAD: OnceLock<Arc<Mutex<Option<tokio::process::Child>>>> = OnceLock::new();
    CURRENT_DOWNLOAD.get_or_init(|| Arc::new(Mutex::new(None)))
}

// Helper download functions
async fn load_settings(app: &tauri::AppHandle) -> Result<Settings> {
    let settings_path = app.path().app_config_dir().unwrap().join("settings.json");
    let settings: Settings =
        serde_json::from_str(&std::fs::read_to_string(&settings_path).unwrap()).unwrap();
    Ok(settings)
}

async fn base_command(app: &tauri::AppHandle, command: &str) -> Result<Command> {
    let mut cmd = Command::new(command);
    #[cfg(target_os = "windows")]
    cmd.creation_flags(winapi::um::winbase::CREATE_NO_WINDOW);

    if cmd.arg("--version").output().await.is_err() {
        if let Ok(sidecar_cmd) = app.shell().sidecar(command) {
            let std_cmd: std::process::Command = sidecar_cmd.into();
            return Ok(std_cmd.into())
        }   
        return Err(format!("{} is not installed or available in PATH", command).into());
    }

    Ok(cmd)
}

fn apply_cookies(cmd: &mut Command, app: &tauri::AppHandle, link: &str) -> Result<()> {
    let cookies_dir: std::path::PathBuf = app.path().app_data_dir().unwrap().join("cookies");

    if let Ok(entries) = std::fs::read_dir(&cookies_dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if link.to_lowercase().contains(&name.to_lowercase()) {
                        cmd.args(["-C", path.to_str().unwrap()]);
                        break;
                    }
                }
            }
        }
    }
    Ok(())
}

async fn run_downloader(app: tauri::AppHandle, mut cmd: Command, link: &str, ytdlp: bool) -> Result<()> {
    let app_stdout = app.clone();
    let app_stderr = app.clone();
    set_download_path(app.clone()).await;
    
    let mut child = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    
    {
        let mut guard = get_current_download().lock().unwrap();
        *guard = Some(child);
    }
    
    let mut out_reader = BufReader::new(stdout).lines();
    let mut err_reader = BufReader::new(stderr).lines();

    let total_urls = if !ytdlp {
        let out = base_command(&app, "gallery-dl").await?.args(["-g", link]).output().await?;
        String::from_utf8_lossy(&out.stdout).lines().filter(|l| !l.trim().starts_with('|')).count()
    } else { 0 };

    let mut downloaded = Vec::new();
    
    let out_handle = tokio::spawn(async move {
        while let Ok(Some(line)) = out_reader.next_line().await {
            if ytdlp {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                    if let (Some("downloading"), Some(percent)) = (
                        json.get("status").and_then(|s| s.as_str()),
                        json.get("_percent").and_then(|p| p.as_f64()),
                    ) {
                        let _ = app_stdout.emit("download-progress", percent);

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

                        let _ = app_stdout.emit("download-status", msg);
                    } else if json.get("status").and_then(|s| s.as_str()) == Some("finished") {
                        let _ = app_stdout.emit("download-progress", 100.0);
                        let _ = app_stdout.emit("download-status", "Download finished");
                    }
                } else {
                    let _ = app_stdout.emit("download-status", &line);
                }
            } else {
                let _ = app_stdout.emit("download-status", &line);
                downloaded.push(line);
                let _ = app_stdout.emit("download-progress", 
                    (downloaded.len() as f64 / total_urls as f64) * 100.0);
            }
        }
    });

    let err_handle = tokio::spawn(async move {
        while let Ok(Some(line)) = err_reader.next_line().await {
            println!("{:#}", line);
            let (event, msg) = if line.contains("[error") || line.to_lowercase().contains("error") {
                ("download-error", line.clone())
            } else if !ytdlp || line.to_lowercase().contains("downloaded") || line.to_lowercase().contains("merged") {
                ("download-status", line.clone())
            } else {
                ("notification", line.clone())
            };
            let _ = app_stderr.emit(event, msg);
        }
    });

    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let mut guard = get_current_download().lock().map_err(|_| "Lock failed")?;
        if guard.is_none() { break; }
        
        if let Some(child) = guard.as_mut() {
            match child.try_wait() {
                Ok(Some(_)) | Err(_) => { *guard = None; break; }
                Ok(None) => {}
            }
        }
    }

    let _ = tokio::join!(out_handle, err_handle);
    
    if !ytdlp { // === directlink file cleanup ===
        let dl_path = std::path::Path::new("directlink");
        if dl_path.exists() && dl_path.is_dir() {
            for entry in std::fs::read_dir(dl_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.to_lowercase().contains("part") { continue; }
                        let domain_dir = std::path::Path::new(".").join(name.split('_').next().unwrap_or("unknown"));
                        if !domain_dir.exists() { std::fs::create_dir(&domain_dir)?; }
                        std::fs::rename(&path, domain_dir.join(name))?;
                    }
                }
            }
            if dl_path.read_dir()?.next().is_none() { std::fs::remove_dir(dl_path)?; }
        }
    }

    Ok(())
}

fn strip_ansi_codes(s: &str) -> String {
    regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap().replace_all(s, "").to_string()
}

#[tauri::command]
pub async fn downloader(app: tauri::AppHandle, url: String) {
    let lc = url.to_lowercase();

    if !lc.contains("http") {
        let _ = app.emit("download-error", "Invalid URL");
        return;
    }

    let is_youtube = lc.contains("youtube") || lc.contains("youtu.be") || lc.contains("music.youtube") ||
        lc.contains("twitch") || lc.contains("vimeo") || lc.contains("soundcloud") ||
        lc.contains("bandcamp") || lc.contains("twitter") || lc.contains("x.com") ||
        lc.contains("instagram") || lc.contains("facebook") || lc.contains("tiktok");

    let _ = app.emit("download-started", ());
    
    let settings = load_settings(&app).await.unwrap();
    let mut cmd = base_command(&app, if is_youtube { "yt-dlp" } else { "gallery-dl" }).await.unwrap();
    
    if is_youtube { // TODO: make this customizable
        cmd.args(["-o", "%(title)s.%(ext)s", "-f", "244", "--progress-template", "%(progress)j"]);
        if settings.user_agent != "None" { cmd.args(["--user-agent", &settings.user_agent]); }
    } else {
        cmd.args(["-d", "."]);
        if settings.user_agent != "None" { cmd.args(["-a", &settings.user_agent]); }
    }
    
    apply_cookies(&mut cmd, &app, &url).unwrap();
    cmd.args([&url]);

    let result = run_downloader(app.clone(), cmd, &url, is_youtube).await;
    
    if let Err(e) = result {
        println!("[MediaMagnet] Download failed: {}", e);
        let _ = app.emit("download-error", format!("Download failed: {}", e));
    }
    
    let _ = app.emit("download-finished", ());
}

#[tauri::command]
pub async fn cancel_download() -> std::result::Result<(), String> {
    let child = get_current_download().lock().map_err(|_| "Lock failed")?.take();
    
    if let Some(mut child) = child {
        child.kill().await.map_err(|e| format!("Kill failed: {}", e))?;
        child.wait().await.map_err(|e| format!("Wait failed: {}", e))?;
        Ok(())
    } else {
        Err("No download in progress".to_string())
    }
}