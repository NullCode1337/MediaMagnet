use std::path::PathBuf;
use std::sync::Arc;

use mm_core::Engine;

use crate::assets::StaticAssets;
use crate::events::EventHub;
use crate::jobs::Jobs;
use crate::retention::Retention;

pub struct AppState {
    pub engine: Arc<Engine>,
    pub events: EventHub,
    pub download_root: PathBuf,
    /// `None` means the server was started without `MM_TOKEN`: the API is open.
    pub token: Option<String>,
    pub static_assets: StaticAssets,
    pub jobs: Arc<Jobs>,
    pub retention: Retention,
}
