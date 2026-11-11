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
#[doc(hidden)]
pub struct ToolVersion {
    parts: Vec<u64>,
    dev: bool,
}

impl ToolVersion {
    pub fn parse(text: &str) -> Self {
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
    /// Version the probe reported, when the tool answered `--version`.
    pub version: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
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

impl ToolSource {
    pub fn as_str(self) -> &'static str {
        match self {
            ToolSource::Preinstalled => "preinstalled",
            ToolSource::System => "system",
            ToolSource::Bundled => "bundled",
            ToolSource::Overlay => "overlay",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ToolStatus {
    pub name: &'static str,
    pub available: bool,
    pub source: Option<ToolSource>,
    pub version: Option<String>,
    pub program: Option<String>,
    pub args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
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
    /// `overlay` is set when the module lives in the writable overlay, so the
    /// probe imports the same copy a download would.
    async fn probe_python(
        &self,
        python: &Path,
        tool: Tool,
        overlay: Option<&Path>,
    ) -> Option<ToolVersion> {
        let mut cmd = Command::new(python);
        hide_console(&mut cmd);
        cmd.arg("-m").arg(tool.module());
        cmd.envs(self.environment(overlay));
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
        version: Option<&ToolVersion>,
    ) -> ResolvedTool {
        ResolvedTool {
            tool,
            program,
            args: vec!["-m".to_string(), tool.module().to_string()],
            env: self.environment(overlay),
            source,
            version: version.map(ToString::to_string),
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
            let version = self.probe_python(python, tool, Some(&overlay)).await;
            let resolved = self.python_tool(
                tool,
                python.clone(),
                ToolSource::Overlay,
                Some(&overlay),
                version.as_ref(),
            );
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
            if let Some(version) = self.probe_python(python, tool, None).await {
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
                version: Some(candidate.version.to_string()),
            };
            self.remember(&resolved);
            return Ok(resolved);
        }

        if let Some((python, source)) = interpreters.into_iter().next() {
            let resolved = self.python_tool(tool, python, source, None, None);
            self.remember(&resolved);
            return Ok(resolved);
        }

        Err(ToolError::Missing { tool: tool.name() })
    }

    /// Resolves one tool into the shape both hosts report to the UI.
    pub async fn status(&self, tool: Tool) -> ToolStatus {
        match self.resolve(tool).await {
            Ok(resolved) => ToolStatus {
                name: tool.name(),
                available: true,
                source: Some(resolved.source),
                version: resolved.version,
                program: Some(resolved.program.display().to_string()),
                args: resolved.args,
                error: None,
            },
            Err(e) => ToolStatus {
                name: tool.name(),
                available: false,
                source: None,
                version: None,
                program: None,
                args: Vec::new(),
                error: Some(e.to_string()),
            },
        }
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
