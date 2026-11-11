//! In-memory job records for the API.
//!
//! Core removes a job from its process table when it pauses or finishes, so the
//! server keeps the request and the last state here. This is also what makes
//! `GET /downloads` and resume possible. Like core, nothing survives a restart.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use mm_core::DownloadRequest;
use serde::Serialize;

#[doc(hidden)]
pub const RETAINED_JOBS: usize = 100;

#[derive(Clone, Serialize)]
pub struct JobSummary {
    pub id: String,
    pub url: String,
    pub state: String,
    /// Output files relative to the download root; filled when the job finishes.
    pub files: Vec<String>,
    /// Unix time in milliseconds; used to order the list.
    pub created_at_ms: u64,
}

struct JobRecord {
    request: DownloadRequest,
    state: String,
    files: Vec<String>,
    created_at_ms: u64,
}

#[derive(Default)]
pub struct Jobs {
    records: Mutex<HashMap<String, JobRecord>>,
}

impl Jobs {
    /// Returns false when the id belongs to a running or paused job. Finished
    /// and cancelled ids may be reused, matching the old "ids come from the
    /// client" behaviour.
    pub fn insert(&self, request: DownloadRequest) -> bool {
        let Ok(mut records) = self.records.lock() else {
            return false;
        };
        if let Some(existing) = records.get(&request.id) {
            if existing.state == "running" || existing.state == "paused" {
                return false;
            }
        }
        records.insert(
            request.id.clone(),
            JobRecord {
                request,
                state: "running".to_string(),
                files: Vec::new(),
                created_at_ms: now_ms(),
            },
        );
        prune(&mut records);
        true
    }

    pub fn set_state(&self, id: &str, state: &str) {
        if !matches!(
            state,
            "running" | "paused" | "cancelled" | "failed" | "finished"
        ) {
            return;
        }
        if let Ok(mut records) = self.records.lock() {
            if let Some(record) = records.get_mut(id) {
                record.state = state.to_string();
            }
        }
    }

    /// Records completion together with the files the job produced.
    pub fn set_finished(&self, id: &str, files: Vec<String>) {
        if let Ok(mut records) = self.records.lock() {
            if let Some(record) = records.get_mut(id) {
                record.state = "finished".to_string();
                record.files = files;
            }
        }
    }

    pub fn summary(&self, id: &str) -> Option<JobSummary> {
        let records = self.records.lock().ok()?;
        records.get(id).map(summarize)
    }

    /// Jobs that occupy the queue: waiting for a permit, running, or paused.
    pub fn active_count(&self) -> usize {
        let Ok(records) = self.records.lock() else {
            return 0;
        };
        records
            .values()
            .filter(|record| matches!(record.state.as_str(), "running" | "paused"))
            .count()
    }

    pub fn list(&self) -> Vec<JobSummary> {
        let Ok(records) = self.records.lock() else {
            return Vec::new();
        };
        let mut jobs: Vec<JobSummary> = records.values().map(summarize).collect();
        jobs.sort_by(|a, b| {
            a.created_at_ms
                .cmp(&b.created_at_ms)
                .then_with(|| a.id.cmp(&b.id))
        });
        jobs
    }

    /// Marks a paused job running and returns the request to replay.
    pub fn take_for_resume(&self, id: &str) -> Option<DownloadRequest> {
        let mut records = self.records.lock().ok()?;
        let record = records.get_mut(id)?;
        if record.state != "paused" {
            return None;
        }
        record.state = "running".to_string();
        record.files.clear();
        Some(record.request.clone())
    }
}

fn summarize(record: &JobRecord) -> JobSummary {
    JobSummary {
        id: record.request.id.clone(),
        url: record.request.url.clone(),
        state: record.state.clone(),
        files: record.files.clone(),
        created_at_ms: record.created_at_ms,
    }
}

fn prune(records: &mut HashMap<String, JobRecord>) {
    if records.len() <= RETAINED_JOBS {
        return;
    }
    let mut finished: Vec<(u64, String)> = records
        .iter()
        .filter(|(_, record)| matches!(record.state.as_str(), "cancelled" | "failed" | "finished"))
        .map(|(id, record)| (record.created_at_ms, id.clone()))
        .collect();
    finished.sort();
    for (_, id) in finished {
        if records.len() <= RETAINED_JOBS {
            break;
        }
        records.remove(&id);
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as u64)
        .unwrap_or(0)
}
