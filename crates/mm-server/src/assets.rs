//! Static files for the SPA, embedded at build time or read from disk.
//!
//! `MM_STATIC_DIR` always wins, which keeps `vite dev` and split deployments
//! working. Otherwise the copy compiled into the binary is used.

use std::path::PathBuf;

include!(concat!(env!("OUT_DIR"), "/spa_assets.rs"));

#[derive(Clone)]
pub enum StaticAssets {
    Disk(PathBuf),
    Embedded,
}

impl StaticAssets {
    pub fn from_env() -> Self {
        if let Some(dir) = std::env::var_os("MM_STATIC_DIR").map(PathBuf::from) {
            return Self::Disk(dir);
        }
        if !FILES.is_empty() {
            return Self::Embedded;
        }
        let dir = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|parent| parent.join("public")))
            .unwrap_or_else(|| PathBuf::from("public"));
        Self::Disk(dir)
    }

    pub fn describe(&self) -> String {
        match self {
            Self::Disk(dir) => dir.display().to_string(),
            Self::Embedded => "embedded in the binary".to_string(),
        }
    }

    pub async fn read(&self, path: &str) -> Option<Vec<u8>> {
        match self {
            Self::Disk(dir) => tokio::fs::read(dir.join(path)).await.ok(),
            Self::Embedded => FILES
                .iter()
                .find(|(name, _)| *name == path)
                .map(|(_, bytes)| bytes.to_vec()),
        }
    }
}
