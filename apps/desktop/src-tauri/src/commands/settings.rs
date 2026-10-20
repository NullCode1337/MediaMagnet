use mm_core::Settings;
use serde_json::Value;
use tauri::{State, WebviewWindow};

use crate::app::{apply_window_settings, App};

fn apply(app: &App, window: &WebviewWindow, settings: &Settings) {
    apply_window_settings(window, settings);
    let _ = app;
}

#[tauri::command]
pub fn settings(
    app: State<'_, App>,
    window: WebviewWindow,
    action: String,
) -> Result<Settings, String> {
    match action.as_str() {
        "check" => {
            let settings = Settings::load_from(&app.config_dir);
            apply(&app, &window, &settings);
            Ok(settings)
        }
        "reset" => {
            let default = Settings::default();
            apply(&app, &window, &default);
            default.save_to(&app.config_dir)?;
            Ok(default)
        }
        _ => Err(format!("unknown settings action: {action}")),
    }
}

#[tauri::command]
pub fn update_settings(
    app: State<'_, App>,
    window: WebviewWindow,
    settings: Value,
) -> Result<Settings, String> {
    let parsed = Settings::from_value(&settings)?;
    apply(&app, &window, &parsed);
    parsed.save_to(&app.config_dir)?;
    Ok(parsed)
}
