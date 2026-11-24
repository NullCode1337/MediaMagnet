use mm_server::testing::{archive_file_name, file_route, valid_job_id};

#[test]
fn sandbox_policy_only_applies_to_api_file_routes() {
    assert!(file_route("/api/v1/files/a.mp4"));
    assert!(file_route("/api/v1/downloads/job-1/file"));
    assert!(!file_route("/files"));
    assert!(!file_route("/downloads/job-1/file"));
    assert!(!file_route("/api/v1/settings"));
    assert!(!file_route("/api/v1/downloads/profile"));
}

#[test]
fn job_ids_are_bounded_and_safe() {
    assert!(valid_job_id("job-1"));
    assert!(valid_job_id("6f3a0f4e-6c1a-4b3e-9e0e-0c9a2b7d1e55"));
    assert!(valid_job_id("srv-18f2a-0"));
    // Path-shaped values belong to the file picker, not the job table.
    assert!(!valid_job_id("file://abc/name.png"));
    assert!(!valid_job_id(""));
    assert!(!valid_job_id("job 1"));
    assert!(!valid_job_id("job/1"));
    assert!(!valid_job_id("job\n1"));
    assert!(!valid_job_id(&"a".repeat(129)));
}

#[test]
fn archive_names_use_the_source_host() {
    assert_eq!(
        archive_file_name("https://www.example.com/watch?v=1"),
        "example.com.zip"
    );
    assert_eq!(
        archive_file_name("https://media.example.org/a/b"),
        "media.example.org.zip"
    );
}

#[test]
fn archive_names_fall_back_without_a_host() {
    assert_eq!(archive_file_name("not a url"), "download.zip");
}
