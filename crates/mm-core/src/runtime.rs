use std::collections::{HashMap, HashSet};
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

    pub fn module(self) -> &'static str {
        match self {
            Tool::YtDlp => "yt_dlp",
            Tool::GalleryDl => "gallery_dl",
            Tool::SpotDl => "spotdl",
        }
    }

    /// pip requirement with the extras the tool needs at runtime. The bundles are
    /// built from these, so an overlay reinstall gets the same dependencies.
    pub fn package(self) -> &'static str {
        match self {
            Tool::YtDlp => "yt-dlp[default]",
            Tool::GalleryDl => "gallery-dl[extra]",
            Tool::SpotDl => "spotdl",
        }
    }

    pub fn all() -> [Tool; 3] {
        [Tool::YtDlp, Tool::GalleryDl, Tool::SpotDl]
    }

    /// Names to look for on PATH: the release binary and the dev/git variants.
    pub fn candidate_names(self) -> [&'static str; 3] {
        match self {
            Tool::YtDlp => ["yt-dlp", "yt-dlp-dev", "yt-dlp-git"],
            Tool::GalleryDl => ["gallery-dl", "gallery-dl-dev", "gallery-dl-git"],
            Tool::SpotDl => ["spotdl", "spotdl-dev", "spotdl-git"],
        }
    }
}

/// Parsed `--version` output. The version tuple orders numerically, and a
/// dev/git/nightly build wins a tie against the matching release.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
struct ToolVersion {
    parts: Vec<u64>,
    dev: bool,
}

impl ToolVersion {
    fn parse(text: &str) -> Self {
        static PATTERN: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
        let re = PATTERN
            .get_or_init(|| regex::Regex::new(r"\d+(?:\.\d+)+").expect("valid version pattern"));
        let parts = re
            .find(text)
            .map(|m| {
                m.as_str()
                    .split('.')
                    .filter_map(|p| p.parse().ok())
                    .collect()
            })
            .unwrap_or_default();
        let lower = text.to_ascii_lowercase();
        Self {
            parts,
            dev: lower.contains("dev") || lower.contains("git") || lower.contains("nightly"),
        }
    }
}

impl std::fmt::Display for ToolVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let parts = self
            .parts
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(".");
        write!(f, "{parts}{}", if self.dev { "-dev" } else { "" })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("{tool} is not installed and no bundled runtime is available")]
    Missing { tool: &'static str },
    #[error("python dependency install failed: {0}")]
    Install(String),
}

#[derive(Debug, Clone, Default)]
pub struct RuntimeConfig {
    /// Directory holding the bundled `python/`, `deno`, `ffmpeg` and `ffprobe`.
    pub bundled_tools_dir: Option<PathBuf>,
    /// Writable copy of the tools installed by `reinstall_python_deps`.
    pub overlay_dir: Option<PathBuf>,
    /// User-supplied interpreter for the System mode (`MM_PYTHON`).
    pub python_path: Option<PathBuf>,
}

impl RuntimeConfig {
    pub fn with_bundled_tools_dir(dir: impl Into<PathBuf>) -> Self {
        Self {
            bundled_tools_dir: Some(dir.into()),
            ..Self::default()
        }
    }

    pub fn with_overlay_dir(dir: impl Into<PathBuf>) -> Self {
        Self {
            overlay_dir: Some(dir.into()),
            ..Self::default()
        }
    }

