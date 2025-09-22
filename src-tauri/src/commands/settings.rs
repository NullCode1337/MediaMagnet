use tauri::{Emitter, Manager};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub download_path: String,
    pub dark_mode: bool,
    pub always_on_top: bool,
    pub notifications: bool,
}

#[tauri::command]
pub fn check_settings(app: tauri::AppHandle) {
    let config_dir = app.path().app_config_dir().unwrap();
    let config_path = config_dir.join("settings.json");
    let configs = std::fs::read_to_string(&config_path).unwrap();
    let settings: Settings = serde_json::from_str(&configs).unwrap();
    app.emit("settings", &settings).unwrap();
}

#[tauri::command]
pub fn reset_settings(app: tauri::AppHandle) {
    let config_dir = app.path().app_config_dir().unwrap();
    let config_path = config_dir.join("settings.json");
    let default_settings = Settings {
        download_path: "None".to_string(),
        dark_mode: true,
        always_on_top: true,
        notifications: false,
    };
    std::fs::write(&config_path, serde_json::to_string_pretty(&default_settings).unwrap())
        .map_err(|e| format!("Failed to write default settings to file: {}", e))
        .unwrap();
}

#[tauri::command]
pub fn update_settings(app: tauri::AppHandle, settings: Settings) {
    let config_dir = app.path().app_config_dir().unwrap();
    let config_path = config_dir.join("settings.json");
    
    app.get_webview_window("main").unwrap().set_always_on_top(settings.always_on_top).unwrap();
    
    std::fs::write(&config_path, serde_json::to_string_pretty(&settings).unwrap())
        .map_err(|e| format!("Failed to write default settings to file: {}", e))
        .unwrap();
}