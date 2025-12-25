use std::{
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};

use super::settings::Settings;
use tauri::{Emitter, Manager};

// Check if a cookies file is Netscape
pub fn is_netscape(file_path: &PathBuf) -> Result<bool, String> {
    let file = std::fs::File::open(file_path).map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(file);

    let mut has_header = false;
    let mut has_valid_cookie = false;
    let mut lines_iter = reader.lines().peekable();

    if let Some(Ok(first_line)) = lines_iter.peek() {
        let trimmed_first_line = first_line.trim();
        if trimmed_first_line.starts_with("# HTTP Cookie File")
            || trimmed_first_line.starts_with("# Netscape HTTP Cookie File")
        {
            has_header = true;
        }
    }

    for line_result in lines_iter {
        let line = line_result.map_err(|e| format!("Failed to read line: {}", e))?;
        let trimmed_line = line.trim();

        if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
            continue;
        }

        if trimmed_line.split('\t').count() == 7 {
            has_valid_cookie = true;
            break;
        }
    }

    Ok(has_header || has_valid_cookie)
}

// Convert a JSON cookie file into Netscape
pub fn convert_json(source_path: &PathBuf, dest_path: &PathBuf) -> Result<(), String> {
    let content = std::fs::read_to_string(source_path)
        .map_err(|e| format!("Failed to read JSON file: {}", e))?;

    let json_data: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("Invalid JSON format: {}", e))?;

    let output_file = std::fs::File::create(dest_path).unwrap();
    let mut output = std::io::BufWriter::new(output_file);

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
            let domain = cookie_obj
                .get("domain")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let flag = if domain.starts_with('.') {
                "TRUE"
            } else {
                "FALSE"
            };

            let is_http_only = cookie_obj
                .get("httpOnly")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let line_prefix = if is_http_only { "#HttpOnly_" } else { "" };

            let path = cookie_obj
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or("/");

            let secure = cookie_obj
                .get("secure")
                .and_then(|v| v.as_bool())
                .map(|b| if b { "TRUE" } else { "FALSE" })
                .unwrap_or("FALSE");

            let expiration = cookie_obj
                .get("expirationDate")
                .or_else(|| cookie_obj.get("expires"))
                .and_then(|v| {
                    if let Some(num) = v.as_f64() {
                        Some(num as i64)
                    } else if let Some(str) = v.as_str() {
                        str.parse::<f64>()
                            .ok()
                            .map(|f| f as i64)
                            .or_else(|| str.parse().ok())
                    } else {
                        None
                    }
                })
                .unwrap_or(0);

            let name = cookie_obj
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let value = cookie_obj
                .get("value")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            writeln!(
                output,
                "{}{}\t{}\t{}\t{}\t{}\t{}\t{}",
                line_prefix, domain, flag, path, secure, expiration, name, value
            )
            .map_err(|e| format!("Failed to write cookie data: {}", e))?;
        }
    }

    output
        .flush()
        .map_err(|e| format!("Failed to flush cookie file: {}", e))?;

    Ok(())
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

// Set download path
pub async fn set_download_path(app: tauri::AppHandle) {
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