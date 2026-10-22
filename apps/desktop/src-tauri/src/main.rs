// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let _ = fix_path_env::fix();

    if std::env::args().skip(1).any(|arg| arg == "--launch-server") {
        if let Err(e) = mediamagnet_lib::launch_server() {
            eprintln!("[MediaMagnet][Server] {e}");
            std::process::exit(1);
        }
        return;
    }

    mediamagnet_lib::run()
}
