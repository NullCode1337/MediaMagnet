use serde::{Deserialize, Serialize};
use serde_json::Number;
use tauri::Manager;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SiteArguments {
    pub id: String,
    pub domain: String,
    pub args: String,
}

fn default_download_path() -> String {
    "Default".to_string()
}
fn default_hue() -> Number {
    Number::from(260)
}
fn default_user_agent() -> String {
    "None".to_string()
}
fn default_dark_mode() -> bool {
    true
}
fn default_yt_format() -> String {
    "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best".to_string()
}
fn default_yt_output_template() -> String {
    "%(title)s.%(ext)s".to_string()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    #[serde(default = "default_download_path")]
    pub download_path: String,

    #[serde(default = "default_user_agent")]
    pub user_agent: String,

    #[serde(default = "default_dark_mode")]
    pub dark_mode: bool,

    #[serde(default = "default_hue")]
    pub accent_hue: serde_json::Number,

    #[serde(default)]
    pub always_on_top: bool,

    #[serde(default)]
    pub show_custom: bool,

    #[serde(default)]
    pub notifications: bool,

    #[serde(default)]
    pub clear_on_exit: bool,

    // YouTube / yt-dlp backend options
    #[serde(default = "default_yt_format")]
    pub yt_format: String,

    #[serde(default = "default_yt_output_template")]
    pub yt_output_template: String,

    #[serde(default)]
    pub yt_embed_thumbnail: bool,

    #[serde(default)]
    pub yt_embed_subs: bool,

    #[serde(default)]
    pub yt_restrict_filenames: bool,

    #[serde(default)]
    pub yt_global_args: String,

    #[serde(default)]
    pub yt_site_args: Vec<SiteArguments>,

    // gallery-dl backend options
    #[serde(default)]
    pub gdl_global_args: String,

    #[serde(default)]
    pub gdl_site_args: Vec<SiteArguments>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            download_path: default_download_path(),
            user_agent: default_user_agent(),
            dark_mode: default_dark_mode(),
            accent_hue: default_hue(),
            always_on_top: false,
            show_custom: false,
            notifications: false,
            clear_on_exit: false,

            yt_format: default_yt_format(),
            yt_output_template: default_yt_output_template(),
            yt_embed_thumbnail: false,
            yt_embed_subs: false,
            yt_restrict_filenames: false,

            yt_global_args: "".to_string(),
            yt_site_args: Vec::new(),

            gdl_global_args: "".to_string(),
            gdl_site_args: Vec::new(),
        }
    }
}

impl Settings {
    pub fn apply(&self, app: &tauri::AppHandle) {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.set_always_on_top(self.always_on_top);
            let _ = window.set_decorations(!self.show_custom);
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
