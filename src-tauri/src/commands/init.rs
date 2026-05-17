use serde::{Deserialize, Serialize};
use std::io::Write;
use tauri::{Emitter, Manager};

use super::settings::Settings;

#[derive(Clone, Serialize, Deserialize)]
struct LinkEvent {
    links: Vec<String>,
    message: String,
}

#[tauri::command]
pub fn check_links(app: tauri::AppHandle) {
    let path = app.path().app_data_dir().unwrap().join("links.json");

    let contents = std::fs::read_to_string(&path).unwrap_or_default();
    let links: Vec<String> = serde_json::from_str(&contents).unwrap_or_default();

    if links.is_empty() {
        println!("[MediaMagnet][Init] No pending downloads from last startup!");
        app.emit(
            "link-event",
            LinkEvent {
                links: vec![],
                message: "Nothing".to_string(),
            },
        )
        .unwrap();
    } else {
        println!("[MediaMagnet][Init] {} downloads pending!", links.len());
        app.emit(
            "link-event",
            LinkEvent {
                message: format!("Found {} links", links.len()),
                links,
            },
        )
        .unwrap();
    }
}

pub fn init_config(app: tauri::AppHandle) {
    let app_data_dir = app.path().app_data_dir().unwrap();
    let app_config_dir = app.path().app_config_dir().unwrap();

    let cookies_dir = app_data_dir.join("cookies");
    let links_json = app_data_dir.join("links.json");
    let settings_json = app_config_dir.join("settings.json");

    for dir in [&app_data_dir, &cookies_dir, &app_config_dir] {
        if !dir.exists() {
            std::fs::create_dir_all(dir)
                .unwrap_or_else(|e| panic!("[MediaMagnet][Init] Failed to create {dir:?}: {e}"));
        }
    }

    let settings = Settings::load(&app);
    settings.save(&app);

    if settings_json.exists() {
        println!("[MediaMagnet][Init] Settings verified OK");
    } else {
        println!("[MediaMagnet][Init] Default settings written");
    }

    if links_json.exists() {
        if let Ok(content) = std::fs::read_to_string(&links_json) {
            if serde_json::from_str::<serde_json::Value>(&content).is_ok() {
                println!("[MediaMagnet][Init] Valid links.json file found!");
                return;
            }
        }
        println!("[MediaMagnet][Init] Corrupted data file, recreating...");
    }

    let mut file = std::fs::File::create(&links_json).unwrap();
    file.write_all(b"[]")
        .unwrap_or_else(|e| panic!("Failed to write to data file: {e}"));
}
