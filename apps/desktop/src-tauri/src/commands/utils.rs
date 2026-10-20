use std::path::Path;

use mm_core::host::{disk_usage, Host};
use mm_core::Settings;
use tauri::State;

use crate::app::App;

#[tauri::command]
pub fn check_download_path(path: String) -> Result<(), String> {
    if path.trim().is_empty() || path.trim().eq_ignore_ascii_case("default") {
        return Ok(());
    }

    let dir = Path::new(&path);

    if let Err(e) = std::fs::create_dir_all(dir) {
        return Err(format!("cannot create directory: {e}"));
    }

    let meta = std::fs::metadata(dir).map_err(|e| format!("cannot access path: {e}"))?;

    if !meta.is_dir() {
        return Err("path is not a directory".to_string());
    }

    let write = dir.join(".mediamagnet-write-test");
    match std::fs::File::create(&write) {
        Ok(file) => {
            drop(file);
            let _ = std::fs::remove_file(&write);
            Ok(())
        }
        Err(e) => Err(format!("no write permission: {e}")),
    }
}

#[tauri::command]
pub async fn get_free_space(app: State<'_, App>) -> Result<f64, String> {
    let settings = Settings::load_from(&app.config_dir);
    let download_path = if settings.download_path.is_empty() {
        app.host.default_download_dir()
    } else {
        std::path::PathBuf::from(&settings.download_path)
    };

    disk_usage(&download_path)
        .map(|usage| usage.used_percent)
        .map_err(|e| e.to_string())
}

/// Native notification when enabled, otherwise an in-app toast, as before.
#[tauri::command]
pub async fn notify(app: State<'_, App>, body: String) -> Result<(), String> {
    if !app.host.notify(&body) {
        app.engine.notify(&body);
    }
    Ok(())
}
