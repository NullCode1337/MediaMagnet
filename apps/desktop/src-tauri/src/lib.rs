mod app;
mod commands;
mod events;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.unminimize();
            let _ = window.set_focus();
        }
    }));

    builder
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            const LOGO: &str = r#"
                 _ _                                     _
  /\/\   ___  __| (_) __ _  /\/\   __ _  __ _ _ __   ___| |_
 /    \ / _ \/ _` | |/ _` |/    \ / _` |/ _` | '_ \ / _ \ __|
/ /\/\ \  __/ (_| | | (_| / /\/\ \ (_| | (_| | | | |  __/ |_
\/    \/\___|\__,_|_|\__,_\/    \/\__,_|\__, |_| |_|\___|\__|
       Download any and all media       |___/                "#;

            println!("{}\n", LOGO);
            println!("-> Version: {}", app.package_info().version);
            app::init(app.handle());
            Ok(())
        })
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::cookie::clear_cookies,
            commands::cookie::delete_cookie,
            commands::cookie::get_cookies,
            commands::cookie::save_cookie,
            commands::download::cancel_download,
            commands::download::cancel_all_downloads,
            commands::download::downloader,
            commands::download::pause_download,
            commands::settings::settings,
            commands::settings::update_settings,
            commands::utils::check_download_path,
            commands::utils::get_free_space,
            commands::utils::notify,
            commands::utils::reinstall_python_deps,
            commands::utils::reset_python_deps,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
