use std::io::{Read, Write};
use tauri::Manager;

#[tauri::command]
pub fn save_to_disk(app: tauri::AppHandle, links: Vec<String>) {
    let path = app.path().app_data_dir().unwrap().join("links.json");

    let old = if path.exists() {
        let mut file = std::fs::File::open(&path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        contents
    } else {
        String::from("[]")
    };

    let mut file_links: Vec<String> = if old.trim().is_empty() {
        Vec::new()
    } else {
        serde_json::from_str(&old).unwrap()
    };

    let mut existing_links: std::collections::HashSet<String> = file_links.iter().cloned().collect();
    
    let links_dedup: Vec<String> = links
        .into_iter()
        .filter(|link| {
            if existing_links.contains(link) {
                false 
            } else {
                existing_links.insert(link.clone());
                true
            }
        })
        .collect();

    file_links.extend(links_dedup);

    let json_data = serde_json::to_string_pretty(&file_links).unwrap();

    std::fs::File::create(&path)
        .unwrap()
        .write_all(json_data.as_bytes())
        .unwrap();
}