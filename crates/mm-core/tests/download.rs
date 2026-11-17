use mm_core::download::{
    contains_separator, long_dir, matching_cookie, mtime_flag, recode_args, too_long, url_host_is,
    Backend, JobCookie,
};

fn cookie(domain: &str, content: &str) -> JobCookie {
    JobCookie {
        domain: domain.into(),
        content: content.into(),
    }
}

#[test]
fn detects_enametoolong_errors() {
    assert!(too_long(
        "ERROR: unable to open for writing: [Errno 36] File name too long: '/tmp/a'"
    ));
    assert!(too_long("OSError: [Errno 36] File name too long"));
    assert!(too_long("os error 36"));
    assert!(too_long("ENAMETOOLONG"));
    assert!(!too_long("HTTP Error 404: Not Found"));
}

#[test]
fn classifies_long_path_components() {
    let directory = "OSError: [Errno 36] File name too long: './nhentai/697736 [Torinuki] A very long gallery directory title that goes on and on and on/nhentai_697736_009.webp.part'";
    assert!(long_dir(directory), "long folder");
    let filename = "OSError: [Errno 36] File name too long: './nhentai/697736/this_is_a_very_long_individual_file_name_that_keeps_going.webp.part'";
    assert!(!long_dir(filename), "long file");

    let directory_only = "OSError: [Errno 36] File name too long: './nhentai/697736 [Torinuki] A very long gallery directory title that goes on and on and on/'";
    assert!(long_dir(directory_only), "long folder ending in separator");
    assert!(!long_dir("no path here"));
}

#[test]
fn host_checks_ignore_paths_and_queries() {
    assert!(url_host_is(
        "https://www.youtube.com/watch?v=1",
        "youtube.com"
    ));
    assert!(url_host_is("https://music.youtube.com/", "youtube.com"));
    assert!(url_host_is("https://youtu.be/abc", "youtu.be"));
    assert!(url_host_is(
        "https://open.spotify.com/track/1",
        "spotify.com"
    ));

    assert!(!url_host_is(
        "https://example.com/?next=youtube.com",
        "youtube.com"
    ));
    assert!(!url_host_is("https://notyoutube.com/x", "youtube.com"));
    assert!(!url_host_is("not a url", "youtube.com"));
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
