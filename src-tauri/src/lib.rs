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
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::cookie::add_cookie,
            commands::cookie::clear_cookies,
            commands::cookie::create_cookie,
            commands::cookie::delete_cookie,
            commands::cookie::get_cookies,
            commands::download::cancel_download,
            commands::download::downloader,
            commands::init::check_links,
            commands::settings::settings,
            commands::settings::update_settings,
            commands::utils::overwrite_json
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
