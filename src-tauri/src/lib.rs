mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            commands::init::init_config(app.handle().clone());
            Ok(())
        })
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            commands::download::downloader,
            commands::download::overwrite_json,
            commands::init::check_links,
            commands::settings::settings,
            commands::settings::update_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
