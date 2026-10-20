use std::path::{Path, PathBuf};
use std::sync::Arc;

use mm_core::events::SharedEventSink;
use mm_core::host::Host;
use mm_core::runtime::{RuntimeConfig, ToolRuntime};
use mm_core::{Engine, Policy};
use tauri::{AppHandle, Manager};

use crate::events::TauriEventSink;

pub struct App {
    pub engine: Arc<Engine>,
    pub config_dir: PathBuf,
    pub host: DesktopHost,
}

pub fn init(app: &AppHandle) {
    let app_data_dir = app.path().app_data_dir().expect("app data dir");
    let config_dir = app.path().app_config_dir().expect("app config dir");

    for dir in [
        app_data_dir.clone(),
        app_data_dir.join("cookies"),
        config_dir.clone(),
    ] {
        if !dir.exists() {
            std::fs::create_dir_all(&dir)
                .unwrap_or_else(|e| panic!("[MediaMagnet][Init] Failed to create {dir:?}: {e}"));
        }
    }

    let settings = mm_core::Settings::load_from(&config_dir);
    if let Err(e) = settings.save_to(&config_dir) {
        eprintln!("[MediaMagnet][Init] Failed to persist settings: {e}");
    }
    println!(
        "[MediaMagnet][Init] {}",
        if config_dir.join("settings.json").exists() {
            "Settings verified OK"
        } else {
            "Default settings written"
        }
    );

    let links_json = app_data_dir.join("links.json");
    match std::fs::read_to_string(&links_json) {
        Ok(content) if serde_json::from_str::<serde_json::Value>(&content).is_ok() => {
            println!("[MediaMagnet][Init] Valid links.json file found!");
        }
        _ => {
            if links_json.exists() {
                println!("[MediaMagnet][Init] Corrupted data file, recreating...");
            }
            let _ = std::fs::write(&links_json, b"[]");
        }
    }

    let host = DesktopHost { app: app.clone() };
    let sink: SharedEventSink = Arc::new(TauriEventSink::new(app.clone()));
    let runtime = Arc::new(ToolRuntime::new(RuntimeConfig::with_bundled_tools_dir(
        bundled_tools_dir(app),
    )));
    let engine = Engine::new(
        runtime,
        config_dir.clone(),
        app_data_dir.clone(),
        Policy::desktop(),
        sink,
    );

    app.manage(App {
        engine,
        config_dir,
        host,
    });
}

fn bundled_tools_dir(app: &AppHandle) -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
        .unwrap_or_default();
    if exe_dir.join("gallery-dl").is_file() || exe_dir.join("gallery-dl.exe").is_file() {
        exe_dir
    } else {
        std::env::var_os("MM_BINARIES_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                app.path()
                    .resource_dir()
                    .unwrap_or_default()
                    .join("binaries")
            })
    }
}

pub struct DesktopHost {
    pub app: AppHandle,
}

impl Host for DesktopHost {
    fn default_download_dir(&self) -> PathBuf {
        self.app.path().download_dir().unwrap_or_default()
    }

    fn notify(&self, message: &str) -> bool {
        let config_dir = self.app.path().app_config_dir().unwrap_or_default();
        if !mm_core::Settings::load_from(&config_dir).native_notifications {
            return false;
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("notify-send")
                .arg(message)
                .spawn()
                .is_ok()
        }
        #[cfg(not(target_os = "linux"))]
        {
            use tauri_plugin_notification::NotificationExt;
            self.app
                .notification()
                .builder()
                .title("MediaMagnet")
                .body(message)
                .show()
                .is_ok()
        }
    }

    fn open_path(&self, path: &Path) -> mm_core::Result<()> {
        use tauri_plugin_opener::OpenerExt;
        self.app
            .opener()
            .open_path(path.to_string_lossy(), None::<&str>)
            .map_err(|e| mm_core::Error::Msg(e.to_string()))
    }
}

pub fn apply_window_settings(window: &tauri::WebviewWindow, settings: &mm_core::Settings) {
    #[cfg(not(target_os = "android"))]
    {
        let _ = window.set_always_on_top(settings.always_on_top);
        let _ = window.set_decorations(!settings.custom_titlebar);
    }
    #[cfg(target_os = "android")]
    let _ = (window, settings);
}
