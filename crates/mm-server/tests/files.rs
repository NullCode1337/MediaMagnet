use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::body::to_bytes;
use axum::http::HeaderMap;

use mm_server::testing::{
    encode_path, is_active_content, serve, DeleteAfter, Disposition, Retention,
};

#[test]
fn encoding_keeps_slashes_and_escapes_others() {
    assert_eq!(encode_path("a/b c.mp4"), "a/b%20c.mp4");
    assert_eq!(encode_path("ünï.txt"), "%C3%BCn%C3%AF.txt");
}

#[test]
fn active_content_is_never_served_inline() {
    for name in [
        "a.html", "a.HTM", "a.xhtml", "a.svg", "a.svgz", "a.xml", "a.xsl", "a.js", "a.mjs",
        "a.css", "a.wasm",
    ] {
        assert!(
            is_active_content(Path::new(name)),
            "{name} should be active"
        );
    }
    for name in ["a.mp4", "a.jpg", "a.png", "a.txt", "a.zip", "a.pdf"] {
        assert!(
            !is_active_content(Path::new(name)),
            "{name} should be inert"
        );
    }
}

#[tokio::test]
async fn a_fully_streamed_body_deletes_the_output() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("mm-files-cleanup-{unique}"));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("clip.mp4");
    std::fs::write(&path, b"hello").unwrap();

    let retention = Retention::open(
        root.clone(),
        Duration::from_secs(3600),
        root.join("job-files.json"),
    );
    retention.record("job", &["clip.mp4".into()]);

    let response = serve(
        &root,
        "clip.mp4",
        HeaderMap::new(),
        Disposition::Attachment,
        Some(DeleteAfter {
            retention,
            job_id: "job".into(),
        }),
    )
    .await;
    let body = to_bytes(response.into_body(), 16).await.unwrap();
    assert_eq!(&body[..], b"hello");

    // Deletion happens in a task spawned from the stream.
    tokio::time::timeout(Duration::from_secs(2), async {
        while path.exists() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert!(!path.exists());
    let _ = std::fs::remove_dir_all(&root);
}
