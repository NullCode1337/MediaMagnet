//! Embeds the built web UI so the server binary is self-contained.
//!
//! `bun run build:server` builds the SPA before cargo runs. When the dist
//! directory is missing, the generated table is empty and the server falls
//! back to `MM_STATIC_DIR` at runtime.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let dist = manifest.join("../../apps/web/dist");
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));

    // Watch the web app rather than dist alone: Cargo does not notice a watched
    // path that did not exist, but it does rerun when a child of an existing
    // directory appears.
    println!(
        "cargo:rerun-if-changed={}",
        manifest.join("../../apps/web").display()
    );

    let mut files = Vec::new();
    collect(&dist, &dist, &mut files);
    files.sort_by_key(|(name, _)| name.clone());

    let mut generated = String::from("pub static FILES: &[(&str, &[u8])] = &[\n");
    for (name, path) in files {
        let _ = writeln!(generated, "    ({name:?}, include_bytes!({path:?})),");
    }
    generated.push_str("];\n");

    std::fs::write(out.join("spa_assets.rs"), generated).expect("write embedded SPA table");
}

fn collect(root: &Path, dir: &Path, files: &mut Vec<(String, PathBuf)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(root, &path, files);
        } else if let Ok(relative) = path.strip_prefix(root) {
            files.push((relative.to_string_lossy().replace('\\', "/"), path));
        }
    }
}
