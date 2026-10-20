use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tokio::process::Command;

pub const TOOL_TARGET: &str = env!("MM_TARGET");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tool {
    YtDlp,
    GalleryDl,
    SpotDl,
}

impl Tool {
    pub fn name(self) -> &'static str {
        match self {
            Tool::YtDlp => "yt-dlp",
            Tool::GalleryDl => "gallery-dl",
            Tool::SpotDl => "spotdl",
        }
    }

    pub fn all() -> [Tool; 3] {
        [Tool::YtDlp, Tool::GalleryDl, Tool::SpotDl]
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("{tool} is not installed and no bundled binary is available")]
    Missing { tool: &'static str },
}

#[derive(Debug, Clone, Default)]
pub struct RuntimeConfig {
    /// Directory holding `yt-dlp[-<target>][.exe]` style binaries shipped with the app.
    pub bundled_tools_dir: Option<PathBuf>,
}

impl RuntimeConfig {
    pub fn with_bundled_tools_dir(dir: impl Into<PathBuf>) -> Self {
        Self {
            bundled_tools_dir: Some(dir.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedTool {
    pub tool: Tool,
    pub path: PathBuf,
    pub source: ToolSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolSource {
    Preinstalled,
    Bundled,
}

#[derive(Debug)]
pub struct ToolRuntime {
    config: RuntimeConfig,
    cache: Mutex<std::collections::HashMap<Tool, PathBuf>>,
    reported: Mutex<HashSet<Tool>>,
}

impl ToolRuntime {
    pub fn new(config: RuntimeConfig) -> Self {
        Self {
            config,
            cache: Mutex::new(std::collections::HashMap::new()),
            reported: Mutex::new(HashSet::new()),
        }
    }

    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    async fn probe(path: &Path) -> bool {
        let mut cmd = Command::new(path);
        hide_console(&mut cmd);
        cmd.arg("--version")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .await
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn bundled_candidates(&self, tool: Tool) -> Vec<PathBuf> {
        let Some(dir) = &self.config.bundled_tools_dir else {
            return Vec::new();
        };
        let suffix = if cfg!(windows) { ".exe" } else { "" };
        vec![
            dir.join(format!("{}-{}{}", tool.name(), TOOL_TARGET, suffix)),
            dir.join(format!("{}{}", tool.name(), suffix)),
        ]
    }

    pub async fn resolve(&self, tool: Tool) -> Result<ResolvedTool, ToolError> {
        if let Ok(cache) = self.cache.lock() {
            if let Some(path) = cache.get(&tool) {
                return Ok(ResolvedTool {
                    tool,
                    path: path.clone(),
                    source: ToolSource::Preinstalled,
                });
            }
        }

        if let Ok(paths) = which::which_all(tool.name()) {
            // The app's own bundled binaries must never shadow a real installation here.
            let filtered = paths.filter(|path| {
                !path
                    .to_string_lossy()
                    .to_ascii_lowercase()
                    .contains("mediamagnet")
            });

            for bin_path in filtered {
                if Self::probe(&bin_path).await {
                    self.remember(tool, bin_path.clone());
                    return Ok(ResolvedTool {
                        tool,
                        path: bin_path,
                        source: ToolSource::Preinstalled,
                    });
                }
                println!(
                    "[MediaMagnet][Download] Backend: Local {} [INVALID]. Continuing...",
                    bin_path.display()
                );
            }
        }

        for candidate in self.bundled_candidates(tool) {
            if candidate.is_file() {
                self.report_once(tool, || {
                    println!("\n[MediaMagnet][Download] Backend: Prebuilt {}", tool.name());
                    println!("  -> This is usually out-of-date, please download the latest version and put in PATH!");
                });
                return Ok(ResolvedTool {
                    tool,
                    path: candidate,
                    source: ToolSource::Bundled,
                });
            }
        }

        Err(ToolError::Missing { tool: tool.name() })
    }

    fn remember(&self, tool: Tool, path: PathBuf) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(tool, path.clone());
        }
        self.report_once(tool, || {
            println!("[MediaMagnet][Download] Backend: Local {}", path.display());
        });
    }

    fn report_once(&self, tool: Tool, report: impl FnOnce()) {
        let first = self
            .reported
            .lock()
            .map(|mut set| set.insert(tool))
            .unwrap_or(false);
        if first {
            report();
        }
    }
}

pub fn hide_console(cmd: &mut Command) {
    #[cfg(target_os = "windows")]
    cmd.creation_flags(0x08000000);
    #[cfg(not(target_os = "windows"))]
    let _ = cmd;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("mm-runtime-{tag}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[tokio::test]
    async fn preinstalled_tool_is_found_before_bundled() {
        let runtime = ToolRuntime::new(RuntimeConfig::default());
        let tool = Tool::GalleryDl;
        let expected = which::which(tool.name()).unwrap();
        let resolved = runtime.resolve(tool).await.unwrap();
        assert_eq!(resolved.source, ToolSource::Preinstalled);
        assert_eq!(resolved.path, expected);
    }

    #[tokio::test]
    async fn preinstalled_resolution_is_cached() {
        let runtime = ToolRuntime::new(RuntimeConfig::default());
        let first = runtime.resolve(Tool::YtDlp).await.unwrap();
        let second = runtime.resolve(Tool::YtDlp).await.unwrap();
        assert_eq!(first.path, second.path);
    }

    #[tokio::test]
    async fn bundled_binary_is_used_when_nothing_is_on_path() {
        let dir = temp_dir("bundled");
        let suffix = if cfg!(windows) { ".exe" } else { "" };
        let target = dir.join(format!("spotdl-{TOOL_TARGET}{suffix}"));
        std::fs::copy(which::which("gallery-dl").unwrap(), &target).unwrap();
        let runtime = ToolRuntime::new(RuntimeConfig::with_bundled_tools_dir(&dir));
        let resolved = runtime.resolve(Tool::SpotDl).await.unwrap();
        assert_eq!(resolved.source, ToolSource::Bundled);
        assert_eq!(resolved.path, target);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[tokio::test]
    async fn missing_tool_reports_its_own_name() {
        let runtime = ToolRuntime::new(RuntimeConfig::default());
        let err = runtime.resolve(Tool::SpotDl).await.unwrap_err();
        assert!(err.to_string().starts_with("spotdl is not installed"));
    }
}
