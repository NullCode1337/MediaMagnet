//! Deletes server job outputs so the download root does not accumulate them.
//!
//! A file is removed as soon as the client that asked for it has fetched the
//! whole body, or after [`Retention::ttl`] if nobody does. Ownership is keyed by
//! the path relative to the download root, so a newer job that wrote the same
//! path is never deleted by an older job's cleanup.
//!
//! The ownership map is persisted beside the server's settings. A restart drops
//! every job record, so a file left by the previous process can never be fetched
//! again; `Retention::open` removes those files before serving starts.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use mm_core::files::safe_join;

/// Identifies one file instance, so cleanup does not delete a different file
/// that reused the same path.
#[derive(Clone, Copy, Debug)]
pub struct FileIdentity {
    len: u64,
    #[cfg(unix)]
    dev: u64,
    #[cfg(unix)]
    ino: u64,
    #[cfg(not(unix))]
    modified: Option<std::time::SystemTime>,
}

impl FileIdentity {
    pub fn from_metadata(meta: &std::fs::Metadata) -> Self {
        #[cfg(unix)]
        use std::os::unix::fs::MetadataExt;

        Self {
            len: meta.len(),
            #[cfg(unix)]
            dev: meta.dev(),
            #[cfg(unix)]
            ino: meta.ino(),
            #[cfg(not(unix))]
            modified: meta.modified().ok(),
        }
    }

    pub fn matches(&self, meta: &std::fs::Metadata) -> bool {
        if self.len != meta.len() {
            return false;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            self.dev == meta.dev() && self.ino == meta.ino()
        }
        #[cfg(not(unix))]
        {
            meta.modified().ok() == self.modified
        }
    }
}

#[derive(Clone)]
pub struct Retention {
    root: PathBuf,
    ttl: Duration,
    manifest: PathBuf,
    /// Relative path to the id of the job that last wrote it.
    files: Arc<Mutex<HashMap<String, String>>>,
}

impl Retention {
    /// Loads the previous process's manifest, deletes any files it names, and
    /// returns a clean registry.
    pub fn open(root: PathBuf, ttl: Duration, manifest: PathBuf) -> Self {
        let mut previous = HashMap::new();
        if let Ok(bytes) = std::fs::read(&manifest) {
            if let Ok(map) = serde_json::from_slice::<HashMap<String, String>>(&bytes) {
                previous = map;
            }
        }

        for relative in previous.keys() {
            let Ok(path) = safe_join(&root, relative) else {
                continue;
            };
            if std::fs::remove_file(&path).is_ok() {
                prune_empty_parents_sync(&root, &path);
            }
        }

        let retention = Self {
            root,
            ttl,
            manifest,
            files: Arc::new(Mutex::new(HashMap::new())),
        };
        retention.persist(&HashMap::new());
        retention
    }

    /// Starts the expiry clock for a finished job and remembers that this
    /// process owns its files.
    pub fn record(&self, id: &str, files: &[String]) {
        if files.is_empty() {
            return;
        }

        let mut owned = self.lock();
        for file in files {
            owned.insert(file.clone(), id.to_string());
        }
        self.persist(&owned);
        drop(owned);

        let this = self.clone();
        let id = id.to_string();
        let files = files.to_vec();
        tokio::spawn(async move {
            tokio::time::sleep(this.ttl).await;
            for relative in files {
                this.delete(&id, &relative, None).await;
            }
        });
    }

    /// Removes a file that was streamed to a client in full. `identity` is the
    /// file opened for that response, so a path that was replaced in the
    /// meantime is left alone.
    pub async fn delete_delivered(&self, id: &str, relative: &str, identity: FileIdentity) {
        self.delete(id, relative, Some(identity)).await;
    }

    async fn delete(&self, id: &str, relative: &str, identity: Option<FileIdentity>) {
        if !self.owned_by(id, relative) {
            return;
        }

        let Ok(path) = safe_join(&self.root, relative) else {
            self.release(id, relative);
            return;
        };

        let Ok(meta) = tokio::fs::metadata(&path).await else {
            self.release(id, relative);
            return;
        };
        if !meta.is_file() || !identity.is_none_or(|want| want.matches(&meta)) {
            // The path no longer holds this job's file (or never did).
            self.release(id, relative);
            return;
        }

        // After a delivery the response body may still hold the file open (on
        // Windows that blocks unlinking), so give it a moment to close. The TTL
        // sweep is the fallback if it does not.
        let attempts = if identity.is_some() { 10 } else { 1 };
        if remove_file(&path, attempts).await {
            self.release(id, relative);
            prune_empty_parents(&self.root, &path).await;
        }
        // Otherwise leave ownership in place so the TTL sweep retries.
    }

    fn owned_by(&self, id: &str, relative: &str) -> bool {
        matches!(self.lock().get(relative), Some(owner) if owner == id)
    }

