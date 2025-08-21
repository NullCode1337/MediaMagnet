use tauri_plugin_clipboard_manager::ClipboardExt;

#[tauri::command]
fn download(app: tauri::AppHandle, mut url: String) {
  if url == "" {
    match app.clipboard().read_text() {
      Ok(content) => {
          url = content;
      }
      Err(e) => {
          println!("Clipboard error: {}", e);
          println!("Not a valid URL in clipboard");
          return;
      }
    }
  }
  if !url.to_lowercase().contains("http") {
    println!("Not a url")
  }
  else {
    println!("Download {}", url);
  }
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
