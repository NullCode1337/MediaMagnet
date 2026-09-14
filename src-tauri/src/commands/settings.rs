use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Write;
use std::path::{Path, PathBuf};
use tauri::Manager;

const SETTINGS_VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(default)]
pub struct Settings {
    // schema
    #[serde(default = "current_settings_version")]
    pub settings_version: u32,

    // app options
    pub dark_mode: bool,
    pub amoled_mode: bool,
    pub accent_hue: u32,
    pub accent_kind: String,
    pub always_on_top: bool,
    pub custom_titlebar: bool,
    pub custom_titlebar_type: String,
    pub native_notifications: bool,
    pub clear_cookies_on_exit: bool,

    // download options
    pub download_path: String,
    pub user_agent: String,

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

fn current_settings_version() -> u32 {
    SETTINGS_VERSION
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SiteArguments {
    pub id: String,
    pub domain: String,
    pub args: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            settings_version: SETTINGS_VERSION,

            download_path: String::new(),
            user_agent: String::new(),
            dark_mode: true,
            amoled_mode: false,
            accent_hue: 260,
            accent_kind: "hue".to_string(),
            custom_titlebar_type: "system".to_string(),
            yt_format: "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best".to_string(),
            yt_output_template: "%(title)s.%(ext)s".to_string(),

            always_on_top: false,
            custom_titlebar: false,
            native_notifications: false,
            clear_cookies_on_exit: false,
            yt_embed_thumbnail: false,
            yt_embed_subs: false,
            yt_restrict_filenames: false,

            yt_global_args: String::new(),
            gdl_global_args: String::new(),
            spotdl_global_args: String::new(),
            spotdl_bitrate: String::new(),
            spotdl_format: String::new(),

            yt_site_args: Vec::new(),
            gdl_site_args: Vec::new(),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct SanitizeReport {
    pub dropped: Vec<String>,
    pub defaulted: Vec<String>,
    pub coerced: Vec<String>,
}

#[derive(Clone, Copy, PartialEq)]
enum Type {
    Bool,
    Str,
    Hue,
    AccentKind,
    Version,
    SiteArgs,
}

const KNOWN_KEYS: &[&str] = &[
    "settings_version",
    "dark_mode",
    "amoled_mode",
    "accent_hue",
    "accent_kind",
    "always_on_top",
    "custom_titlebar",
    "custom_titlebar_type",
    "native_notifications",
    "clear_cookies_on_exit",
    "download_path",
    "user_agent",
    "yt_format",
    "yt_output_template",
    "yt_embed_thumbnail",
    "yt_embed_subs",
    "yt_restrict_filenames",
    "yt_global_args",
    "yt_site_args",
    "gdl_global_args",
    "gdl_site_args",
    "spotdl_format",
    "spotdl_bitrate",
    "spotdl_global_args",
];

fn field_type(key: &str) -> Option<Type> {
    match key {
        "dark_mode"
        | "amoled_mode"
        | "always_on_top"
        | "custom_titlebar"
        | "native_notifications"
        | "clear_cookies_on_exit"
        | "yt_embed_thumbnail"
        | "yt_embed_subs"
        | "yt_restrict_filenames" => Some(Type::Bool),

        "accent_hue" => Some(Type::Hue),
        "accent_kind" => Some(Type::AccentKind),
        "settings_version" => Some(Type::Version),

        "custom_titlebar_type"
        | "download_path"
        | "user_agent"
        | "yt_format"
        | "yt_output_template"
        | "yt_global_args"
        | "gdl_global_args"
        | "spotdl_format"
        | "spotdl_bitrate"
        | "spotdl_global_args" => Some(Type::Str),

        "yt_site_args" | "gdl_site_args" => Some(Type::SiteArgs),

        _ => None,
    }
}

pub fn sanitize_settings(value: &Value) -> (Value, SanitizeReport) {
    let mut report = SanitizeReport::default();
    let mut out = serde_json::Map::new();

    let object = match value {
        Value::Object(map) => map,
        _ => {
            report.dropped.push("<root>".to_string());
            return (Value::Object(out), report);
        }
    };

    for (key, item) in object {
        let kind = match field_type(key) {
            Some(kind) => kind,
            None => {
                report.dropped.push(key.clone());
                continue;
            }
        };

        match kind {
            Type::Bool => {
                if item.is_boolean() {
                    out.insert(key.clone(), item.clone());
                } else {
                    report.dropped.push(key.clone());
                }
            }
            Type::Str => {
                if item.is_string() {
                    out.insert(key.clone(), item.clone());
                } else {
                    report.dropped.push(key.clone());
                }
            }
            Type::Hue => match item.as_f64() {
                Some(h) if h.is_finite() => {
                    let clamped = h.round().clamp(0.0, 360.0) as u32;
                    if clamped as f64 != h {
                        report.coerced.push(key.clone());
                    }
                    out.insert(key.clone(), Value::from(clamped));
                }
                _ => report.dropped.push(key.clone()),
            },
            Type::AccentKind => {
                if item
                    .as_str()
                    .is_some_and(|s| matches!(s, "hue" | "black" | "white"))
                {
                    out.insert(key.clone(), item.clone());
                } else {
                    report.dropped.push(key.clone());
                }
            }
            Type::Version => match item.as_u64() {
                Some(v) if (1..=1_000_000).contains(&v) => {
                    out.insert(key.clone(), Value::from(v as u32));
                }
                _ => report.dropped.push(key.clone()),
            },
            Type::SiteArgs => match item.as_array() {
                Some(entries) => {
                    let mut kept = Vec::new();
                    for (index, entry) in entries.iter().enumerate() {
                        let valid = entry.as_object().is_some_and(|o| {
                            o.get("domain").is_some_and(Value::is_string)
                                && o.get("args").is_some_and(Value::is_string)
                        });
                        if valid {
                            let mut entry = entry.clone();
                            let obj = entry.as_object_mut().unwrap();
                            if !obj.get("id").is_some_and(Value::is_string) {
                                obj.insert(
                                    "id".to_string(),
                                    Value::from(format!("id-{}-{}", key, index)),
                                );
                                report.coerced.push(format!("{key}[{index}]"));
                            }
                            kept.push(entry);
                        } else {
                            report.dropped.push(format!("{key}[{index}]"));
                        }
                    }
                    out.insert(key.clone(), Value::from(kept));
                }
                None => report.dropped.push(key.clone()),
            },
        }
    }

    for known in KNOWN_KEYS {
        if !out.contains_key(*known) {
            report.defaulted.push((*known).to_string());
        }
    }

    (Value::Object(out), report)
}

fn migrate(value: &mut Value, from: u32) {
    let _ = (value, from);
}

fn prepare_settings_value(raw: &Value) -> (Value, SanitizeReport) {
    let (mut value, report) = sanitize_settings(raw);

    let from = value
        .as_object()
        .and_then(|o| o.get("settings_version"))
        .and_then(Value::as_u64)
        .unwrap_or(0) as u32;

    migrate(&mut value, from);

    if let Some(obj) = value.as_object_mut() {
        obj.insert(
            "settings_version".to_string(),
            Value::from(SETTINGS_VERSION),
        );
    }

    (value, report)
}

fn backup_corrupt_file(path: &Path) -> Option<PathBuf> {
    let stem = path.file_stem()?.to_str()?;
    let parent = path.parent()?;
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("json");

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();

    for attempt in 0..100u32 {
        let name = if attempt == 0 {
            format!("{stem}.corrupt-{timestamp}.{extension}")
        } else {
            format!("{stem}.corrupt-{timestamp}-{attempt}.{extension}")
        };
        let candidate = parent.join(name);
        if !candidate.exists() {
            return match std::fs::copy(path, &candidate) {
                Ok(_) => Some(candidate),
                Err(e) => {
                    eprintln!(
                        "[MediaMagnet][Settings] Failed to create backup {:?}: {e}",
                        candidate
                    );
                    None
                }
            };
        }
    }
    None
}

impl Settings {
    pub fn apply(&self, app: &tauri::AppHandle) {
        if let Some(window) = app.get_webview_window("main") {
            #[cfg(not(target_os = "android"))]
            {
                let _ = window.set_always_on_top(self.always_on_top);
                let _ = window.set_decorations(!self.custom_titlebar);
            }
        }
    }

    pub fn save(&self, app: &tauri::AppHandle) -> Result<(), String> {
        let config_path = app
            .path()
            .app_config_dir()
            .map_err(|e| format!("could not resolve config dir: {e}"))?
            .join("settings.json");

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("could not serialize settings: {e}"))?;

        let tmp_path = config_path.with_extension("json.tmp");

        let write_tmp = || -> Result<(), String> {
            let mut file = std::fs::File::create(&tmp_path)
                .map_err(|e| format!("could not create temp file: {e}"))?;
            file.write_all(json.as_bytes())
                .map_err(|e| format!("could not write temp file: {e}"))?;
            file.sync_all()
                .map_err(|e| format!("could not flush temp file: {e}"))?;
            Ok(())
        };
        if let Err(e) = write_tmp() {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(e);
        }

        if let Err(e) = std::fs::rename(&tmp_path, &config_path) {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(format!("could not replace settings file: {e}"));
        }

        #[cfg(unix)]
        if let Some(parent) = config_path.parent() {
            if let Ok(dir) = std::fs::File::open(parent) {
                let _ = dir.sync_all();
            }
        }

        Ok(())
    }

    pub fn load(app: &tauri::AppHandle) -> Self {
        let config_path = match app.path().app_config_dir() {
            Ok(dir) => dir.join("settings.json"),
            Err(e) => {
                eprintln!(
                    "[MediaMagnet][Settings] Could not resolve config dir, using defaults: {e}"
                );
                return Self::default();
            }
        };

        let contents = match std::fs::read_to_string(&config_path) {
            Ok(contents) => contents,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Self::default(),
            Err(e) => {
                eprintln!(
                    "[MediaMagnet][Settings] Could not read settings file, using defaults: {e}"
                );
                return Self::default();
            }
        };

        let raw: Value = match serde_json::from_str(&contents) {
            Ok(raw) => raw,
            Err(e) => {
                let backup = backup_corrupt_file(&config_path);
                eprintln!(
                    "[MediaMagnet][Settings] Corrupted settings file ({e}), backed up to {backup:?}, using defaults"
                );
                return Self::default();
            }
        };

        let (value, _) = prepare_settings_value(&raw);

        match serde_json::from_value::<Settings>(value) {
            Ok(settings) => settings,
            Err(e) => {
                eprintln!("[MediaMagnet][Settings] Failed to build settings after sanitize, using defaults: {e}");
                Self::default()
            }
        }
    }
}

#[tauri::command]
pub fn settings(app: tauri::AppHandle, action: String) -> Result<Settings, String> {
    match action.as_str() {
        "check" => {
            let settings = Settings::load(&app);
            settings.apply(&app);
            Ok(settings)
        }
        "reset" => {
            let default = Settings::default();
            default.apply(&app);
            default.save(&app)?;
            Ok(default)
        }
        _ => Err(format!("unknown settings action: {action}")),
    }
}

#[tauri::command]
pub fn update_settings(app: tauri::AppHandle, settings: Value) -> Result<Settings, String> {
    let (value, _) = prepare_settings_value(&settings);
    let parsed: Settings =
        serde_json::from_value(value).map_err(|e| format!("invalid settings payload: {e}"))?;

    parsed.apply(&app);
    parsed.save(&app)?;
    Ok(parsed)
}
