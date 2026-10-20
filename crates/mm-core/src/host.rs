use std::path::{Path, PathBuf};

use crate::error::Result;

/// Platform services the core needs from whichever process embeds it.
pub trait Host: Send + Sync {
    /// Default download root, e.g. `~/Downloads/MediaMagnet`.
    fn default_download_dir(&self) -> PathBuf;

    /// Directory containing tools shipped with the application, if any.
    fn bundled_tools_dir(&self) -> Option<PathBuf> {
        None
    }

    /// Shows a native notification if the host supports it; returns whether it did.
    fn notify(&self, message: &str) -> bool {
        let _ = message;
        false
    }

    /// Reveals a path in the platform file manager.
    fn open_path(&self, path: &Path) -> Result<()> {
        let _ = path;
        Err(crate::error::Error::Msg(
            "opening files is not supported by this host".into(),
        ))
    }
}

/// Resolves the download root the same way the desktop app always has.
pub fn resolve_download_path(settings: &crate::settings::Settings, host: &dyn Host) -> PathBuf {
    let default_dir = host.default_download_dir().join("MediaMagnet");

    let downloads_path = match settings.download_path.as_str() {
        p if p.trim().is_empty() || p.trim().eq_ignore_ascii_case("default") => default_dir.clone(),
        custom_path if custom_path.to_lowercase().contains("mediamagnet") => {
            std::path::PathBuf::from(custom_path)
        }
        path => std::path::PathBuf::from(path).join("MediaMagnet"),
    };

    if downloads_path.exists() {
        return downloads_path;
    }

    match std::fs::create_dir_all(&downloads_path) {
        Ok(()) => downloads_path,
        Err(_e) => {
            let _ = host.notify("[MME] Failed to create directory, using default...");
            let _ = std::fs::create_dir_all(&default_dir);
            default_dir
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DiskUsage {
    pub used_percent: f64,
}

/// Used-space percentage of the filesystem holding `path`.
pub fn disk_usage(path: &Path) -> Result<DiskUsage> {
    let disks = sysinfo::Disks::new_with_refreshed_list();

    let disk = disks
        .iter()
        .filter(|d| path.starts_with(d.mount_point()))
        .max_by_key(|d| d.mount_point().as_os_str().len());

    match disk {
        Some(d) => {
            let total = d.total_space() as f64;
            let available = d.available_space() as f64;
            if total <= 0.0 {
                return Err(crate::error::Error::Msg(
                    "Could not determine disk usage".into(),
                ));
            }
            Ok(DiskUsage {
                used_percent: ((total - available) / total) * 100.0,
            })
        }
        None => Err(crate::error::Error::Msg(
            "Could not determine disk usage".into(),
        )),
    }
}
