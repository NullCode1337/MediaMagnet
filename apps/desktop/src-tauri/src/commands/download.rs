use mm_core::host::resolve_download_path;
use mm_core::DownloadRequest;
use tauri::State;

use crate::app::App;

#[tauri::command]
pub async fn downloader(
    app: State<'_, App>,
    url: String,
    download_id: String,
) -> Result<(), String> {
    let engine = app.engine.clone();
    let base_dir = resolve_download_path(&engine.settings(), &app.host);
    let request = DownloadRequest {
        id: download_id,
        url,
        backend: None,
        base_dir,
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
