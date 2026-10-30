//! MediaMagnet core: settings, cookies, downloads and tool discovery with no UI
//! framework dependencies, shared by the desktop app and the HTTP server.

pub mod cookies;
pub mod download;
pub mod error;
pub mod events;
pub mod files;
pub mod host;
pub mod policy;
pub mod runtime;
pub mod settings;

pub use download::{Backend, DownloadRequest, Engine, JobCookie};
pub use error::{Error, Result};
pub use events::{
    CoreEvent, DownloadState, EventSink, NotificationLevel, NullEventSink, SharedEventSink,
};
pub use policy::{ArgTool, Policy};
pub use runtime::{
    ResolvedTool, RuntimeConfig, Tool, ToolError, ToolRuntime, ToolSource, ToolStatus,
};
pub use settings::{force_desktop_fields, Settings};
