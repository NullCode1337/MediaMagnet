use mm_core::host::resolve_download_path;
use mm_core::{Backend, DownloadRequest};
use serde_json::Value;
use tauri::State;

use crate::app::App;

#[tauri::command]
pub async fn downloader(
    app: State<'_, App>,
    url: String,
    download_id: String,
    backend: Option<Backend>,
    overrides: Option<Value>,
) -> Result<(), String> {
    let engine = app.engine.clone();
    if let Some(backend) = backend {
        engine
            .ensure_backend(backend)
            .await
            .map_err(|e| e.to_string())?;
    }
    let base_dir = resolve_download_path(&engine.settings(), &app.host);
    // Desktop downloads stay on disk. Output retention lives in `mm-server` and
    // only tracks jobs started through its API; do not reuse that path here.
    let request = DownloadRequest {
        id: download_id,
        url,
        backend,
        base_dir,
        settings: None,
        cookies: None,
        overrides,
    };
    tokio::spawn(async move {
        engine.run_job(request).await;
    });
    Ok(())
}

#[tauri::command]
pub async fn pause_download(app: State<'_, App>, download_id: String) -> Result<(), String> {
    app.engine
        .pause(&download_id)
        .await
        .map_err(|e| e.to_string())
}

/// `url` is part of the UI contract but cleanup now uses the engine's recorded URL.
#[tauri::command]
pub async fn cancel_download(
    app: State<'_, App>,
    download_id: String,
    _url: Option<String>,
) -> Result<(), String> {
    app.engine
        .cancel(&download_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cancel_all_downloads(app: State<'_, App>) -> Result<(), String> {
    app.engine.cancel_all().await.map_err(|e| e.to_string())
}
