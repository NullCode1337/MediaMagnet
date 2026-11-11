use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use mm_server::testing::{FileIdentity, Retention};

fn temp_dir(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("mm-retention-{name}-{unique}"));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn retention(root: &Path, ttl: Duration) -> Retention {
    Retention::open(root.to_path_buf(), ttl, root.join("job-files.json"))
}

#[tokio::test]
async fn delivered_files_are_removed() {
    let root = temp_dir("delivered");
    std::fs::create_dir_all(root.join("set")).unwrap();
    std::fs::write(root.join("set/one.jpg"), b"one").unwrap();
    // Simulates a file the server did not produce, for example a desktop
    // download in a shared directory.
    std::fs::write(root.join("set/desktop.jpg"), b"keep").unwrap();

    let retention = retention(&root, Duration::from_secs(3600));
    retention.record("job", &["set/one.jpg".into()]);

    let file = tokio::fs::File::open(root.join("set/one.jpg"))
        .await
        .unwrap();
    let identity = FileIdentity::from_metadata(&file.metadata().await.unwrap());
    retention
        .delete_delivered("job", "set/one.jpg", identity)
        .await;

    assert!(!root.join("set/one.jpg").exists());
    assert!(root.join("set/desktop.jpg").exists());
    assert!(root.join("set").exists());
    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn expiry_removes_only_recorded_files() {
    let root = temp_dir("expiry");
    std::fs::write(root.join("clip.mp4"), b"clip").unwrap();
    std::fs::write(root.join("desktop.mp4"), b"keep").unwrap();

    let retention = retention(&root, Duration::from_millis(10));
    retention.record("job", &["clip.mp4".into()]);
    tokio::time::sleep(Duration::from_millis(100)).await;

    assert!(!root.join("clip.mp4").exists());
    assert!(root.join("desktop.mp4").exists());
    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn a_newer_job_keeps_ownership_of_the_same_path() {
    let root = temp_dir("ownership");
    std::fs::write(root.join("clip.mp4"), b"first").unwrap();

    let retention = retention(&root, Duration::from_secs(3600));
    retention.record("old", &["clip.mp4".into()]);
    retention.record("new", &["clip.mp4".into()]);

    let file = tokio::fs::File::open(root.join("clip.mp4")).await.unwrap();
    let identity = FileIdentity::from_metadata(&file.metadata().await.unwrap());
    retention
        .delete_delivered("old", "clip.mp4", identity)
        .await;
    assert!(root.join("clip.mp4").exists());

    retention
        .delete_delivered("new", "clip.mp4", identity)
        .await;
    assert!(!root.join("clip.mp4").exists());
    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn a_replaced_file_is_not_deleted() {
    let root = temp_dir("replaced");
    let path = root.join("clip.mp4");
    std::fs::write(&path, b"first").unwrap();

    let retention = retention(&root, Duration::from_secs(3600));
    retention.record("job", &["clip.mp4".into()]);

    let file = tokio::fs::File::open(&path).await.unwrap();
    let identity = FileIdentity::from_metadata(&file.metadata().await.unwrap());
    drop(file);

    // Same length, different inode: a replacement, not the served file.
    let replacement = root.join("replacement.mp4");
    std::fs::write(&replacement, b"other").unwrap();
    std::fs::rename(&replacement, &path).unwrap();

    retention
        .delete_delivered("job", "clip.mp4", identity)
        .await;
    assert!(path.exists());
    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn restart_removes_previous_outputs() {
    let root = temp_dir("restart");
    let manifest = root.join("job-files.json");
    std::fs::write(root.join("clip.mp4"), b"clip").unwrap();
    std::fs::write(
        &manifest,
        serde_json::to_vec(&HashMap::from([(
            "clip.mp4".to_string(),
            "job".to_string(),
        )]))
        .unwrap(),
    )
    .unwrap();

    let _retention = Retention::open(root.clone(), Duration::from_secs(3600), manifest);
    assert!(!root.join("clip.mp4").exists());
    let _ = std::fs::remove_dir_all(&root);
}
