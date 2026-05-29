use serde::{Deserialize, Serialize};
use serde_json::Number;
use tauri::Manager;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SiteArguments {
    pub id: String,
    pub domain: String,
    pub args: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Settings {
    pub download_path: String,
    pub user_agent: String,
    pub dark_mode: bool,
    pub accent_hue: Number,
    pub always_on_top: bool,
    pub show_custom: bool,
    pub custom_type: String,
    pub notifications: bool,
    pub clear_on_exit: bool,

    // yt-dlp backend options
    pub yt_format: String,
    pub yt_output_template: String,
    pub yt_embed_thumbnail: bool,
    pub yt_embed_subs: bool,
    pub yt_restrict_filenames: bool,
    pub yt_global_args: String,
    pub yt_site_args: Vec<SiteArguments>,

    // gallery-dl backend options
    pub gdl_global_args: String,
    pub gdl_site_args: Vec<SiteArguments>,

    // spotdl backend options
    pub spotdl_format: String,
    pub spotdl_bitrate: String,
    pub spotdl_global_args: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            download_path: "Default".to_string(),
            user_agent: "None".to_string(),
            dark_mode: true,
            accent_hue: Number::from(260),
            custom_type: "system".to_string(),
            yt_format: "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best".to_string(),
            yt_output_template: "%(title)s.%(ext)s".to_string(),

            always_on_top: false,
            show_custom: false,
            notifications: false,
            clear_on_exit: false,
            yt_embed_thumbnail: false,
            yt_embed_subs: false,
            yt_restrict_filenames: false,
            spotdl_bitrate: String::new(),
            spotdl_format: String::new(),
            yt_global_args: String::new(),
            yt_site_args: Vec::new(),
            gdl_global_args: String::new(),
            gdl_site_args: Vec::new(),
            spotdl_global_args: String::new(),
        }
    }
}

impl Settings {
    pub fn apply(&self, app: &tauri::AppHandle) {
        if let Some(window) = app.get_webview_window("main") {
            #[cfg(not(target_os = "android"))] {
                let _ = window.set_always_on_top(self.always_on_top);
                let _ = window.set_decorations(!self.show_custom);
            }
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

        match std::fs::read_to_string(&config_path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_else(|e| {
                eprintln!(
                    "[MediaMagnet][Settings] Failed to parse settings, resetting to defaults: {}",
                    e
                );
                Self::default()
            }),
            Err(e) => {
                eprintln!(
                    "[MediaMagnet][Settings] Could not read settings file, resetting to defaults: {}",
                    e
                );
                Self::default()
            }
        }
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
