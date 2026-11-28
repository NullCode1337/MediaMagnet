use mm_core::download::{
    contains_separator, is_youtube, long_dir, matching_cookie, mtime_flag, recode_args, too_long,
    url_host_is, with_warnings, ytdlp_challenge, ytdlp_cookie_retry, Backend, JobCookie,
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
fn youtube_challenge_failures_say_what_to_update() {
    let error = "ERROR: [youtube] x: Requested format is not available";
    let hinted = ytdlp_challenge(error.to_string(), true);
    assert!(hinted.contains("Requested format is not available"));
    assert!(hinted.contains("yt-dlp-ejs"));
    assert_eq!(ytdlp_challenge(error.to_string(), false), error);
    assert!(!ytdlp_challenge(String::new(), true).is_empty());
}

#[test]
fn errors_carry_every_collected_warning() {
    let warnings = vec![
        "WARNING: [youtube] [jsc] n challenge solving failed".to_string(),
        "WARNING: Only images are available for download".to_string(),
    ];
    let error = with_warnings(
        "ERROR: [youtube] x: Requested format is not available".into(),
        &warnings,
    );
    assert!(error.contains("Requested format is not available"));
    assert!(error.contains("n challenge solving failed"));
    assert!(error.contains("Only images are available"));

    assert_eq!(with_warnings("ERROR: x".into(), &[]), "ERROR: x");
    assert_eq!(
        with_warnings(String::new(), &warnings),
        "",
        "no error line keeps the exit-status message"
    );
}

#[test]
fn only_cookie_related_errors_are_retried_without_cookies() {
    assert!(ytdlp_cookie_retry(
        "ERROR: [youtube] x: Sign in to confirm you're not a bot"
    ));
    assert!(ytdlp_cookie_retry("HTTP Error 403: Forbidden"));
    assert!(ytdlp_cookie_retry("Unable to load cookies"));
    assert!(!ytdlp_cookie_retry(
        "ERROR: [youtube] x: Requested format is not available. Use --list-formats"
    ));
    assert!(!ytdlp_cookie_retry("ERROR: Unsupported URL"));
    assert!(!ytdlp_cookie_retry(
        "ERROR: [youtube] x: Requested format is not available\nWarnings:\nWARNING: bad cookies were ignored"
    ));
}

#[test]
fn youtube_hosts_are_recognized() {
    assert!(is_youtube("https://www.youtube.com/watch?v=1"));
    assert!(is_youtube("https://music.youtube.com/watch?v=1"));
    assert!(is_youtube("https://youtu.be/abc"));
    assert!(is_youtube("https://www.youtube-nocookie.com/embed/abc"));
    assert!(!is_youtube("https://example.com/?next=youtube.com"));
    assert!(!is_youtube("https://notyoutube.com/x"));
    assert!(!is_youtube("not a url"));
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
