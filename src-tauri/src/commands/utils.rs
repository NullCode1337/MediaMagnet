use serde_json::Value;
use std::{
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
};

use super::settings::Settings;
use tauri::{Emitter, Manager};

pub fn is_netscape(file_path: &Path) -> Result<bool, String> {
    let file = std::fs::File::open(file_path).map_err(|e| {
        format!(
            "[MediaMagnet][Utils] Failed to open cookie file '{}': {}",
            file_path.display(),
            e
        )
    })?;

    let reader = BufReader::new(file);
    let mut has_header = false;
    let mut has_valid_cookie = false;

    for (line_num, result) in reader.lines().enumerate() {
        let line = result.map_err(|e| format!("Failed to read line {}: {}", line_num + 1, e))?;

        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("# Netscape") || trimmed.starts_with("# HTTP Cookie File") {
            has_header = true;
            continue;
        }

        if trimmed.starts_with('#') && !trimmed.to_lowercase().starts_with("#httponly_") {
            continue;
        }

        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() == 7 {
            has_valid_cookie = true;
            if has_header {
                break;
            }
        }
    }

    Ok(has_header || has_valid_cookie)
}

// Convert a JSON cookie file into Netscape
pub fn convert_json(source_path: &Path, dest_path: &Path) -> Result<(), String> {
    let content = std::fs::read_to_string(source_path)
        .map_err(|e| format!("[MediaMagnet][Utils] Failed to read JSON file: {}", e))?;

    let json_data: Value = serde_json::from_str(&content)
        .map_err(|e| format!("[MediaMagnet][Utils] Invalid JSON format: {}", e))?;

    let output_file = std::fs::File::create(dest_path).map_err(|e| {
        format!(
            "[MediaMagnet][Utils] Failed to create destination cookie file: {}",
            e
        )
    })?;

    let mut output = BufWriter::new(output_file);

    writeln!(output, "# Netscape HTTP Cookie File").map_err(|e| format!("Write error: {}", e))?;
    writeln!(
        output,
        "# This file was generated from JSON by MediaMagnet\n"
    )
    .map_err(|e| format!("Write error: {}", e))?;

    let cookies = match json_data {
        Value::Array(arr) => arr,
        Value::Object(obj) => {
            if let Some(Value::Array(cookies)) = obj.get("cookies") {
                cookies.clone()
            } else {
                vec![Value::Object(obj)]
            }
        }
        _ => return Err("JSON must be an array or object".to_string()),
    };

    for cookie in cookies {
        if let Value::Object(c) = cookie {
            let domain = c.get("domain").and_then(|v| v.as_str()).unwrap_or("");

            let flag = if domain.starts_with('.') {
                "TRUE"
            } else {
                "FALSE"
            };

            let path = c.get("path").and_then(|v| v.as_str()).unwrap_or("/");

            let secure = c
                .get("secure")
                .and_then(|v| v.as_bool())
                .map(|b| if b { "TRUE" } else { "FALSE" })
                .unwrap_or("FALSE");

            let expiration = c
                .get("expirationDate")
                .or_else(|| c.get("expires"))
                .and_then(|v| v.as_f64().map(|f| f as i64))
                .unwrap_or(0);

            let name = c.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let value = c.get("value").and_then(|v| v.as_str()).unwrap_or("");

            let http_only: bool = c.get("httpOnly").and_then(|v| v.as_bool()).unwrap_or(false);
            let prefix = if http_only { "#HttpOnly_" } else { "" };

            writeln!(
                output,
                "{}{}\t{}\t{}\t{}\t{}\t{}\t{}",
                prefix, domain, flag, path, secure, expiration, name, value
            )
            .map_err(|e| format!("[MediaMagnet][Utils] Failed to write cookie: {}", e))?;
        }
    }

    output.flush().map_err(|e| format!("Flush error: {}", e))?;
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
pub async fn set_download_path(app: tauri::AppHandle) -> std::path::PathBuf {
    let settings = Settings::load(&app);
    let default_dir = app.path().download_dir().unwrap().join("MediaMagnet");

    let downloads_path = match settings.download_path.as_str() {
        "" => default_dir.clone(),
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
                std::fs::create_dir_all(&default_dir).unwrap();
                default_dir
            })
    };

    std::env::set_current_dir(&final_dir).unwrap();

    final_dir
}

#[tauri::command]
pub async fn get_free_space(app: tauri::AppHandle) -> Result<f64, String> {
    let settings = Settings::load(&app);
    let download_path = if settings.download_path.is_empty() {
        app.path().download_dir().unwrap_or_default()
    } else {
        Path::new(&settings.download_path).to_path_buf()
    };

    let disks = sysinfo::Disks::new_with_refreshed_list();

    let disk = disks
        .iter()
        .filter(|d| download_path.starts_with(d.mount_point()))
        .max_by_key(|d| d.mount_point().as_os_str().len());

    if let Some(d) = disk {
        let total = d.total_space() as f64;
        let available = d.available_space() as f64;
        let used_percent = ((total - available) / total) * 100.0;
        Ok(used_percent)
    } else {
        Err("Could not determine disk usage".to_string())
    }
}

#[tauri::command]
pub async fn notify(app: tauri::AppHandle, body: String) -> Result<(), String> {
    let settings = Settings::load(&app);

    if settings.native_notifications {
        #[cfg(target_os = "linux")]
        {
            use std::process::Command;
            Command::new("notify-send")
                //.arg("--app-name=MediaMagnet") // For some reason this causes
                //.arg(title)                    // notifications to not send
                .arg(&body)
                .spawn()
                .map_err(|e| format!("Notification error: {e}"))?;
        }

        #[cfg(not(target_os = "linux"))]
        {
            use tauri_plugin_notification::NotificationExt;
            app.notification()
                .builder()
                .title("MediaMagnet")
                .body(&body)
                .show()
                .map_err(|e| format!("Notification error: {e}"))?;
        }
    } else {
        let _ = app.emit("notification", body.to_string());
    }
    Ok(())
}
