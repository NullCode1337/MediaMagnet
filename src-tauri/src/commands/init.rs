use std::io::Write;
use tauri::Manager;

pub fn init_config(app: tauri::AppHandle) {
    let file_path = app.path().app_data_dir()
        .unwrap()
        .join("links.json");
    
    if file_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&file_path) {
            if serde_json::from_str::<serde_json::Value>(&content).is_ok() {
                println!("Valid data file found");
                return;
            }
        }
        println!("Corrupted data file, recreating...");
    }
    
    let mut file = std::fs::File::create(&file_path).unwrap();
    
    file.write_all(b"{}")
        .map_err(|e| format!("Failed to write to data file: {}", e))
        .unwrap();
}