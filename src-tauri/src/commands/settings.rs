use serde::{Deserialize, Serialize};
use tauri::Manager;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub download_path: String,
    pub user_agent: String,
    pub dark_mode: bool,
    pub always_on_top: bool,
    pub show_decor: bool,
    pub notifications: bool,
    pub clear_on_exit: bool
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            download_path: "Default".to_string(),
            user_agent: "None".to_string(),
            dark_mode: true,
            always_on_top: true,
            show_decor: true,
            notifications: false,
            clear_on_exit: false
        }
    }
}

impl Settings {
    pub fn apply(&self, app: &tauri::AppHandle) {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.set_always_on_top(self.always_on_top);
            let _ = window.set_decorations(!self.show_decor);
        }
    }

    pub fn save(&self, app: &tauri::AppHandle) {
        if let Ok(config_path) = app
            .path()
            .app_config_dir()
            .map(|dir| dir.join("settings.json"))
        {
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
pub fn settings(app: tauri::AppHandle, action: String) -> Settings {
    match action.as_str() {
        "check" => {
            let settings = Settings::load(&app);
            settings.apply(&app);
            settings
        }
        "reset" => {
            let default = Settings::default();
            default.apply(&app);
            default.save(&app);
            default
        }
        _ => Settings::default(),
    }
}

#[tauri::command]
pub fn update_settings(app: tauri::AppHandle, settings: Settings) {
    settings.apply(&app);
    settings.save(&app);
}
