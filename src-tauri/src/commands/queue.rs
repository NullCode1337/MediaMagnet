use std::io::Write;
use tauri::Manager;

#[tauri::command]
pub fn overwrite_json(app: tauri::AppHandle, links: Vec<String>) {
    let path = app.path().app_data_dir().unwrap().join("links.json");
    
    let mut unique_links = Vec::new();
    let mut seen_links = std::collections::HashSet::new();
    
    for link in links {
        if !seen_links.contains(&link) {
            seen_links.insert(link.clone());
            unique_links.push(link);
        }
    }
    
    let json_data = serde_json::to_string_pretty(&unique_links).unwrap();

    std::fs::File::create(&path)
        .unwrap()
        .write_all(json_data.as_bytes())
        .unwrap();
}