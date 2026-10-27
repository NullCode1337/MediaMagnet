use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use axum::body::Body;
use axum::extract::{Path as UrlPath, Query, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use futures_util::Stream;
use serde::Deserialize;
use tokio_util::io::ReaderStream;

use mm_core::files::{parse_range, safe_join, DirEntry, EntryKind};

use crate::retention::{FileIdentity, Retention};
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

/// `inline` lets a media player or browser preview a file; `attachment` makes
/// the browser save it, which is what a completed job's output wants.
#[derive(Clone, Copy)]
pub enum Disposition {
    Inline,
    Attachment,
}

impl Disposition {
    fn as_str(self) -> &'static str {
        match self {
            Self::Inline => "inline",
            Self::Attachment => "attachment",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct FileQuery {
    /// `?download=1` (or `true`/`yes`) forces an attachment disposition.
    #[serde(default)]
    pub download: Option<String>,
}

/// Serves one file, honouring a single byte range so media players can seek.
pub async fn download(
    State(state): State<Arc<AppState>>,
    UrlPath(relative): UrlPath<String>,
    Query(query): Query<FileQuery>,
    headers: HeaderMap,
) -> Response {
    let disposition = if matches!(query.download.as_deref(), Some("1" | "true" | "yes")) {
        Disposition::Attachment
    } else {
        Disposition::Inline
    };
    serve(&state.download_root, &relative, headers, disposition, None).await
}

/// Deletes a job's output after the whole body has been streamed to the client.
pub struct DeleteAfter {
    pub retention: Retention,
    pub job_id: String,
}

/// Streams `relative` (inside `root`), with the same range handling for both
/// `/files/{path}` and a finished job's output file.
pub async fn serve(
    root: &Path,
    relative: &str,
    headers: HeaderMap,
    disposition: Disposition,
    delete_after: Option<DeleteAfter>,
) -> Response {
    let relative = relative.replace('\\', "/");

    let path = match safe_join(root, &relative) {
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
            format!(
                "{}; filename*=UTF-8''{}",
                disposition.as_str(),
                encode_path(&file_name)
            ),
        );

    let file = match tokio::fs::File::open(&path).await {
        Ok(file) => file,
        Err(_) => return not_found("no such file"),
    };
    // Taken from the handle, not the path: if another job replaces the file
    // while this response is streaming, cleanup must not delete the newcomer.
    let opened_meta = match file.metadata().await {
        Ok(meta) => meta,
        Err(_) => return not_found("no such file"),
    };
    let full_response = range.is_none();

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

    // Output files disappear once a client has received them in full. A range
    // request may be one seek in a longer session, so it leaves the file for
    // the expiry sweep instead.
    let stream = match delete_after {
        Some(delete) if full_response => {
            let cleanup = DeleteOnComplete {
                retention: delete.retention,
                job_id: Some(delete.job_id),
                relative,
                identity: FileIdentity::from_metadata(&opened_meta),
                delivered: 0,
                expected: len,
                armed: true,
            };
            Box::pin(BodyWithCleanup {
                inner: stream,
                cleanup,
            })
                as Pin<Box<dyn Stream<Item = Result<axum::body::Bytes, std::io::Error>> + Send>>
        }
        _ => stream,
    };

    builder
        .body(Body::from_stream(stream))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

struct DeleteOnComplete {
    retention: Retention,
    job_id: Option<String>,
    relative: String,
    identity: FileIdentity,
    delivered: u64,
    expected: u64,
    armed: bool,
}

impl DeleteOnComplete {
    fn complete(&mut self) {
        if !self.armed || self.delivered < self.expected {
            return;
        }
        self.armed = false;
        let Some(job_id) = self.job_id.take() else {
            return;
        };
        let retention = self.retention.clone();
        let relative = std::mem::take(&mut self.relative);
        let identity = self.identity;
        tokio::spawn(async move {
            retention
                .delete_delivered(&job_id, &relative, identity)
                .await;
        });
    }
}

struct BodyWithCleanup {
    inner: Pin<Box<dyn Stream<Item = Result<axum::body::Bytes, std::io::Error>> + Send>>,
    cleanup: DeleteOnComplete,
}

impl Stream for BodyWithCleanup {
    type Item = Result<axum::body::Bytes, std::io::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        match this.inner.as_mut().poll_next(cx) {
            Poll::Ready(Some(Ok(bytes))) => {
                this.cleanup.delivered = this.cleanup.delivered.saturating_add(bytes.len() as u64);
                // Hyper stops polling once `Content-Length` bytes are out, so
                // the last chunk is the completion signal, not `None`.
                this.cleanup.complete();
                Poll::Ready(Some(Ok(bytes)))
            }
            Poll::Ready(Some(Err(e))) => {
                this.cleanup.armed = false;
                Poll::Ready(Some(Err(e)))
            }
            Poll::Ready(None) => {
                this.cleanup.complete();
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoding_keeps_slashes_and_escapes_others() {
        assert_eq!(encode_path("a/b c.mp4"), "a/b%20c.mp4");
        assert_eq!(encode_path("ünï.txt"), "%C3%BCn%C3%AF.txt");
    }

    #[tokio::test]
    async fn a_fully_streamed_body_deletes_the_output() {
        use futures_util::StreamExt;
        use std::time::{Duration, SystemTime, UNIX_EPOCH};

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("mm-files-cleanup-{unique}"));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("clip.mp4");
        std::fs::write(&path, b"hello").unwrap();

        let retention = Retention::open(
            root.clone(),
            Duration::from_secs(3600),
            root.join("job-files.json"),
        );
        retention.record("job", &["clip.mp4".into()]);

        let file = tokio::fs::File::open(&path).await.unwrap();
        let identity = FileIdentity::from_metadata(&file.metadata().await.unwrap());
        let inner: Pin<Box<dyn Stream<Item = Result<axum::body::Bytes, std::io::Error>> + Send>> =
            Box::pin(ReaderStream::new(file));
        let mut body = BodyWithCleanup {
            inner,
            cleanup: DeleteOnComplete {
                retention,
                job_id: Some("job".into()),
                relative: "clip.mp4".into(),
                identity,
                delivered: 0,
                expected: 5,
                armed: true,
            },
        };

        while let Some(chunk) = body.next().await {
            chunk.unwrap();
        }

        // Deletion happens in a task spawned from the stream.
        tokio::time::timeout(Duration::from_secs(1), async {
            while path.exists() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert!(!path.exists());
        let _ = std::fs::remove_dir_all(&root);
    }
}
