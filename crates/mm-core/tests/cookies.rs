use std::path::PathBuf;

use mm_core::cookies::{
    delete_cookie, host_matches, list_cookies, match_cookie, netscape_from_json, save_cookie,
    validate_domain, CookieInput,
};

#[test]
fn rejects_traversal_domains() {
    for bad in ["..", "../../etc/passwd", "a/b", "a\\b", "", "  ", "a b"] {
        assert!(validate_domain(bad).is_err(), "{bad:?} accepted");
    }
    assert_eq!(validate_domain("youtube").unwrap(), "youtube");
    assert_eq!(validate_domain("google.com").unwrap(), "google.com");
}

#[test]
fn save_and_match_content() {
    let dir = std::env::temp_dir().join(format!("mm-cookies-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let content = "# Netscape HTTP Cookie File\n.youtube.com\tTRUE\t/\tTRUE\t0\tname\tvalue\n";
    save_cookie(&dir, "youtube", &CookieInput::Content(content.to_string())).unwrap();
    assert!(list_cookies(&dir).unwrap().contains_key("youtube"));
    let matched = match_cookie(&dir, "https://WWW.youtube.com/watch").unwrap();
    assert_eq!(matched.file_name().unwrap(), "youtube.txt");
    assert!(match_cookie(&dir, "https://vimeo.com/1").is_none());
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn cookies_never_follow_substrings_of_the_url() {
    assert!(host_matches("www.youtube.com", "youtube"));
    assert!(host_matches("youtube.de", "youtube"));
    assert!(host_matches("music.youtube.com", "youtube.com"));
    assert!(!host_matches("notyoutube.com", "youtube"));
    assert!(!host_matches("youtube.evil.com", "youtube"));
    assert!(!host_matches("evil.com", "youtube"));
    assert!(!host_matches("google.com.evil.com", "google.com"));
    assert!(!host_matches("youtube", ""));
    assert!(!host_matches("youtube", "../youtube"));
}

#[test]
fn longest_matching_profile_wins() {
    let dir = std::env::temp_dir().join(format!("mm-cookies-long-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for domain in ["youtube", "youtube.com"] {
        std::fs::write(
            dir.join(format!("{domain}.txt")),
            "# Netscape HTTP Cookie File\n.youtube.com\tTRUE\t/\tTRUE\t0\ta\tb\n",
        )
        .unwrap();
    }
    let matched = match_cookie(&dir, "https://www.youtube.com/watch").unwrap();
    assert_eq!(matched.file_name().unwrap(), "youtube.com.txt");
    let _ = std::fs::remove_dir_all(&dir);
}

#[cfg(unix)]
#[test]
fn stored_cookies_are_owner_only() {
    use std::os::unix::fs::PermissionsExt;

    let dir = std::env::temp_dir().join(format!("mm-cookies-mode-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let content = "# Netscape HTTP Cookie File\n.youtube.com\tTRUE\t/\tTRUE\t0\ta\tb\n";
    let path = save_cookie(&dir, "youtube", &CookieInput::Content(content.to_string()))
        .map(PathBuf::from)
        .unwrap();
    let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn delete_outside_store_is_refused() {
    let dir = std::env::temp_dir().join(format!("mm-cookies-del-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let outside = std::env::temp_dir().join(format!("mm-cookies-outside-{}", std::process::id()));
    std::fs::write(&outside, "x").unwrap();
    assert!(delete_cookie(&dir, &outside).is_err());
    assert!(outside.exists());
    std::fs::remove_file(&outside).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn json_converts_to_netscape() {
    let json = r#"[{"domain":".example.com","path":"/","secure":true,"expirationDate":1700000000,"name":"a","value":"b","httpOnly":true}]"#;
    let out = netscape_from_json(json).unwrap();
    assert!(out.starts_with("# Netscape HTTP Cookie File"));
    assert!(out.contains("#HttpOnly_.example.com\tTRUE\t/\tTRUE\t1700000000\ta\tb"));
}
