use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Component, Path};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{Error, Result};

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum CookieInput {
    FilePath(String),
    Content(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CookieProfile {
    pub domain: String,
    pub path: String,
}

pub fn is_netscape(file_path: &Path) -> Result<bool> {
    let file = std::fs::File::open(file_path).map_err(|e| {
        Error::Msg(format!(
            "[MediaMagnet][Utils] Failed to open cookie file '{}': {}",
            file_path.display(),
            e
        ))
    })?;

    let reader = BufReader::new(file);
    let mut has_header = false;
    let mut has_valid_cookie = false;

    for (line_num, result) in reader.lines().enumerate() {
        let line = result
            .map_err(|e| Error::Msg(format!("Failed to read line {}: {}", line_num + 1, e)))?;

        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("# Netscape") || trimmed.starts_with("# HTTP Cookie File") {
            has_header = true;
            continue;
        }

        if trimmed.starts_with('#') && !trimmed.to_lowercase().starts_with("#httponly_") {
            continue;
        }

        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() == 7 {
            has_valid_cookie = true;
            if has_header {
                break;
            }
        }
    }

    Ok(has_header || has_valid_cookie)
}

pub fn netscape_from_json(content: &str) -> Result<String> {
    let json_data: Value = serde_json::from_str(content)
        .map_err(|e| Error::Msg(format!("[MediaMagnet][Utils] Invalid JSON format: {}", e)))?;

    let cookies = match json_data {
        Value::Array(arr) => arr,
        Value::Object(obj) => {
            if let Some(Value::Array(cookies)) = obj.get("cookies") {
                cookies.clone()
            } else {
                vec![Value::Object(obj)]
            }
        }
        _ => return Err(Error::Msg("JSON must be an array or object".to_string())),
    };

    let mut output = Vec::new();
    {
        let mut output = BufWriter::new(&mut output);
        writeln!(output, "# Netscape HTTP Cookie File").map_err(write_err)?;
        writeln!(
            output,
            "# This file was generated from JSON by MediaMagnet\n"
        )
        .map_err(write_err)?;

        for cookie in cookies {
            if let Value::Object(c) = cookie {
                let domain = c.get("domain").and_then(|v| v.as_str()).unwrap_or("");
                let flag = if domain.starts_with('.') {
                    "TRUE"
                } else {
                    "FALSE"
                };
                let path = c.get("path").and_then(|v| v.as_str()).unwrap_or("/");
                let secure = c
                    .get("secure")
                    .and_then(|v| v.as_bool())
                    .map(|b| if b { "TRUE" } else { "FALSE" })
                    .unwrap_or("FALSE");
                let expiration = c
                    .get("expirationDate")
                    .or_else(|| c.get("expires"))
                    .and_then(|v| v.as_f64().map(|f| f as i64))
                    .unwrap_or(0);
                let name = c.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let value = c.get("value").and_then(|v| v.as_str()).unwrap_or("");
                let http_only = c.get("httpOnly").and_then(|v| v.as_bool()).unwrap_or(false);
                let prefix = if http_only { "#HttpOnly_" } else { "" };

                writeln!(
                    output,
                    "{}{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    prefix, domain, flag, path, secure, expiration, name, value
                )
                .map_err(write_err)?;
            }
        }
        output.flush().map_err(write_err)?;
    }

    String::from_utf8(output).map_err(|e| Error::Msg(format!("Invalid UTF-8: {e}")))
}

pub fn convert_json(source_path: &Path, dest_path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(source_path).map_err(|e| {
        Error::Msg(format!(
            "[MediaMagnet][Utils] Failed to read JSON file: {}",
            e
        ))
    })?;
    let netscape = netscape_from_json(&content)?;
    std::fs::write(dest_path, netscape).map_err(|e| {
        Error::Msg(format!(
            "[MediaMagnet][Utils] Failed to create destination cookie file: {}",
            e
        ))
    })
}

fn write_err(e: std::io::Error) -> Error {
    Error::Msg(format!(
        "[MediaMagnet][Utils] Failed to write cookie: {}",
        e
    ))
}

/// Domain names become file names; anything that could escape the cookies directory is refused.
pub fn validate_domain(domain: &str) -> Result<String> {
    let domain = domain.trim();
    if domain.is_empty() {
        return Err(Error::Msg("cookie domain is empty".into()));
    }
    if domain.contains("..")
        || domain.contains('/')
        || domain.contains('\\')
        || domain.contains('\0')
        || domain.chars().any(char::is_whitespace)
        || Path::new(domain).components().count() != 1
        || !matches!(
            Path::new(domain).components().next(),
            Some(Component::Normal(_))
        )
    {
        return Err(Error::Msg(format!("invalid cookie domain: {domain}")));
    }
    Ok(domain.to_string())
}

pub fn cookie_path(dir: &Path, domain: &str) -> Result<std::path::PathBuf> {
    Ok(dir.join(format!("{}.txt", validate_domain(domain)?)))
}

pub fn save_cookie(dir: &Path, domain: &str, input: &CookieInput) -> Result<String> {
    let dest_path = cookie_path(dir, domain)?;

    match input {
        CookieInput::FilePath(file_path_str) => {
            let file_path = Path::new(file_path_str);
            if !file_path.exists() {
                return Err(Error::Msg(format!(
                    "Source file does not exist: {}",
                    file_path.display()
                )));
            }

            let file_extension = file_path
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("")
                .to_lowercase();

            match file_extension.as_str() {
                "txt" | "cookies" => {
                    if !is_netscape(file_path)? {
                        return Err(Error::Msg(
                            "File is not a valid Netscape format cookie file".to_string(),
                        ));
                    }
                    std::fs::copy(file_path, &dest_path)
                        .map_err(|e| Error::Msg(format!("Failed to copy cookie file: {}", e)))?;
                }
                "json" => {
                    convert_json(file_path, &dest_path)?;
                }
                _ => {
                    return Err(Error::Msg("Unsupported file format. Only .txt, .cookies (Netscape), and .json files are supported".to_string()));
                }
            }
        }

        CookieInput::Content(content) => {
            let is_json =
                content.trim_start().starts_with('{') || content.trim_start().starts_with('[');

            if is_json {
                let netscape = netscape_from_json(content)?;
                std::fs::write(&dest_path, netscape).map_err(|e| Error::Msg(e.to_string()))?;
            } else {
                std::fs::write(&dest_path, content).map_err(|e| Error::Msg(e.to_string()))?;

                if !is_netscape(&dest_path)? {
                    let _ = std::fs::remove_file(&dest_path);
                    return Err(Error::Msg(
                        "Invalid format. Please provide Netscape/JSON cookies or cookie file path"
                            .to_string(),
                    ));
                }
            }
        }
    }

    Ok(dest_path.to_string_lossy().into_owned())
}

pub fn list_cookies(dir: &Path) -> Result<BTreeMap<String, String>> {
    let mut cookie_files = BTreeMap::new();
    let entries = std::fs::read_dir(dir)
        .map_err(|e| Error::Msg(format!("Failed to read cookies directory: {}", e)))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                cookie_files.insert(file_stem.to_string(), path.to_string_lossy().to_string());
            }
        }
    }
    Ok(cookie_files)
}

