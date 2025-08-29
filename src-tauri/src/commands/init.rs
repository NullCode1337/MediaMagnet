use std::io::{Read, Write};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager};

#[derive(Clone, Serialize, Deserialize)]
struct LinkEvent {
    links: Vec<String>,
    message: String,
}

#[tauri::command]
pub fn check_links(app: tauri::AppHandle) {
    let path = app.path().app_data_dir()
        .unwrap()
        .join("links.json");

    let mut file = std::fs::File::open(path).unwrap();
    let mut contents = String::new();
    let _ = file.read_to_string(&mut contents);

    let links: Vec<String> = serde_json::from_str(&contents)
        .unwrap();  
    
    if links.is_empty() {
        let event = LinkEvent {
            links: vec![],
            message: "Nothing".to_string(),
        };
        
        app.emit("link-event", event).unwrap();
    } else {
        let event = LinkEvent {
            links: links.clone(),
            message: format!("Found {} links", links.len()),
        };
        
        app.emit("link-event", event).unwrap();
    }
}

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
    
    file.write_all(b"[]")
        .map_err(|e| format!("Failed to write to data file: {}", e))
        .unwrap();
}