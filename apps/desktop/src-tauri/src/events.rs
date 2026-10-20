use mm_core::events::{CoreEvent, EventSink, NotificationLevel};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

/// Forwards core events onto the app's IPC channels.
pub struct TauriEventSink {
    app: AppHandle,
}

impl TauriEventSink {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

#[derive(Serialize, Clone)]
struct IdPayload {
    id: String,
}

#[derive(Serialize, Clone)]
struct IdValuePayload<T> {
    id: String,
    value: T,
}

#[derive(Serialize, Clone)]
struct NotificationPayload {
    level: NotificationLevel,
    message: String,
}

impl EventSink for TauriEventSink {
    fn emit(&self, event: CoreEvent) {
        let app = &self.app;
        match event {
            CoreEvent::DownloadStarted { id } => {
                let _ = app.emit("download-started", IdPayload { id });
            }
            CoreEvent::DownloadFinished { id } => {
                let _ = app.emit("download-finished", IdPayload { id });
            }
            CoreEvent::DownloadProgress { id, value } => {
                let _ = app.emit("download-progress", IdValuePayload { id, value });
            }
            CoreEvent::DownloadStatus { id, value } => {
                let _ = app.emit("download-status", IdValuePayload { id, value });
            }
            CoreEvent::DownloadError { id, value } => {
                let _ = app.emit("download-error", IdValuePayload { id, value });
            }
            CoreEvent::DownloadState { id, state } => {
                let state = match state {
                    mm_core::DownloadState::Paused => "paused",
                    mm_core::DownloadState::Cancelled => "cancelled",
                    mm_core::DownloadState::Failed => "failed",
                    mm_core::DownloadState::Finished => "finished",
                };
                let _ = app.emit("download-state", IdValuePayload { id, value: state });
            }
            CoreEvent::Notification { level, message } => {
                let _ = app.emit("notification", NotificationPayload { level, message });
            }
        }
    }
}
