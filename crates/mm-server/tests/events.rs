use mm_core::{CoreEvent, DownloadState, NotificationLevel};
use mm_server::testing::to_transport_event;

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
fn finished_event_carries_output_files() {
    let event = to_transport_event(CoreEvent::DownloadFinished {
        id: "a".into(),
        files: vec!["clip.mp4".into(), "set/one.jpg".into()],
    });
    assert_eq!(event["type"], "download-finished");
    assert_eq!(event["id"], "a");
    assert_eq!(event["files"][0], "clip.mp4");
    assert_eq!(event["files"][1], "set/one.jpg");
}

#[test]
fn notification_keeps_its_own_shape() {
    let event = to_transport_event(CoreEvent::Notification {
        level: NotificationLevel::Warning,
        message: "hi".into(),
    });
    assert_eq!(event["type"], "notification");
    assert_eq!(event["level"], "warning");
    assert_eq!(event["message"], "hi");
}
