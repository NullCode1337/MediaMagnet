use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri::Emitter;
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};

// TODO: backend and frontend for user-agent, cookies and oauth
async fn async_dl(app: tauri::AppHandle, link: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut downloaded: Vec<String> = Vec::new();

    let url_list = Command::new("gallery-dl")
      .args(["-g", link])
      .output()
      .await?;
    
    let total_urls: usize = String::from_utf8_lossy(&url_list.stdout)
        .lines()
        .filter(|line| !line.trim_start().starts_with('|'))
        .count();

    if let Some(downloads_path) = dirs::download_dir() {
      std::env::set_current_dir(&downloads_path)?;
    } else {
      eprintln!("Could not find Downloads directory");
    }

    let mut downloader = Command::new("gallery-dl")
      .arg(link)
      .stdout(std::process::Stdio::piped())
      .stderr(std::process::Stdio::piped())
      .spawn()?;
    
    let stdout = downloader.stdout.take().unwrap();
    let stderr = downloader.stderr.take().unwrap();
    
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();
    
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
        app_stderr.emit("notification", &line).unwrap();
        if line.contains("error") {
          app_stderr.emit("download-error", &line).unwrap();
        }
      }
    });
        
    let _status = downloader.wait().await?;
    Ok(())
}

#[tauri::command]
pub async fn gallery_dl(app: tauri::AppHandle, mut url: String) {
  if url == "" {
    let result = app.clipboard()
      .read_text()
      .map_err(|e| e.to_string());

    match result {
      Ok(content) => {
        url = content;
      }
      Err(err) => {
        app.emit("notification", err).unwrap();
        return;
      }
    }
  }

  if !url.to_lowercase().contains("http") {
    app.emit("notification", "Not a valid url").unwrap();
    return;
  }

  // Start download
  app.emit("download-started", ()).unwrap();
  let _ = async_dl(app.clone(), &url).await;
  app.emit("download-finished", ()).unwrap();
}