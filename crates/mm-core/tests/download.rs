use mm_core::download::{
    contains_separator, matching_cookie, mtime_flag, recode_args, Backend, JobCookie,
};

fn cookie(domain: &str, content: &str) -> JobCookie {
    JobCookie {
        domain: domain.into(),
        content: content.into(),
    }
}

#[test]
fn job_cookie_matches_the_most_specific_domain() {
    let cookies = vec![cookie("youtube", "a"), cookie("youtube.com", "b")];
    assert_eq!(
        matching_cookie(&cookies, "https://WWW.YOUTUBE.COM/watch")
            .unwrap()
            .content,
        "b"
    );
    assert_eq!(
        matching_cookie(&cookies, "https://youtube.de/1")
            .unwrap()
            .content,
        "a"
    );
    assert!(matching_cookie(&cookies, "https://vimeo.com/1").is_none());
    assert!(matching_cookie(&cookies, "https://evil.com/?x=youtube").is_none());
}

#[test]
fn recode_accepts_known_formats_and_drops_the_rest() {
    assert_eq!(recode_args("MP4"), Some(&["--recode-video", "mp4"][..]));
    assert_eq!(
        recode_args(" mp3 "),
        Some(&["-x", "--audio-format", "mp3", "--audio-quality", "0"][..])
    );
    assert!(recode_args("").is_none());
    assert!(recode_args("avi").is_none());
    assert!(recode_args("--exec").is_none());
}

#[test]
fn empty_domains_never_match() {
    assert!(matching_cookie(&[cookie("  ", "x")], "https://example.com").is_none());
}

#[test]
fn mtime_flags_follow_the_setting() {
    let flags = |preserve| {
        (
            mtime_flag(Backend::YtDlp, preserve),
            mtime_flag(Backend::GalleryDl, preserve),
        )
    };

    // Default: download time. yt-dlp already does that; gallery-dl needs `--no-mtime`.
    assert_eq!(flags(false), (None, Some("--no-mtime")));
    // Original date: the mirror image.
    assert_eq!(flags(true), (Some("--mtime"), None));
    // spotdl has no equivalent option in either mode.
    assert_eq!(mtime_flag(Backend::SpotDl, false), None);
    assert_eq!(mtime_flag(Backend::SpotDl, true), None);
}

#[test]
fn output_templates_cannot_escape_the_job_directory() {
    assert!(!contains_separator("%(title)s.%(ext)s"));
    assert!(!contains_separator(
        "%(uploader)s - %(title)s [%(id)s].%(ext)s"
    ));
    assert!(contains_separator("../../etc/cron.d/x"));
    assert!(contains_separator("%(title)s/%(id)s.%(ext)s"));
    assert!(contains_separator("..\\evil.%(ext)s"));
    assert!(contains_separator("C:evil.%(ext)s"));
}
