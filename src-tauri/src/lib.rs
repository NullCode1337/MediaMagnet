mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
            println!("-> Version: {}", app.package_info().version.to_string());
            commands::init::init_config(app.handle().clone());

            Ok(())
        })
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::cookie::clear_cookies,
            commands::cookie::delete_cookie,
            commands::cookie::get_cookies,
            commands::cookie::save_cookie,
            commands::download::cancel_download,
            commands::download::downloader,
            commands::init::check_links,
            commands::settings::settings,
            commands::settings::update_settings,
            commands::utils::overwrite_json,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
