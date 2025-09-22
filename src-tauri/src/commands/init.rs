use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use tauri::{Emitter, Manager};

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub download_path: String,
    pub dark_mode: bool,
    pub always_on_top: bool,
    pub notifications: bool,
}

#[derive(Clone, Serialize, Deserialize)]
struct LinkEvent {
    links: Vec<String>,
    message: String,
}

#[tauri::command]
pub fn check_links(app: tauri::AppHandle) {
    let path = app.path().app_data_dir().unwrap().join("links.json");

    let mut file = std::fs::File::open(path).unwrap();
    let mut contents = String::new();
    let _ = file.read_to_string(&mut contents);

    let links: Vec<String> = serde_json::from_str(&contents).unwrap();

    if links.is_empty() {
        let event = LinkEvent {
            links: vec![],
            message: "Nothing".to_string(),
        };

        app.emit("link-event", event).unwrap();
    } else {
        let event = LinkEvent {
            links: links.clone(),
            message: format!("Found {} links", links.len()),
        };

        app.emit("link-event", event).unwrap();
    }
}

pub fn init_config(app: tauri::AppHandle) {
    let app_data_dir = app.path().app_data_dir().unwrap();
    let app_config_dir = app.path().app_config_dir().unwrap();

    let links_json = app_data_dir.join("links.json");
    let settings_json = app_config_dir.join("settings.json");

    if !app_data_dir.exists() {
        std::fs::create_dir_all(&app_data_dir)
            .map_err(|e| format!("Failed to create app data directory: {}", e))
            .unwrap();
    }

    if !app_config_dir.exists() {
        std::fs::create_dir_all(&app_config_dir)
            .map_err(|e| format!("Failed to create app config directory: {}", e))
            .unwrap();
    }

    if links_json.exists() {
        if let Ok(content) = std::fs::read_to_string(&links_json) {
            if serde_json::from_str::<serde_json::Value>(&content).is_ok() {
                println!("Valid data file found");
                return;
            }
        }
        println!("Corrupted data file, recreating...");
    }

    if !settings_json.exists() {
        let default_settings = Settings {
            download_path: "Default".to_string(),
            dark_mode: true,
            always_on_top: true,
            notifications: false,
        };

        let settings_json_content = serde_json::to_string_pretty(&default_settings).unwrap();
        
        std::fs::write(&settings_json, settings_json_content)
            .map_err(|e| format!("Failed to write default settings to file: {}", e))
            .unwrap();
    }

    let mut file = std::fs::File::create(&links_json).unwrap();

    file.write_all(b"[]")
        .map_err(|e| format!("Failed to write to data file: {}", e))
        .unwrap();
}