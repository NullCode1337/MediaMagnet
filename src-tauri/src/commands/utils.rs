use std::{
    io::Write, 
    path::PathBuf
};

use tauri::{Emitter, Manager};

fn is_netscape(file_path: &PathBuf) -> Result<bool, String> {
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return Ok(false);
    }

    let has_header = lines[0].starts_with("# HTTP Cookie File") || 
                     lines[0].starts_with("# Netscape HTTP Cookie File");
    
    let mut has_valid_cookie = false;
    for line in lines {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() == 7 {
            has_valid_cookie = true;
            break;
        }
    }
    Ok(has_valid_cookie || has_header)
}

fn convert_json(source_path: &PathBuf, dest_path: &PathBuf) -> Result<(), String> {
    let content = std::fs::read_to_string(source_path)
        .map_err(|e| format!("Failed to read JSON file: {}", e))?;
    
    let json_data: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Invalid JSON format: {}", e))?;
    
    let mut output = std::fs::File::create(dest_path).unwrap();

    writeln!(output, "# Netscape HTTP Cookie File").unwrap();
    writeln!(output, "# This file was generated from JSON by MediaMagnet").unwrap();
    writeln!(output, "").unwrap();

    let cookies = match json_data {
        serde_json::Value::Array(arr) => arr,
        serde_json::Value::Object(obj) => {
            if let Some(serde_json::Value::Array(cookies)) = obj.get("cookies") {
                cookies.clone()
            } else {
                vec![serde_json::Value::Object(obj)]
            }
        }
        _ => return Err("JSON must be an array or object".to_string()),
    };
    
    for cookie in cookies {
        if let serde_json::Value::Object(cookie_obj) = cookie {
            let domain = cookie_obj.get("domain")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            
            let flag = cookie_obj.get("httpOnly")
                .and_then(|v| v.as_bool())
                .map(|b| if b { "TRUE" } else { "FALSE" })
                .unwrap_or("FALSE")
                .to_string();
            
            let path = cookie_obj.get("path")
                .and_then(|v| v.as_str())
                .unwrap_or("/")
                .to_string();
            
            let secure = cookie_obj.get("secure")
                .and_then(|v| v.as_bool())
                .map(|b| if b { "TRUE" } else { "FALSE" })
                .unwrap_or("FALSE")
                .to_string();
            
            let expiration = cookie_obj.get("expirationDate")
                .or_else(|| cookie_obj.get("expires"))
                .and_then(|v| {
                    if let Some(num) = v.as_f64() {
                        Some(num as i64)
                    } else if let Some(str) = v.as_str() {
                        str.parse().ok()
                    } else {
                        None
                    }
                })
                .unwrap_or(0);
            
            let name = cookie_obj.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            
            let value = cookie_obj.get("value")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            
            writeln!(output, "{}\t{}\t{}\t{}\t{}\t{}\t{}", 
                    domain, flag, path, secure, expiration, name, value)
                .map_err(|e| format!("Failed to write cookie data: {}", e))?;
        }
    }
    Ok(())
}

// Cookies
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
pub fn clear_cookies(app: tauri::AppHandle) {

}

// JSON clearer
#[tauri::command]
pub fn overwrite_json(app: tauri::AppHandle, links: Vec<String>) {
    let path = app.path().app_data_dir().unwrap().join("links.json");

    let mut unique_links = Vec::new();
    let mut seen_links = std::collections::HashSet::new();

    for link in links {
        if !seen_links.contains(&link) {
            seen_links.insert(link.clone());
            unique_links.push(link);
        }
    }

    let json_data = serde_json::to_string_pretty(&unique_links).unwrap();

    std::fs::File::create(&path)
        .unwrap()
        .write_all(json_data.as_bytes())
        .unwrap();
}