pub fn clear_cookies(dir: &Path) -> Result<usize> {
    let mut deleted_count = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && std::fs::remove_file(&path).is_ok() {
                deleted_count += 1;
            }
        }
    }
    Ok(deleted_count)
}

/// `delete_cookie(path)` must stay inside the cookies directory.
pub fn delete_cookie(dir: &Path, path: &Path) -> Result<()> {
    let canonical_dir = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
    let target = if path.is_absolute() {
        path.to_path_buf()
    } else {
        dir.join(path)
    };
    let canonical_target = target
        .canonicalize()
        .map_err(|e| Error::Msg(format!("Unable to delete cookie file: {}", e)))?;
    if !canonical_target.starts_with(&canonical_dir) {
        return Err(Error::Msg(
            "Unable to delete cookie file: outside cookie store".into(),
        ));
    }
    std::fs::remove_file(&canonical_target)
        .map_err(|e| Error::Msg(format!("Unable to delete cookie file: {}", e)))
}

/// First cookie profile whose file stem appears in the URL, in directory order.
pub fn match_cookie(dir: &Path, link: &str) -> Option<std::path::PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    let link = link.to_lowercase();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                if link.contains(&file_stem.to_lowercase()) && path.to_str().is_some() {
                    return Some(path);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_traversal_domains() {
        for bad in ["..", "../../etc/passwd", "a/b", "a\\b", "", "  ", "a b"] {
            assert!(validate_domain(bad).is_err(), "{bad:?} accepted");
        }
        assert_eq!(validate_domain("youtube").unwrap(), "youtube");
        assert_eq!(validate_domain("google.com").unwrap(), "google.com");
    }

    #[test]
    fn save_and_match_content() {
        let dir = std::env::temp_dir().join(format!("mm-cookies-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let content = "# Netscape HTTP Cookie File\n.youtube.com\tTRUE\t/\tTRUE\t0\tname\tvalue\n";
        save_cookie(&dir, "youtube", &CookieInput::Content(content.to_string())).unwrap();
        assert!(list_cookies(&dir).unwrap().contains_key("youtube"));
        let matched = match_cookie(&dir, "https://WWW.youtube.com/watch").unwrap();
        assert_eq!(matched.file_name().unwrap(), "youtube.txt");
        assert!(match_cookie(&dir, "https://vimeo.com/1").is_none());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn delete_outside_store_is_refused() {
        let dir = std::env::temp_dir().join(format!("mm-cookies-del-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let outside =
            std::env::temp_dir().join(format!("mm-cookies-outside-{}", std::process::id()));
        std::fs::write(&outside, "x").unwrap();
        assert!(delete_cookie(&dir, &outside).is_err());
        assert!(outside.exists());
        std::fs::remove_file(&outside).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn json_converts_to_netscape() {
        let json = r#"[{"domain":".example.com","path":"/","secure":true,"expirationDate":1700000000,"name":"a","value":"b","httpOnly":true}]"#;
        let out = netscape_from_json(json).unwrap();
        assert!(out.starts_with("# Netscape HTTP Cookie File"));
        assert!(out.contains("#HttpOnly_.example.com\tTRUE\t/\tTRUE\t1700000000\ta\tb"));
    }
}
