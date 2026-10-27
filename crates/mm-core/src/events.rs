use std::sync::Arc;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DownloadState {
    Paused,
    Cancelled,
    Failed,
    Finished,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum CoreEvent {
    DownloadStarted {
        id: String,
    },
    DownloadState {
        id: String,
        state: DownloadState,
    },
    DownloadProgress {
        id: String,
        value: f64,
    },
    DownloadStatus {
        id: String,
        value: String,
    },
    DownloadError {
        id: String,
        value: String,
    },
    DownloadFinished {
        id: String,
        files: Vec<String>,
    },
    Notification {
        level: NotificationLevel,
        message: String,
    },
}

pub trait EventSink: Send + Sync {
    fn emit(&self, event: CoreEvent);
}

pub type SharedEventSink = Arc<dyn EventSink>;

#[derive(Debug, Default)]
pub struct NullEventSink;

impl EventSink for NullEventSink {
    fn emit(&self, _event: CoreEvent) {}
}
