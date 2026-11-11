use std::path::{Path, PathBuf};

use mm_core::runtime::ToolVersion;
use mm_core::{RuntimeConfig, Tool, ToolRuntime, ToolSource};

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
    assert_eq!(ToolVersion::parse("spotdl 4.5.2").to_string(), "4.5.2");
    assert_eq!(ToolVersion::parse("no version here").to_string(), "");
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
