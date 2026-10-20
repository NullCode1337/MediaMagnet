use std::path::Path;
use std::pin::Pin;

use axum::body::Body;
use axum::extract::{Path as UrlPath, Query, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use futures_util::Stream;
use serde::Deserialize;
use std::sync::Arc;
use tokio_util::io::ReaderStream;

use mm_core::files::{parse_range, safe_join, DirEntry, EntryKind};

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    pub path: String,
}

#[derive(Debug, serde::Serialize)]
pub struct ListResponse {
    pub path: String,
    pub entries: Vec<EntryPayload>,
}

#[derive(Debug, serde::Serialize)]
pub struct EntryPayload {
    pub path: String,
    pub name: String,
    pub kind: &'static str,
    pub size: u64,
    /// Download URL, relative to the API root.
    pub url: String,
}

impl From<DirEntry> for EntryPayload {
    fn from(entry: DirEntry) -> Self {
        let kind = match entry.kind {
            EntryKind::Directory => "directory",
            EntryKind::File => "file",
        };
        Self {
            url: format!("/api/v1/files/{}", encode_path(&entry.path)),
            path: entry.path,
            name: entry.name,
            kind,
            size: entry.size,
        }
    }
}

fn encode_path(relative: &str) -> String {
    relative
        .split('/')
        .map(|segment| {
            segment
                .bytes()
                .map(|b| match b {
                    b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                        (b as char).to_string()
                    }
                    other => format!("%{other:02X}"),
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn not_found(message: &str) -> Response {
    (
        StatusCode::NOT_FOUND,
        axum::Json(serde_json::json!({ "error": "not_found", "message": message })),
    )
        .into_response()
}

pub async fn list(State(state): State<Arc<AppState>>, Query(query): Query<ListQuery>) -> Response {
    match mm_core::files::list_dir(&state.download_root, &query.path) {
        Ok(entries) => {
            let entries: Vec<EntryPayload> = entries.into_iter().map(Into::into).collect();
            axum::Json(ListResponse {
                path: query.path,
                entries,
            })
            .into_response()
        }
        Err(e) => not_found(&e.to_string()),
    }
}

fn content_type(path: &Path) -> HeaderValue {
    mime_guess::from_path(path)
        .first_raw()
        .and_then(|m| HeaderValue::from_str(m).ok())
        .unwrap_or_else(|| HeaderValue::from_static("application/octet-stream"))
}

/// Serves one file, honouring a single byte range so media players can seek.
pub async fn download(
    State(state): State<Arc<AppState>>,
    UrlPath(relative): UrlPath<String>,
    headers: HeaderMap,
) -> Response {
    let relative = relative.replace('\\', "/");

    let path = match safe_join(&state.download_root, &relative) {
        Ok(path) => path,
        Err(e) => return not_found(&e.to_string()),
    };

    let meta = match tokio::fs::metadata(&path).await {
        Ok(meta) if meta.is_file() => meta,
        _ => return not_found("no such file"),
    };
    let len = meta.len();

    let range = match headers.get(header::RANGE).and_then(|v| v.to_str().ok()) {
        Some(value) => match parse_range(value, len) {
            Ok(range) => range,
            Err(e) => {
                return (
                    StatusCode::RANGE_NOT_SATISFIABLE,
                    [(header::CONTENT_RANGE, format!("bytes */{len}"))],
                    axum::Json(
                        serde_json::json!({ "error": "bad_range", "message": e.to_string() }),
                    ),
                )
                    .into_response();
            }
        },
        None => None,
    };

    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "download".into());

    let mut builder = Response::builder()
        .header(header::CONTENT_TYPE, content_type(&path))
        .header(header::ACCEPT_RANGES, "bytes")
        .header(
            header::CONTENT_DISPOSITION,
            format!("inline; filename*=UTF-8''{}", encode_path(&file_name)),
        );

    let file = match tokio::fs::File::open(&path).await {
        Ok(file) => file,
        Err(_) => return not_found("no such file"),
    };

    let (start, end) = match range {
        Some(range) => {
            builder = builder
                .status(StatusCode::PARTIAL_CONTENT)
                .header(
                    header::CONTENT_RANGE,
                    format!("bytes {}-{}/{}", range.start, range.end, len),
                )
                .header(header::CONTENT_LENGTH, range.len());
            (range.start, range.end)
        }
        None => {
            builder = builder
                .status(StatusCode::OK)
                .header(header::CONTENT_LENGTH, len);
            (0, len.saturating_sub(1))
        }
    };

    let stream: Pin<Box<dyn Stream<Item = Result<axum::body::Bytes, std::io::Error>> + Send>> =
        if len == 0 || end < start {
            Box::pin(futures_util::stream::empty())
        } else {
            use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};
            let mut file = file;
            if start > 0 && file.seek(SeekFrom::Start(start)).await.is_err() {
                return not_found("no such file");
            }
            Box::pin(ReaderStream::new(file.take(end - start + 1)))
        };

    builder
        .body(Body::from_stream(stream))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoding_keeps_slashes_and_escapes_others() {
        assert_eq!(encode_path("a/b c.mp4"), "a/b%20c.mp4");
        assert_eq!(encode_path("ünï.txt"), "%C3%BCn%C3%AF.txt");
    }
}
