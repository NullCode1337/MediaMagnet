use tauri::{Emitter, Manager};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use super::settings::Settings;

async fn set_download_path(app: tauri::AppHandle) {
    let config_path = app.path().app_config_dir().unwrap().join("settings.json");
    let settings: Settings = serde_json::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
    
    let default = dirs::download_dir().unwrap().join("MediaMagnet");
    
    let downloads_path = match settings.download_path.as_str() {
        "Default" => default.clone(),
        custom_path if custom_path.to_lowercase().contains("mediamagnet") => 
            std::path::PathBuf::from(custom_path),
        path => std::path::PathBuf::from(path).join("MediaMagnet"),
    };

    let final_dir = if downloads_path.exists() {
        downloads_path
    } else {
        std::fs::create_dir_all(&downloads_path)
            .map(|_| downloads_path)
            .unwrap_or_else(|_e| {
                app.emit("notification", "[MME] Failed to create directory, using default...").unwrap();
                std::fs::create_dir_all(&default).unwrap();
                default
            })
    };
    
    std::env::set_current_dir(&final_dir).unwrap();
}

async fn gallery_dl(app: tauri::AppHandle, link: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut downloaded: Vec<String> = Vec::new();
    let config_path = app.path().app_config_dir().unwrap().join("settings.json");
    let settings: Settings = serde_json::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();

    // === Total urls in link ===
    let mut url_cmd = Command::new("gallery-dl");
    url_cmd.args(["-g", link]);

    #[cfg(target_os = "windows")]
    url_cmd.creation_flags(winapi::um::winbase::CREATE_NO_WINDOW); 

    let url_list = url_cmd
        .output()
        .await?;

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

    let mut downloader = cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let (mut stdout_reader, mut stderr_reader) = (
        BufReader::new(downloader.stdout.take().unwrap()).lines(),
        BufReader::new(downloader.stderr.take().unwrap()).lines()
    );

    let app_stdout = app.clone();
    let app_stderr = app.clone();

    //stdout
    tokio::spawn(async move {
        while let Ok(Some(line)) = stdout_reader.next_line().await {
            app_stdout.emit("download-status", &line).unwrap();
            downloaded.push(line);
            let progress = (downloaded.len() as f64 / total_urls as f64) * 100.0;
            app_stdout.emit("download-progress", progress).unwrap();
        }
    });

    //stderr
    tokio::spawn(async move {
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            println!("{:#}", &line);
            if line.contains("error") {
                app_stderr.emit("download-error", &line).unwrap();
            } else {
                app_stderr.emit("notification", &line).unwrap();
            }
        }
    });

    let _status = downloader.wait().await?;

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