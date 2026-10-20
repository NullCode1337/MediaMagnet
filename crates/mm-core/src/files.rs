use std::path::{Component, Path, PathBuf};

use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryKind {
    File,
    Directory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirEntry {
    /// Path relative to the root, using `/` separators.
    pub path: String,
    pub name: String,
    pub kind: EntryKind,
    pub size: u64,
}

/// Resolves a client-supplied relative path under `root`, refusing anything that
/// escapes it (including `..`, absolute paths and symlinks pointing outside).
pub fn safe_join(root: &Path, relative: &str) -> Result<PathBuf> {
    let relative = relative.replace('\\', "/");
    let rel = Path::new(&relative);

    if rel.is_absolute() {
        return Err(Error::Msg("path must be relative".into()));
    }
    for component in rel.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {}
            _ => return Err(Error::Msg("path escapes the download root".into())),
        }
    }

    let canonical_root = root
        .canonicalize()
        .map_err(|e| Error::Msg(format!("download root unavailable: {e}")))?;
    let candidate = canonical_root.join(rel);

    // Resolve the closest existing ancestor, so a missing tail cannot hide a symlink escape.
    let mut probe = candidate.as_path();
    let mut missing = Vec::new();
    let resolved = loop {
        if let Ok(existing) = probe.canonicalize() {
            break existing.join(missing.iter().rev().collect::<PathBuf>());
        }
        match (probe.parent(), probe.file_name()) {
            (Some(parent), Some(name)) => {
                missing.push(name.to_os_string());
                probe = parent;
            }
            _ => return Err(Error::Msg("path escapes the download root".into())),
        }
    };

    if resolved.starts_with(&canonical_root) {
        Ok(resolved)
    } else {
        Err(Error::Msg("path escapes the download root".into()))
    }
}

pub fn list_dir(root: &Path, relative: &str) -> Result<Vec<DirEntry>> {
    let dir = safe_join(root, relative)?;
    if !dir.is_dir() {
        return Err(Error::Msg("not a directory".into()));
    }

    let mut entries = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let meta = entry.metadata()?;
        let is_dir = meta.is_dir();
        entries.push(DirEntry {
            path: join_relative(relative, &name),
            name,
            kind: if is_dir {
                EntryKind::Directory
            } else {
                EntryKind::File
            },
            size: if is_dir { 0 } else { meta.len() },
        });
    }

    entries.sort_by(|a, b| {
        let rank = |kind: &EntryKind| matches!(kind, EntryKind::Directory) as u8;
        rank(&b.kind)
            .cmp(&rank(&a.kind))
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(entries)
}

fn join_relative(relative: &str, name: &str) -> String {
    let base = relative.trim_matches('/');
    if base.is_empty() {
        name.to_string()
    } else {
        format!("{base}/{name}")
    }
}

/// Byte range for a requested file, clamped to the file length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteRange {
    pub start: u64,
    pub end: u64,
}

impl ByteRange {
    pub fn len(&self) -> u64 {
        self.end - self.start + 1
    }

    pub fn is_empty(&self) -> bool {
        false
    }
}

/// Parses a single `bytes=` range. Multiple ranges are deliberately unsupported.
pub fn parse_range(header: &str, len: u64) -> Result<Option<ByteRange>> {
    let Some(spec) = header.trim().strip_prefix("bytes=") else {
        return Ok(None);
    };
    if spec.contains(',') {
        return Err(Error::Msg("multiple ranges are not supported".into()));
    }
    let (start, end) = spec
        .split_once('-')
        .ok_or_else(|| Error::Msg("malformed range".into()))?;

    if len == 0 {
        return Err(Error::Msg("range not satisfiable".into()));
    }

    let range = match (start.trim(), end.trim()) {
        ("", "") => return Err(Error::Msg("malformed range".into())),
        ("", suffix) => {
            let suffix: u64 = suffix
                .parse()
                .map_err(|_| Error::Msg("malformed range".into()))?;
            let suffix = suffix.min(len);
            ByteRange {
                start: len - suffix,
                end: len - 1,
            }
        }
        (start, "") => {
            let start: u64 = start
                .parse()
                .map_err(|_| Error::Msg("malformed range".into()))?;
            if start >= len {
                return Err(Error::Msg("range not satisfiable".into()));
            }
            ByteRange {
                start,
                end: len - 1,
            }
        }
        (start, end) => {
            let start: u64 = start
                .parse()
                .map_err(|_| Error::Msg("malformed range".into()))?;
            let end: u64 = end
                .parse()
                .map_err(|_| Error::Msg("malformed range".into()))?;
            if start > end || start >= len {
                return Err(Error::Msg("range not satisfiable".into()));
            }
            ByteRange {
                start,
                end: end.min(len - 1),
            }
        }
    };

    Ok(Some(range))
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(parse_range("items=0-4", 11).unwrap().is_none());
        assert!(parse_range("bytes=0-4", 0).is_err());
    }
}