    pub fn with_python_path(path: impl Into<PathBuf>) -> Self {
        Self {
            python_path: Some(path.into()),
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedTool {
    pub tool: Tool,
    /// Executable to spawn: the tool itself, or the Python interpreter.
    pub program: PathBuf,
    /// Leading arguments before the tool's own args (`-m <module>` for Python).
    pub args: Vec<String>,
    /// Extra environment for the child process.
    pub env: Vec<(String, String)>,
    pub source: ToolSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolSource {
    /// The tool binary is on PATH.
    Preinstalled,
    /// A user-supplied Python interpreter runs the module.
    System,
    /// The bundled Python interpreter runs the module.
    Bundled,
    /// The bundled/system interpreter runs the module from the writable overlay.
    Overlay,
}

/// One probe-validated way to run a tool, before the newest is picked.
#[derive(Debug)]
struct Candidate {
    program: PathBuf,
    args: Vec<String>,
    source: ToolSource,
    version: ToolVersion,
}

#[derive(Debug)]
pub struct ToolRuntime {
    config: RuntimeConfig,
    cache: Mutex<HashMap<Tool, ResolvedTool>>,
    reported: Mutex<HashSet<Tool>>,
}

impl ToolRuntime {
    pub fn new(config: RuntimeConfig) -> Self {
        Self {
            config,
            cache: Mutex::new(HashMap::new()),
            reported: Mutex::new(HashSet::new()),
        }
    }

    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    async fn probe(path: &Path, dev_name: bool) -> Option<ToolVersion> {
        let mut cmd = Command::new(path);
        hide_console(&mut cmd);
        Self::version_of(cmd, dev_name).await
    }

    /// Ask an interpreter for the module's version (`<python> -m <module> --version`).
    async fn probe_python(&self, python: &Path, tool: Tool) -> Option<ToolVersion> {
        let mut cmd = Command::new(python);
        hide_console(&mut cmd);
        cmd.arg("-m").arg(tool.module());
        cmd.envs(self.environment(None));
        Self::version_of(cmd, false).await
    }

    async fn version_of(mut cmd: Command, dev_name: bool) -> Option<ToolVersion> {
        let output = cmd
            .arg("--version")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .await
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
        text.push_str(&String::from_utf8_lossy(&output.stderr));
        let mut version = ToolVersion::parse(&text);
        version.dev |= dev_name;
        Some(version)
    }

    fn bundled_python(&self) -> Option<PathBuf> {
        let dir = self.config.bundled_tools_dir.as_ref()?.join("python");
        let candidates = if cfg!(windows) {
            vec![dir.join("python.exe"), dir.join("bin").join("python.exe")]
        } else {
            vec![
                dir.join("bin").join("python3"),
                dir.join("bin").join("python"),
            ]
        };
        candidates.into_iter().find(|path| path.is_file())
    }

    fn bundled_bin(&self, name: &str) -> Option<PathBuf> {
        let dir = self.config.bundled_tools_dir.as_ref()?;
        let name = if cfg!(windows) {
            format!("{name}.exe")
        } else {
            name.to_string()
        };
        let path = dir.join(name);
        path.is_file().then_some(path)
    }

    fn interpreters(&self) -> Vec<(PathBuf, ToolSource)> {
        let mut found = Vec::new();
        if let Some(path) = self.bundled_python() {
            found.push((path, ToolSource::Bundled));
        }
        if let Some(path) = self
            .config
            .python_path
            .as_ref()
            .filter(|path| path.is_file())
        {
            found.push((path.clone(), ToolSource::System));
        }
        found
    }

    fn overlay_for(&self, tool: Tool) -> Option<PathBuf> {
        let dir = self.config.overlay_dir.as_ref()?;
        dir.join(tool.module()).is_dir().then(|| dir.clone())
    }

    fn environment(&self, pythonpath: Option<&Path>) -> Vec<(String, String)> {
        let mut env = vec![("PYTHONNOUSERSITE".to_string(), "1".to_string())];

        if let Some(overlay) = pythonpath {
            let mut paths = vec![overlay.to_path_buf()];
            if let Some(existing) = std::env::var_os("PYTHONPATH") {
                paths.extend(std::env::split_paths(&existing));
            }
            if let Ok(joined) = std::env::join_paths(paths) {
                env.push((
                    "PYTHONPATH".to_string(),
                    joined.to_string_lossy().into_owned(),
                ));
            }
        }

        if let Some(dir) = self
            .config
            .bundled_tools_dir
            .as_ref()
            .filter(|dir| dir.is_dir())
        {
            let mut paths = vec![dir.clone()];
            if let Some(existing) = std::env::var_os("PATH") {
                paths.extend(std::env::split_paths(&existing));
            }
            if let Ok(joined) = std::env::join_paths(paths) {
                env.push(("PATH".to_string(), joined.to_string_lossy().into_owned()));
            }
        }

        env
    }

    fn python_tool(
        &self,
        tool: Tool,
        program: PathBuf,
        source: ToolSource,
        overlay: Option<&Path>,
    ) -> ResolvedTool {
        ResolvedTool {
            tool,
            program,
            args: vec!["-m".to_string(), tool.module().to_string()],
            env: self.environment(overlay),
            source,
        }
    }

    fn remember(&self, resolved: &ResolvedTool) -> bool {
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(resolved.tool, resolved.clone());
        }
        let first = self
            .reported
            .lock()
            .map(|mut set| set.insert(resolved.tool))
            .unwrap_or(false);
        if first {
            println!(
                "[MediaMagnet][Download] Backend: {:?} {}",
                resolved.source,
                resolved.program.display()
            );
        }
        first
    }

    pub async fn resolve(&self, tool: Tool) -> Result<ResolvedTool, ToolError> {
        if let Ok(cache) = self.cache.lock() {
            if let Some(resolved) = cache.get(&tool) {
                return Ok(resolved.clone());
            }
        }

        let interpreters = self.interpreters();

        if let (Some(overlay), Some((python, _))) = (self.overlay_for(tool), interpreters.first()) {
            let resolved =
                self.python_tool(tool, python.clone(), ToolSource::Overlay, Some(&overlay));
            self.remember(&resolved);
            return Ok(resolved);
        }

        // Every candidate — PATH binaries and the interpreters' modules — is
        // probed; the newest version wins, with dev/git builds preferred over a
        // release reporting the same version. Ties keep the earlier candidate,
        // so PATH is preferred over an interpreter of the same version.
        let mut newest: Option<Candidate> = None;
        for name in tool.candidate_names() {
            let dev_name = name != tool.name();
            let Ok(paths) = which::which_all(name) else {
                continue;
            };
            // The app's own bundled binaries must never shadow a real installation here.
            let filtered = paths.filter(|path| {
                !path
                    .to_string_lossy()
                    .to_ascii_lowercase()
                    .contains("mediamagnet")
            });
            for bin_path in filtered {
                match Self::probe(&bin_path, dev_name).await {
                    Some(version) => {
                        println!(
                            "[MediaMagnet][Download] Found: {} {version}",
                            bin_path.display()
                        );
                        if newest
                            .as_ref()
                            .is_none_or(|candidate| version > candidate.version)
                        {
                            newest = Some(Candidate {
                                program: bin_path,
                                args: Vec::new(),
                                source: ToolSource::Preinstalled,
                                version,
                            });
                        }
                    }
                    None => println!(
                        "[MediaMagnet][Download] Backend: Local {} [INVALID]. Continuing...",
                        bin_path.display()
                    ),
                }
            }
        }

        // A bundled or MM_PYTHON interpreter can hold a newer tool than the one
        // on PATH, so its module version competes with the PATH candidates.
        for (python, source) in &interpreters {
            if let Some(version) = self.probe_python(python, tool).await {
                println!(
                    "[MediaMagnet][Download] Found: {} -m {} {version}",
                    python.display(),
                    tool.module()
                );
                if newest
                    .as_ref()
                    .is_none_or(|candidate| version > candidate.version)
                {
                    newest = Some(Candidate {
                        program: python.clone(),
                        args: vec!["-m".to_string(), tool.module().to_string()],
                        source: *source,
                        version,
                    });
                }
            }
        }

        if let Some(candidate) = newest {
            println!("[MediaMagnet][Download] Selected: {}", candidate.version);
            let resolved = ResolvedTool {
                tool,
                program: candidate.program,
                args: candidate.args,
                env: self.environment(None),
                source: candidate.source,
            };
            self.remember(&resolved);
            return Ok(resolved);
        }

        if let Some((python, source)) = interpreters.into_iter().next() {
            let resolved = self.python_tool(tool, python, source, None);
            self.remember(&resolved);
            return Ok(resolved);
        }

        Err(ToolError::Missing { tool: tool.name() })
    }

    /// Bundled Deno, if shipped. It is also prepended to PATH for every child, which
    /// is how yt-dlp finds it without a version-specific `--js-runtimes` flag.
    pub fn deno(&self) -> Option<PathBuf> {
        self.bundled_bin("deno")
    }

    /// Bundled ffmpeg first, then one on PATH.
    pub fn ffmpeg(&self) -> Option<PathBuf> {
        self.bundled_bin("ffmpeg")
            .or_else(|| which::which("ffmpeg").ok())
    }

    /// Installs the tools into the overlay and returns pip's console output.
    pub async fn reinstall_python_deps(&self, tools: &[Tool]) -> Result<String, ToolError> {
        let python = self
            .interpreters()
            .into_iter()
            .next()
            .map(|(path, _)| path)
            .ok_or(ToolError::Install(
                "no bundled or system python found".into(),
            ))?;
        let overlay = self
            .config
            .overlay_dir
            .as_ref()
            .ok_or(ToolError::Install("no overlay directory configured".into()))?;

        std::fs::create_dir_all(overlay).map_err(|e| {
            ToolError::Install(format!("could not create {}: {e}", overlay.display()))
        })?;

        let mut cmd = Command::new(python);
        hide_console(&mut cmd);
        cmd.args(["-m", "pip", "install", "--target"])
            .arg(overlay)
            .arg("--upgrade")
            .args(tools.iter().map(|tool| tool.package()));

        let output = cmd
            .output()
            .await
            .map_err(|e| ToolError::Install(e.to_string()))?;

        let mut log = String::from_utf8_lossy(&output.stdout).into_owned();
        log.push_str(&String::from_utf8_lossy(&output.stderr));
        let log = log.trim().to_string();

        if !output.status.success() {
            return Err(ToolError::Install(log));
        }

        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }

        for tool in tools {
            if !overlay.join(tool.module()).is_dir() {
                return Err(ToolError::Install(format!(
                    "pip finished but {} is missing from the overlay",
                    tool.module()
                )));
            }
        }

        Ok(log)
    }

    /// Removes the writable overlay, returning to the bundled packages.
    pub fn reset_python_overlay(&self) -> std::io::Result<()> {
        let Some(dir) = self.config.overlay_dir.as_ref() else {
            return Ok(());
        };
        match std::fs::remove_dir_all(dir) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
        Ok(())
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
        let dir = std::env::temp_dir().join(format!(
            "mm-runtime-{tag}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn fake_python(dir: &Path) -> PathBuf {
        let python = if cfg!(windows) {
            dir.join("python").join("python.exe")
        } else {
            dir.join("python").join("bin").join("python3")
        };
        std::fs::create_dir_all(python.parent().unwrap()).unwrap();
        std::fs::write(&python, b"").unwrap();
        python
    }

    #[test]
    fn newest_version_wins_and_dev_breaks_ties() {
        assert!(ToolVersion::parse("2026.10.21.123456") > ToolVersion::parse("2026.10.21"));
        assert!(ToolVersion::parse("1.32.17-dev") > ToolVersion::parse("1.32.17"));
        assert!(ToolVersion::parse("1.33.0") > ToolVersion::parse("1.32.17-dev"));
        assert_eq!(ToolVersion::parse("spotdl 4.5.2").parts, vec![4, 5, 2]);
        assert!(ToolVersion::parse("no version here").parts.is_empty());
    }

    #[tokio::test]
    async fn preinstalled_tool_is_found_before_bundled() {
        let runtime = ToolRuntime::new(RuntimeConfig::default());
        let tool = Tool::GalleryDl;
        let expected = which::which(tool.name()).unwrap();
        let resolved = runtime.resolve(tool).await.unwrap();
        assert_eq!(resolved.source, ToolSource::Preinstalled);
        assert_eq!(resolved.program, expected);
        assert!(resolved.args.is_empty());
    }

    #[tokio::test]
    async fn preinstalled_resolution_is_cached() {
        let runtime = ToolRuntime::new(RuntimeConfig::default());
        let first = runtime.resolve(Tool::YtDlp).await.unwrap();
        let second = runtime.resolve(Tool::YtDlp).await.unwrap();
        assert_eq!(first.program, second.program);
    }

    #[tokio::test]
    async fn bundled_python_is_used_when_nothing_is_on_path() {
        let dir = temp_dir("bundled");
        let python = fake_python(&dir);
        let runtime = ToolRuntime::new(RuntimeConfig::with_bundled_tools_dir(&dir));
        let resolved = runtime.resolve(Tool::SpotDl).await.unwrap();
        assert_eq!(resolved.source, ToolSource::Bundled);
        assert_eq!(resolved.program, python);
        assert_eq!(resolved.args, ["-m", "spotdl"]);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[tokio::test]
    async fn overlay_wins_over_path() {
        let dir = temp_dir("overlay");
        let python = fake_python(&dir);
        let overlay = dir.join("py-overlay");
        std::fs::create_dir_all(overlay.join("yt_dlp")).unwrap();
        let runtime = ToolRuntime::new(RuntimeConfig {
            bundled_tools_dir: Some(dir.clone()),
            overlay_dir: Some(overlay.clone()),
            python_path: None,
        });
        let resolved = runtime.resolve(Tool::YtDlp).await.unwrap();
        assert_eq!(resolved.source, ToolSource::Overlay);
        assert_eq!(resolved.program, python);
        let pythonpath = resolved
            .env
            .iter()
            .find(|(key, _)| key == "PYTHONPATH")
            .map(|(_, value)| value.clone())
            .unwrap();
        assert!(pythonpath.starts_with(overlay.to_string_lossy().as_ref()));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[tokio::test]
    async fn system_python_runs_the_module() {
        let dir = temp_dir("system");
        let python = fake_python(&dir);
        let runtime = ToolRuntime::new(RuntimeConfig::with_python_path(&python));
        let resolved = runtime.resolve(Tool::SpotDl).await.unwrap();
        assert_eq!(resolved.source, ToolSource::System);
        assert_eq!(resolved.program, python);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[tokio::test]
    async fn reset_removes_the_overlay() {
        let dir = temp_dir("reset");
        let overlay = dir.join("py-overlay");
        std::fs::create_dir_all(overlay.join("spotdl")).unwrap();
        let runtime = ToolRuntime::new(RuntimeConfig::with_overlay_dir(&overlay));
        runtime.reset_python_overlay().unwrap();
        assert!(!overlay.exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[tokio::test]
    async fn missing_tool_reports_its_own_name() {
        let runtime = ToolRuntime::new(RuntimeConfig::default());
        let err = runtime.resolve(Tool::SpotDl).await.unwrap_err();
        assert!(err.to_string().starts_with("spotdl is not installed"));
    }
}
