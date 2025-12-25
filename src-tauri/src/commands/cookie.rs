use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{Emitter, Manager};

use super::utils::{convert_json, is_netscape};

#[derive(Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum CookieInput {
    FilePath(String),
    Content(String),
}

#[tauri::command]
pub async fn save_cookie(
    app: tauri::AppHandle,
    domain: String,
    input: CookieInput,
) -> Result<String, String> {
    let cookies_dir = app.path().app_data_dir().unwrap().join("cookies");
    let dest_path = cookies_dir.join(format!("{}.txt", domain));

    match input {
        CookieInput::FilePath(file_path_str) => {
            let file_path = PathBuf::from(file_path_str);
            if !file_path.exists() {
                return Err(format!(
                    "Source file does not exist: {}",
                    file_path.display()
                ));
            }

            let file_extension = file_path
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("")
                .to_lowercase();

            match file_extension.as_str() {
                "txt" | "cookies" => {
                    if !is_netscape(&file_path)? {
                        return Err("File is not a valid Netscape format cookie file".to_string());
                    }
                    std::fs::copy(&file_path, &dest_path)
                        .map_err(|e| format!("Failed to copy cookie file: {}", e))?;
                }
                "json" => {
                    convert_json(&file_path, &dest_path)?;
                }
                _ => {
                    return Err("Unsupported file format. Only .txt, .cookies (Netscape), and .json files are supported".to_string());
                }
            }
        }

        CookieInput::Content(content) => {
            let is_json =
                content.trim_start().starts_with('{') || content.trim_start().starts_with('[');

            if is_json {
                let temp_json = cookies_dir.join("temp_json.json");
                std::fs::write(&temp_json, &content).map_err(|e| e.to_string())?;

                let convert_result = convert_json(&temp_json, &dest_path);
                let _ = std::fs::remove_file(&temp_json);
                convert_result?;
            } else {
                std::fs::write(&dest_path, &content).map_err(|e| e.to_string())?;

                if !is_netscape(&dest_path)? {
                    let _ = std::fs::remove_file(&dest_path);
                    return Err(
                        "Invalid format. Please provide Netscape/JSON cookies or cookie file path"
                            .to_string(),
                    );
                }
            }
        }
    }

    Ok(dest_path.to_string_lossy().into_owned())
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
                        cookie_files
                            .insert(file_stem.to_string(), path.to_string_lossy().to_string());
                    }
                }
            }
            Ok(cookie_files)
        }
        Err(e) => Err(format!("Failed to read cookies directory: {}", e)),
    }
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
    let _ = app.emit(
        "notification",
        format!("Successfully cleared {} cookie files", deleted_count),
    );
}

#[tauri::command]
pub fn delete_cookie(app: tauri::AppHandle, path: String) {
    let cookie_path = PathBuf::from(&path);

    if let Err(e) = std::fs::remove_file(&cookie_path) {
        let _ = app.emit(
            "notification",
            format!("Unable to delete cookie file: {}", e),
        );
    } else {
        let _ = app.emit("notification", "Cookie deleted!");
    }
}
