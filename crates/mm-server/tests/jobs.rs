use std::path::PathBuf;

use mm_core::DownloadRequest;
use mm_server::testing::{Jobs, RETAINED_JOBS};

fn request(id: &str) -> DownloadRequest {
    DownloadRequest {
        id: id.to_string(),
        url: "https://example.com/a".into(),
        backend: None,
        base_dir: PathBuf::from("/tmp"),
        settings: None,
        cookies: None,
        overrides: None,
    }
}

#[test]
fn running_ids_cannot_be_reused_but_finished_ids_can() {
    let jobs = Jobs::default();
    assert!(jobs.insert(request("a")));
    assert!(!jobs.insert(request("a")));

    jobs.set_state("a", "finished");
    assert!(jobs.insert(request("a")));
    assert_eq!(jobs.summary("a").unwrap().state, "running");
}

#[test]
fn only_paused_jobs_resume() {
    let jobs = Jobs::default();
    jobs.insert(request("a"));
    assert!(jobs.take_for_resume("a").is_none());

    jobs.set_state("a", "paused");
    assert!(jobs.take_for_resume("a").is_some());
    assert_eq!(jobs.summary("a").unwrap().state, "running");
}

#[test]
fn finished_jobs_report_their_files() {
    let jobs = Jobs::default();
    jobs.insert(request("a"));
    jobs.set_finished("a", vec!["clip.mp4".into()]);

    let summary = jobs.summary("a").unwrap();
    assert_eq!(summary.state, "finished");
    assert_eq!(summary.files, vec!["clip.mp4".to_string()]);
}

#[test]
fn resuming_forgets_previous_output_files() {
    let jobs = Jobs::default();
    jobs.insert(request("a"));
    jobs.set_finished("a", vec!["clip.mp4".into()]);
    jobs.set_state("a", "paused");
    assert!(jobs.take_for_resume("a").is_some());
    assert!(jobs.summary("a").unwrap().files.is_empty());
}

#[test]
fn unknown_states_are_ignored() {
    let jobs = Jobs::default();
    jobs.insert(request("a"));
    jobs.set_state("a", "bogus");
    assert_eq!(jobs.summary("a").unwrap().state, "running");
}

#[test]
fn list_is_ordered_by_creation() {
    let jobs = Jobs::default();
    jobs.insert(request("b"));
    jobs.insert(request("a"));
    let listed: Vec<String> = jobs.list().into_iter().map(|job| job.id).collect();
    assert_eq!(listed.len(), 2);
    assert!(listed.contains(&"a".to_string()));
    assert!(listed.contains(&"b".to_string()));
}

#[test]
fn old_finished_jobs_are_pruned() {
    let jobs = Jobs::default();
    for index in 0..(RETAINED_JOBS + 5) {
        let id = format!("job-{index}");
        jobs.insert(request(&id));
        jobs.set_state(&id, "finished");
    }
    assert!(jobs.list().len() <= RETAINED_JOBS);
    assert!(jobs.summary("job-0").is_none());
    assert!(jobs
        .summary(&format!("job-{}", RETAINED_JOBS + 4))
        .is_some());
}

#[test]
fn active_count_tracks_queued_and_paused_jobs() {
    let jobs = Jobs::default();
    jobs.insert(request("a"));
    jobs.insert(request("b"));
    assert_eq!(jobs.active_count(), 2);
    jobs.set_state("a", "finished");
    jobs.set_state("b", "paused");
    assert_eq!(jobs.active_count(), 1);
    jobs.set_state("b", "cancelled");
    assert_eq!(jobs.active_count(), 0);
}
