use tauri_plugin_clipboard_manager::ClipboardExt;

use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use dirs::download_dir;

async fn async_dl(link: &str) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: backend and frontend for user-agent, cookies and oauth
    if let Some(downloads_path) = download_dir() {
      std::env::set_current_dir(&downloads_path)?;
    } else {
      eprintln!("Could not find Downloads directory");
    }

    let mut child = Command::new("gallery-dl")
        .arg(link)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;
    
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();
    
    tokio::spawn(async move {
        while let Ok(Some(line)) = stdout_reader.next_line().await {
            println!("STDOUT: {}", line);
        }
    });
    
    tokio::spawn(async move {
        // TODO: Implement frontend for "[gallery-dl][error] Unsupported URL"
        // TODO: Emit global event to deal with this in the frontend
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            eprintln!("STDERR: {}", line);
        }
    });
    
    let status = child.wait().await?;
    
    if status.success() {
        Ok(())
    } else {
        Err("gallery-dl failed".into())
    }
}

#[tauri::command]
async fn download(app: tauri::AppHandle, mut url: String) {
  if url == "" {
    match app.clipboard().read_text() {
      Ok(content) => {
        url = content;
      }
      Err(e) => {
        println!("Clipboard error: {}", e); // TODO: Implement frontend
        return;
      }
    }
  }
  if !url.to_lowercase().contains("http") {
    println!("Not a url"); // TODO: Implement frontend
    return;
  }

  // Start download
  let _ = async_dl(&url).await;
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![download])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
