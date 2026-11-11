use mm_core::policy::{is_denied, ArgTool, Policy};

#[test]
fn desktop_allows_everything() {
    let policy = Policy::desktop();
    let args = policy
        .filter_custom_args(ArgTool::YtDlp, "--exec echo {}")
        .unwrap();
    assert_eq!(args, vec!["--exec", "echo", "{}"]);
    assert!(policy.cookie_store);
}

#[test]
fn server_has_no_cookie_store() {
    assert!(!Policy::server().cookie_store);
    assert!(!Policy::server_with_custom_args().cookie_store);
}

#[test]
fn server_with_custom_args_still_filters() {
    let policy = Policy::server_with_custom_args();
    assert_eq!(
        policy
            .filter_custom_args(ArgTool::YtDlp, "--no-mtime")
            .unwrap(),
        vec!["--no-mtime"]
    );
    assert!(policy
        .filter_custom_args(ArgTool::YtDlp, "--paths /tmp")
        .is_err());
}

#[test]
fn server_refuses_custom_args_entirely() {
    let policy = Policy::server();
    assert!(policy
        .filter_custom_args(ArgTool::YtDlp, "--no-mtime")
        .is_err());
    assert!(policy
        .filter_custom_args(ArgTool::YtDlp, "   ")
        .unwrap()
        .is_empty());
}

#[test]
fn deny_list_catches_separate_equals_and_attached_forms() {
    let check = |tool, arg| is_denied(tool, arg);
    assert!(check(ArgTool::YtDlp, "--exec"));
    assert!(check(ArgTool::YtDlp, "--exec=rm -rf /"));
    assert!(check(ArgTool::YtDlp, "-o"));
    assert!(check(ArgTool::YtDlp, "-o/tmp/x"));
    assert!(check(ArgTool::YtDlp, "--paths"));
    assert!(check(ArgTool::YtDlp, "-P"));
    assert!(check(ArgTool::YtDlp, "--config-locations"));
    assert!(check(ArgTool::YtDlp, "--cookies-from-browser"));
    assert!(check(ArgTool::GalleryDl, "--postprocessor"));
    assert!(check(ArgTool::GalleryDl, "-D"));
    assert!(check(ArgTool::GalleryDl, "-c"));
    assert!(!check(ArgTool::YtDlp, "--no-mtime"));
    assert!(!check(ArgTool::YtDlp, "-f"));
    assert!(!check(ArgTool::GalleryDl, "--no-mtime"));
    assert!(!check(ArgTool::GalleryDl, "-R"));
}

#[test]
fn deny_list_covers_option_families() {
    let check = |tool, arg| is_denied(tool, arg);
    assert!(check(ArgTool::YtDlp, "--exec-before-download"));
    assert!(check(ArgTool::YtDlp, "--exec-before-download=id"));
    assert!(check(ArgTool::YtDlp, "--output-na-placeholder"));
    assert!(check(ArgTool::YtDlp, "--config-locations"));
    assert!(check(ArgTool::YtDlp, "--ffmpeg-location"));
    assert!(check(ArgTool::YtDlp, "--ffmpeg-location=/tmp/evil"));
    assert!(check(ArgTool::YtDlp, "--downloader-args"));
    assert!(check(ArgTool::SpotDl, "--ffmpeg"));
    assert!(check(ArgTool::GalleryDl, "--filter"));
    assert!(check(ArgTool::GalleryDl, "-f"));
    assert!(check(ArgTool::GalleryDl, "--image-filter"));
    assert!(!check(ArgTool::YtDlp, "--embed-thumbnail"));
    assert!(!check(ArgTool::GalleryDl, "--no-download"));
    assert!(!check(ArgTool::SpotDl, "--bitrate"));
}
