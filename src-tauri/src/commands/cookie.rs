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
pub async fn create_cookie(app: tauri::AppHandle, content: String, domain: String) -> Result<String, String> {    
    let cookies_dir = app.path().app_data_dir().unwrap().join("cookies");
    
    let temp = cookies_dir.join("temp_text.txt");
    let _ = std::fs::write(&temp, &content);
    
    let is_netscape = is_netscape(&temp)?;
    let is_json = content.trim().starts_with('{') || content.trim().starts_with('[');
    
    let final_name = format!("{}.txt", domain);
    let final_path = cookies_dir.join(&final_name);

    if is_json {
        let _ = std::fs::remove_file(&temp);
        let temp_json = cookies_dir.join("temp_json.json");
        let _ = std::fs::write(&temp_json, &content);

        convert_json(&temp_json, &final_path)?;
    } else if is_netscape {
        std::fs::write(&final_path, content).map_err(|e| e.to_string())?;
    } else {
        let _ = std::fs::remove_file(temp);
        return Err("Invalid format. Please provide Netscape/JSON cookies or cookie file path".to_string());
    }
    
    let _ = std::fs::remove_file(temp);
    
    Ok(final_name)
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