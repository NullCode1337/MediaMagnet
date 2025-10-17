use std::path::PathBuf;
use std::collections::HashMap;

use tauri::{Emitter, Manager};

use super::utils::is_netscape;
use super::utils::convert_json;

#[tauri::command]
pub fn add_cookie(app: tauri::AppHandle, domain: String, file_path: String) -> Result<String, String> {
    let cookies_dir = app.path().app_data_dir().unwrap().join("cookies");
    let file_path = PathBuf::from(file_path);
    
    if !file_path.exists() {
        return Err(format!("Source file does not exist: {}", file_path.display()));
    }
    
    let file_extension = file_path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    let dest_path = cookies_dir.join(format!("{}.txt", domain));
    
    match file_extension.as_str() {
        "txt" => {
            if !is_netscape(&file_path)? {
                return Err("File is not a valid Netscape format cookie file".to_string());
            }
            if let Err(e) = std::fs::copy(&file_path, &dest_path) {
                return Err(format!("Failed to copy cookie file: {}", e));
            }
        }
        "json" => {
            convert_json(&file_path, &dest_path)?;
        }
        _ => {
            let _ = app.emit("notification", "Unsupported file format: {}. Only .txt (Netscape format) and .json files are supported");
            return Err("Unsupported file format".to_string());
        }
    }
    
    Ok(format!("Cookie file processed successfully to: {}", dest_path.display()))
}

#[tauri::command]
pub fn get_cookies(app: tauri::AppHandle) -> Result<HashMap<String, String>, String> {
    let cookies_dir = app.path().app_data_dir().unwrap().join("cookies");
    let mut cookie_files = HashMap::new();
    
    match std::fs::read_dir(&cookies_dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                
                if path.is_file() {
                    if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                        cookie_files.insert(
                            file_stem.to_string(),
                            path.to_string_lossy().to_string(),
                        );
                    }
                }
            }            
        }
        Err(e) => {
            let _ = app.emit("notifications", format!("Failed to read cookies directory: {}", e));
        }
    }
    Ok(cookie_files)

}

#[tauri::command]
pub fn clear_cookies(app: tauri::AppHandle) {
    let cookies_dir = app.path().app_data_dir().unwrap().join("cookies");
    let mut deleted_count = 0;
    
    if let Ok(entries) = std::fs::read_dir(&cookies_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && std::fs::remove_file(&path).is_ok() {
                deleted_count += 1;
            }
        }
    }
    
    let _ = app.emit("notification", format!("Successfully cleared {} cookie files", deleted_count));
}

#[tauri::command]
pub fn delete_cookie(app: tauri::AppHandle, path: String) { // delete only one cookie
    let cookie_path = PathBuf::from(&path);
    
    if let Err(e) = std::fs::remove_file(&cookie_path) {
        let _ = app.emit("notification", format!("Unable to delete cookie file, {}", e));
    } else {
        let _ = app.emit("notification", "Cookie deleted!");
    }
}