    fn release(&self, id: &str, relative: &str) {
        let mut owned = self.lock();
        if matches!(owned.get(relative), Some(owner) if owner == id) {
            owned.remove(relative);
            self.persist(&owned);
        }
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<String, String>> {
        self.files
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn persist(&self, owned: &HashMap<String, String>) {
        let Ok(bytes) = serde_json::to_vec(owned) else {
            return;
        };
        let mut tmp = self.manifest.clone().into_os_string();
        tmp.push(".tmp");
        let tmp = PathBuf::from(tmp);
        if std::fs::write(&tmp, bytes).is_ok() {
            let _ = std::fs::rename(&tmp, &self.manifest);
        }
    }
}

async fn remove_file(path: &Path, attempts: u32) -> bool {
    for attempt in 0..attempts {
        if tokio::fs::remove_file(path).await.is_ok() {
            return true;
        }
        if tokio::fs::metadata(path).await.is_err() {
            // Already gone, for example unlinked by another request.
            return true;
        }
        if attempt + 1 < attempts {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    false
}

async fn prune_empty_parents(root: &Path, path: &Path) {
    let mut dir = path.parent();
    while let Some(current) = dir {
        if current == root || !current.starts_with(root) {
            break;
        }
        if tokio::fs::remove_dir(current).await.is_err() {
            break;
        }
        dir = current.parent();
    }
}

fn prune_empty_parents_sync(root: &Path, path: &Path) {
    let mut dir = path.parent();
    while let Some(current) = dir {
        if current == root || !current.starts_with(root) {
            break;
        }
        if std::fs::remove_dir(current).is_err() {
            break;
        }
        dir = current.parent();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("mm-retention-{name}-{unique}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn retention(root: &Path, ttl: Duration) -> Retention {
        Retention::open(root.to_path_buf(), ttl, root.join("job-files.json"))
    }

    #[tokio::test]
    async fn delivered_files_are_removed() {
        let root = temp_dir("delivered");
        std::fs::create_dir_all(root.join("set")).unwrap();
        std::fs::write(root.join("set/one.jpg"), b"one").unwrap();
        // Simulates a file the server did not produce, for example a desktop
        // download in a shared directory.
        std::fs::write(root.join("set/desktop.jpg"), b"keep").unwrap();

        let retention = retention(&root, Duration::from_secs(3600));
        retention.record("job", &["set/one.jpg".into()]);

        let file = tokio::fs::File::open(root.join("set/one.jpg"))
            .await
            .unwrap();
        let identity = FileIdentity::from_metadata(&file.metadata().await.unwrap());
        retention
            .delete_delivered("job", "set/one.jpg", identity)
            .await;

        assert!(!root.join("set/one.jpg").exists());
        assert!(root.join("set/desktop.jpg").exists());
        assert!(root.join("set").exists());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn expiry_removes_only_recorded_files() {
        let root = temp_dir("expiry");
        std::fs::write(root.join("clip.mp4"), b"clip").unwrap();
        std::fs::write(root.join("desktop.mp4"), b"keep").unwrap();

        let retention = retention(&root, Duration::from_millis(10));
        retention.record("job", &["clip.mp4".into()]);
        tokio::time::sleep(Duration::from_millis(100)).await;

        assert!(!root.join("clip.mp4").exists());
        assert!(root.join("desktop.mp4").exists());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn a_newer_job_keeps_ownership_of_the_same_path() {
        let root = temp_dir("ownership");
        std::fs::write(root.join("clip.mp4"), b"first").unwrap();

        let retention = retention(&root, Duration::from_secs(3600));
        retention.record("old", &["clip.mp4".into()]);
        retention.record("new", &["clip.mp4".into()]);

        let file = tokio::fs::File::open(root.join("clip.mp4")).await.unwrap();
        let identity = FileIdentity::from_metadata(&file.metadata().await.unwrap());
        retention
            .delete_delivered("old", "clip.mp4", identity)
            .await;
        assert!(root.join("clip.mp4").exists());

        retention
            .delete_delivered("new", "clip.mp4", identity)
            .await;
        assert!(!root.join("clip.mp4").exists());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn a_replaced_file_is_not_deleted() {
        let root = temp_dir("replaced");
        let path = root.join("clip.mp4");
        std::fs::write(&path, b"first").unwrap();

        let retention = retention(&root, Duration::from_secs(3600));
        retention.record("job", &["clip.mp4".into()]);

        let file = tokio::fs::File::open(&path).await.unwrap();
        let identity = FileIdentity::from_metadata(&file.metadata().await.unwrap());
        drop(file);

        // Same length, different inode: a replacement, not the served file.
        let replacement = root.join("replacement.mp4");
        std::fs::write(&replacement, b"other").unwrap();
        std::fs::rename(&replacement, &path).unwrap();

        retention
            .delete_delivered("job", "clip.mp4", identity)
            .await;
        assert!(path.exists());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn restart_removes_previous_outputs() {
        let root = temp_dir("restart");
        let manifest = root.join("job-files.json");
        std::fs::write(root.join("clip.mp4"), b"clip").unwrap();
        std::fs::write(
            &manifest,
            serde_json::to_vec(&HashMap::from([(
                "clip.mp4".to_string(),
                "job".to_string(),
            )]))
            .unwrap(),
        )
        .unwrap();

        let _retention = Retention::open(root.clone(), Duration::from_secs(3600), manifest);
        assert!(!root.join("clip.mp4").exists());
        let _ = std::fs::remove_dir_all(&root);
    }
}
