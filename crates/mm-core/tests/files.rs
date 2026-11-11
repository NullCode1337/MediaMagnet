use std::path::PathBuf;

use mm_core::files::{list_dir, parse_range, safe_join, ByteRange, EntryKind};

fn root() -> PathBuf {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static N: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "mm-files-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(dir.join("sub")).unwrap();
    std::fs::write(dir.join("a.txt"), b"hello world").unwrap();
    std::fs::write(dir.join("sub/b.txt"), b"b").unwrap();
    dir
}

#[test]
fn safe_join_allows_inside_paths() {
    let dir = root();
    let root = dir.canonicalize().unwrap();
    assert_eq!(safe_join(&root, "a.txt").unwrap(), root.join("a.txt"));
    assert_eq!(
        safe_join(&root, "sub/b.txt").unwrap(),
        root.join("sub/b.txt")
    );
    assert_eq!(safe_join(&root, "").unwrap(), root);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn safe_join_has_no_trailing_separator() {
    let dir = root();
    let root = dir.canonicalize().unwrap();
    for relative in ["a.txt", "sub", "sub/b.txt"] {
        let path = safe_join(&root, relative).unwrap();
        let text = path.to_string_lossy();
        assert!(!text.ends_with('/'), "{relative} -> {text}");
        assert!(path.is_file() || path.is_dir(), "{relative} -> {text}");
    }
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn safe_join_rejects_traversal() {
    let dir = root();
    let root = dir.canonicalize().unwrap();
    for bad in [
        "../etc/passwd",
        "sub/../../etc/passwd",
        "/etc/passwd",
        "..\\..\\windows",
        "sub/../..",
    ] {
        assert!(safe_join(&root, bad).is_err(), "{bad:?} accepted");
    }
    std::fs::remove_dir_all(&dir).unwrap();
}

#[cfg(unix)]
#[test]
fn safe_join_rejects_symlink_escape() {
    let dir = root();
    let root = dir.canonicalize().unwrap();
    let outside = std::env::temp_dir().join(format!("mm-outside-{}", std::process::id()));
    std::fs::write(&outside, b"secret").unwrap();
    std::os::unix::fs::symlink(&outside, root.join("link")).unwrap();
    assert!(safe_join(&root, "link").is_err());
    assert!(safe_join(&root, "link/child").is_err());
    std::fs::remove_file(&outside).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn list_dir_reports_directories_first() {
    let dir = root();
    let entries = list_dir(&dir, "").unwrap();
    assert_eq!(entries[0].kind, EntryKind::Directory);
    assert_eq!(entries[0].path, "sub");
    assert_eq!(entries[1].path, "a.txt");
    assert_eq!(entries[1].size, 11);
    let nested = list_dir(&dir, "sub").unwrap();
    assert_eq!(nested[0].path, "sub/b.txt");
    std::fs::remove_dir_all(&dir).unwrap();
}

#[cfg(unix)]
#[test]
fn list_dir_skips_symlinks() {
    let dir = root();
    let outside = dir.join("outside-secret");
    std::fs::write(&outside, b"secret").unwrap();
    std::os::unix::fs::symlink(&outside, dir.join("link")).unwrap();

    let entries = list_dir(&dir, "").unwrap();
    assert!(entries.iter().all(|entry| entry.name != "link"));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn range_parsing() {
    assert_eq!(
        parse_range("bytes=0-4", 11).unwrap(),
        Some(ByteRange { start: 0, end: 4 })
    );
    assert_eq!(
        parse_range("bytes=6-", 11).unwrap(),
        Some(ByteRange { start: 6, end: 10 })
    );
    assert_eq!(
        parse_range("bytes=-5", 11).unwrap(),
        Some(ByteRange { start: 6, end: 10 })
    );
    assert_eq!(
        parse_range("bytes=5-100", 11).unwrap(),
        Some(ByteRange { start: 5, end: 10 })
    );
    assert!(parse_range("bytes=11-", 11).is_err());
    assert!(parse_range("bytes=0-4,6-9", 11).is_err());
    assert!(parse_range("bytes=-0", 11).is_err());
    assert!(parse_range("items=0-4", 11).unwrap().is_none());
    assert!(parse_range("bytes=0-4", 0).is_err());
}
