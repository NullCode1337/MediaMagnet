use std::sync::Arc;

use mm_core::events::{CoreEvent, EventSink, SharedEventSink};
use serde_json::{json, Value};
use tokio::sync::broadcast;

const CHANNEL_CAPACITY: usize = 512;

/// Fans core events out to every connected SSE client.
#[derive(Clone)]
pub struct EventHub {
    tx: broadcast::Sender<Value>,
}

impl EventHub {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(CHANNEL_CAPACITY);
        Self { tx }
    }

    pub fn sink(&self) -> SharedEventSink {
        Arc::new(HubSink { hub: self.clone() })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Value> {
        self.tx.subscribe()
    }

    fn publish(&self, event: CoreEvent) {
        // `send` only fails when nobody is listening, which is not an error here.
        let _ = self.tx.send(to_transport_event(event));
    }
}

impl Default for EventHub {
    fn default() -> Self {
        Self::new()
    }
}

struct HubSink {
    hub: EventHub,
}

impl EventSink for HubSink {
    fn emit(&self, event: CoreEvent) {
        self.hub.publish(event);
    }
}

/// The wire shape the UI expects. Differs from `CoreEvent` in one place:
/// `download-state` carries its value under `value`, like every other id event.
pub fn to_transport_event(event: CoreEvent) -> Value {
    match event {
        CoreEvent::DownloadStarted { id } => json!({ "type": "download-started", "id": id }),
        CoreEvent::DownloadFinished { id } => json!({ "type": "download-finished", "id": id }),
        CoreEvent::DownloadProgress { id, value } => {
            json!({ "type": "download-progress", "id": id, "value": value })
        }
        CoreEvent::DownloadStatus { id, value } => {
            json!({ "type": "download-status", "id": id, "value": value })
        }
        CoreEvent::DownloadError { id, value } => {
            json!({ "type": "download-error", "id": id, "value": value })
        }
        CoreEvent::DownloadState { id, state } => json!({
            "type": "download-state",
            "id": id,
            "value": serde_json::to_value(state).unwrap_or(Value::Null),
        }),
        CoreEvent::Notification { level, message } => {
            json!({ "type": "notification", "level": level, "message": message })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_core::DownloadState;

    #[test]
    fn state_event_uses_value_key() {
        let event = to_transport_event(CoreEvent::DownloadState {
            id: "a".into(),
            state: DownloadState::Paused,
        });
        assert_eq!(event["type"], "download-state");
        assert_eq!(event["id"], "a");
        assert_eq!(event["value"], "paused");
        assert!(event.get("state").is_none());
    }

    #[test]
    fn notification_keeps_its_own_shape() {
        let event = to_transport_event(CoreEvent::Notification {
            level: mm_core::NotificationLevel::Warning,
            message: "hi".into(),
        });
        assert_eq!(event["type"], "notification");
        assert_eq!(event["level"], "warning");
        assert_eq!(event["message"], "hi");
    }
}
