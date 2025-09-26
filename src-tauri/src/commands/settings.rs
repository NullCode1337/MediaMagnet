use tauri::{Emitter, Manager};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub download_path: String,
    pub dark_mode: bool,
    pub always_on_top: bool,
    pub notifications: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            download_path: "Default".to_string(),
            dark_mode: true,
            always_on_top: true,
            notifications: false,
        }
    }
}

impl Settings {
    pub fn apply(&self, app: &tauri::AppHandle) {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.set_always_on_top(self.always_on_top);
        }
    }

    pub fn save(&self, app: &tauri::AppHandle) {
        if let Ok(config_path) = app.path().app_config_dir().map(|dir| dir.join("settings.json")) {
            if let Ok(json) = serde_json::to_string_pretty(self) {
                std::fs::write(&config_path, json).unwrap();
            }
        }
    }

    pub fn load(app: &tauri::AppHandle) -> Self {
        let config_path = app.path().app_config_dir().unwrap().join("settings.json");
        let configs = std::fs::read_to_string(&config_path).unwrap();
        if let Ok(settings) = serde_json::from_str(&configs) {
            return settings;
        }
        Settings::default()
    }
}

#[tauri::command]
pub fn settings(app: tauri::AppHandle, action: String) {
    match action.as_str() {
        "check" => {
            let settings = Settings::load(&app);
            settings.apply(&app);
            let _ = app.emit("settings", &settings);
        }
        "reset" => {
            let default = Settings::default();
            default.apply(&app);
            default.save(&app);
            let _ = app.emit("settings", default);
        }
        _ => { }
    }
}

#[tauri::command]
pub fn update_settings(app: tauri::AppHandle, settings: Settings) {
    settings.apply(&app);
    settings.save(&app);
}