use std::sync::{Arc, Mutex, OnceLock};
use tauri::{Emitter, Manager};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use super::settings::Settings;
use super::utils::set_download_path;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn get_current_download() -> &'static Arc<Mutex<Option<tokio::process::Child>>> {
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

fn base_command(command: &str) -> Command {
    let mut cmd = Command::new(command);
    #[cfg(target_os = "windows")]
    cmd.creation_flags(winapi::um::winbase::CREATE_NO_WINDOW);
    cmd
}

fn apply_cookies(cmd: &mut Command, app: &tauri::AppHandle, link: &str) -> Result<()> {
    let cookies_dir = app.path().app_data_dir().unwrap().join("cookies");
    let link_lower = link.to_lowercase();

    if let Ok(entries) = std::fs::read_dir(&cookies_dir) {
        for entry in entries.flatten() {
            let file_path = entry.path();
            if file_path.is_file() {
                if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) {
                    let file_name_lower = file_name.to_lowercase();

                    if link_lower.contains(&file_name_lower) {
                        cmd.args(["-C", file_path.to_str().unwrap()]);
                        break; // Only use one cookie file
                    }
                }
            }
        }
    }
    Ok(())
}

async fn gallery_dl(app: tauri::AppHandle, link: &str) -> Result<()> {
    let mut downloaded: Vec<String> = Vec::new();

    let settings = load_settings(&app).await?;
    set_download_path(app.clone()).await;

    let version = base_command("gallery-dl").arg("--version").output().await?;
    println!("[MediaMagnet] gallery-dl version: {}", String::from_utf8_lossy(&version.stdout));

    // === Total urls in link ===
    let url_list = base_command("gallery-dl").args(["-g", link]).output().await?;

    let total_urls = String::from_utf8_lossy(&url_list.stdout)
        .lines()
        .filter(|line| !line.trim().starts_with('|'))
        .count();

    // === Downloader ===
    let mut cmd = base_command("gallery-dl");
    cmd.args(["-d", "."]);
    if settings.user_agent != "None" {
        cmd.args(["-a", &settings.user_agent]);
    }
    apply_cookies(&mut cmd, &app, link)?;
    cmd.args([link]);

    let mut downloader = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let stdout = downloader.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = downloader.stderr.take().ok_or("Failed to capture stderr")?;

    {
        let current_download = get_current_download();
        let mut download_guard = current_download.lock().unwrap();
        *download_guard = Some(downloader);
    }

    let (mut stdout_reader, mut stderr_reader) = (
        BufReader::new(stdout).lines(),
        BufReader::new(stderr).lines(),
    );

    let app_stdout = app.clone();
    let app_stderr = app.clone();

    //stdout
    let stdout_handle = tokio::spawn(async move {
        while let Ok(Some(line)) = stdout_reader.next_line().await {
            let _ = app_stdout.emit("download-status", &line);
            downloaded.push(line);
            let progress = (downloaded.len() as f64 / total_urls as f64) * 100.0;
            let _ = app_stdout.emit("download-progress", progress);
        }
    });

    let stderr_handle = tokio::spawn(async move {
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            println!("{:#}", &line);
            if line.contains("[error") {
                let _ = app_stderr.emit("download-error", &line);
            } else {
                let _ = app_stderr.emit("notification", &line);
            }
        }
    });

    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let current_download = get_current_download();
        let mut download_guard = current_download.lock().map_err(|_| "Lock on download handle")?;
        if download_guard.is_none() { break; } // Cancelled by user 

        if let Some(child) = download_guard.as_mut() {
            match child.try_wait() {
                Ok(Some(_)) => { 
                    *download_guard = None; 
                    break;
                }
                Err(e) => { 
                    println!("[MediaMagnet] Error checking child process status: {}", e);
                    *download_guard = None; 
                    break;
                }
                Ok(None) => {} 
            }
        }
    }

    let _ = tokio::join!(stdout_handle, stderr_handle);

    // === Cleanup ===
    let directlink = std::path::Path::new("directlink");
    if directlink.exists() && directlink.is_dir() {
        for entry in std::fs::read_dir(directlink)? {
            let entry = entry?;
            let file_path = entry.path();
            if file_path.is_file() {
                let file_name = file_path.file_name().unwrap();
                let new_path = std::path::Path::new(".").join(file_name);
                std::fs::rename(&file_path, new_path)?; 
            }
        }
        std::fs::remove_dir(directlink)?;
    }

    Ok(())
}

#[tauri::command]
pub async fn downloader(app: tauri::AppHandle, url: String) {
    if !url.to_lowercase().contains("http") {
        let _ = app.emit("download-error", "Invalid URL");
        return;
    }

    let _ = app.emit("download-started", ());
    
    if let Err(e) = gallery_dl(app.clone(), &url).await {
        println!("[MediaMagnet] Download failed: {}", e);
    }
    
    let _ = app.emit("download-finished", ());
}

#[tauri::command]
pub async fn cancel_download() -> std::result::Result<(), String> {
    let current_download = get_current_download();
    
    let child = {
        let mut download_guard = current_download.lock().map_err(|_| "Failed to acquire lock to cancel")?;
        download_guard.take()
    };
    
    if let Some(mut child) = child {
        if let Err(e) = child.kill().await { 
            return Err(format!("Failed to kill process: {}", e));
        }
        
        match child.wait().await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Error waiting for process: {}", e)),
        }
    } else {
        Err("No download in progress".to_string())
    }
}