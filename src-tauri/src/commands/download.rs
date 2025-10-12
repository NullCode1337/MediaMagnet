use std::sync::{Arc, Mutex, OnceLock};
use tauri::{Emitter, Manager};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use super::settings::Settings;

fn get_current_download() -> &'static Arc<Mutex<Option<tokio::process::Child>>> {
    static CURRENT_DOWNLOAD: OnceLock<Arc<Mutex<Option<tokio::process::Child>>>> = OnceLock::new();
    CURRENT_DOWNLOAD.get_or_init(|| Arc::new(Mutex::new(None)))
}

async fn set_download_path(app: tauri::AppHandle) {
    let config_path = app.path().app_config_dir().unwrap().join("settings.json");
    let settings: Settings =
        serde_json::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();

    let default = dirs::download_dir().unwrap().join("MediaMagnet");

    let downloads_path = match settings.download_path.as_str() {
        "Default" => default.clone(),
        custom_path if custom_path.to_lowercase().contains("mediamagnet") => {
            std::path::PathBuf::from(custom_path)
        }
        path => std::path::PathBuf::from(path).join("MediaMagnet"),
    };

    let final_dir = if downloads_path.exists() {
        downloads_path
    } else {
        std::fs::create_dir_all(&downloads_path)
            .map(|_| downloads_path)
            .unwrap_or_else(|_e| {
                app.emit(
                    "notification",
                    "[MME] Failed to create directory, using default...",
                )
                .unwrap();
                std::fs::create_dir_all(&default).unwrap();
                default
            })
    };

    std::env::set_current_dir(&final_dir).unwrap();
}

async fn gallery_dl(app: tauri::AppHandle, link: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut downloaded: Vec<String> = Vec::new();
    let config_path = app.path().app_config_dir().unwrap().join("settings.json");
    let settings: Settings =
        serde_json::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();

    // === Total urls in link ===
    let mut url_cmd = Command::new("gallery-dl");
    url_cmd.args(["-g", link]);

    #[cfg(target_os = "windows")]
    url_cmd.creation_flags(winapi::um::winbase::CREATE_NO_WINDOW);

    let url_list = url_cmd.output().await?;

    let total_urls: usize = String::from_utf8_lossy(&url_list.stdout)
        .lines()
        .filter(|line| !line.trim_start().starts_with('|'))
        .count();

    // === Downloader ===
    set_download_path(app.clone()).await;

    let mut cmd = Command::new("gallery-dl");
    cmd.args(["-d", "."]);

    if settings.user_agent != "None" {
        cmd.args(["-a", &settings.user_agent]);
    }

    cmd.args([link]);

    #[cfg(target_os = "windows")]
    cmd.creation_flags(winapi::um::winbase::CREATE_NO_WINDOW);

    let mut downloader = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let stdout = downloader.stdout.take().unwrap();
    let stderr = downloader.stderr.take().unwrap();

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
            app_stdout.emit("download-status", &line).unwrap();
            downloaded.push(line);
            let progress = (downloaded.len() as f64 / total_urls as f64) * 100.0;
            app_stdout.emit("download-progress", progress).unwrap();
        }
    });

    //stderr
    let stderr_handle = tokio::spawn(async move {
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            println!("{:#}", &line);
            if line.contains("error") {
                app_stderr.emit("download-error", &line).unwrap();
            } else {
                app_stderr.emit("notification", &line).unwrap();
            }
        }
    });

    let mut completed = false;
    while !completed {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        let current_download = get_current_download();
        let mut download_guard = current_download.lock().unwrap();
        
        if download_guard.is_none() {
            completed = true;
        } else if let Some(child) = download_guard.as_mut() {
            match child.try_wait() {
                Ok(Some(_status)) => {
                    *download_guard = None;
                    completed = true;
                }
                Ok(None) => {}
                Err(_) => {
                    *download_guard = None;
                    completed = true;
                }
            }
        }
    }

    let _ = stdout_handle.await;
    let _ = stderr_handle.await;

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
        app.emit("download-error", "Invalid URL").unwrap();
        return;
    }

    // Start download
    app.emit("download-started", ()).unwrap();
    let _ = gallery_dl(app.clone(), &url).await;
    app.emit("download-finished", ()).unwrap();
}

#[tauri::command]
pub async fn cancel_download() -> Result<(), String> {
    let current_download = get_current_download();
    
    let child = {
        let mut download_guard = current_download.lock().unwrap();
        download_guard.take()
    };
    
    if let Some(mut child) = child {
        if let Err(e) = child.start_kill() {
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