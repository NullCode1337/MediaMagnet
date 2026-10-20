use std::collections::BTreeMap;
use std::path::Path;

use mm_core::cookies::{
    clear_cookies as clear_cookie_store, delete_cookie as delete_cookie_file, list_cookies,
    save_cookie as save_cookie_file, CookieInput,
};
use tauri::State;

use crate::app::App;

#[tauri::command]
pub async fn save_cookie(
    app: State<'_, App>,
    domain: String,
    input: CookieInput,
) -> Result<String, String> {
    save_cookie_file(&app.engine.cookies_dir(), &domain, &input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_cookies(app: State<'_, App>) -> Result<BTreeMap<String, String>, String> {
    list_cookies(&app.engine.cookies_dir()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_cookies(app: State<'_, App>) {
    let deleted = clear_cookie_store(&app.engine.cookies_dir()).unwrap_or(0);
    app.engine
        .notify(format!("Successfully cleared {} cookie files", deleted));
}

#[tauri::command]
pub fn delete_cookie(app: State<'_, App>, path: String) {
    match delete_cookie_file(&app.engine.cookies_dir(), Path::new(&path)) {
        Ok(()) => app.engine.notify("Cookie deleted!"),
        Err(e) => app.engine.notify(e.to_string()),
    }
}
