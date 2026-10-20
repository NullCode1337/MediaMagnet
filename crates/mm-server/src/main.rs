//! MediaMagnet HTTP server: serves `/api/v1` and the built SPA from one process.

mod auth;
mod events;
mod files;
mod routes;
mod state;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use mm_core::{Engine, Policy, RuntimeConfig, ToolRuntime};

use crate::state::AppState;

fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    runtime.block_on(async_main());
}

pub struct Config {
    pub bind: SocketAddr,
    pub data_dir: PathBuf,
    pub download_root: PathBuf,
    pub static_dir: PathBuf,
    pub token: Option<String>,
    pub allow_custom_args: bool,
}

fn env_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name).map(PathBuf::from)
}

impl Config {
    fn from_env() -> Result<Self, String> {
        let bind = std::env::var("MM_BIND")
            .unwrap_or_else(|_| "0.0.0.0:8080".into())
            .parse()
            .map_err(|e| format!("MM_BIND is not a valid socket address: {e}"))?;

        let data_dir = env_path("MM_DATA_DIR")
            .or_else(|| Some(default_data_dir()))
            .ok_or("could not determine data directory; set MM_DATA_DIR")?;

        let download_root =
            env_path("MM_DOWNLOAD_ROOT").unwrap_or_else(|| data_dir.join("downloads"));

        let static_dir = env_path("MM_STATIC_DIR")
            .or_else(|| {
                std::env::current_exe()
                    .ok()
                    .and_then(|exe| exe.parent().map(|p| p.join("public")))
            })
            .unwrap_or_else(|| PathBuf::from("public"));

        let token = std::env::var("MM_TOKEN")
            .ok()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty());

        let allow_custom_args = std::env::var("MM_ALLOW_CUSTOM_ARGS")
            .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
            .unwrap_or(false);

        Ok(Self {
            bind,
            data_dir,
            download_root,
            static_dir,
            token,
            allow_custom_args,
        })
    }
}

fn default_data_dir() -> PathBuf {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("mediamagnet")
}

async fn async_main() {
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("[MediaMagnet][Server] {e}");
            std::process::exit(2);
        }
    };

    if let Err(e) = serve(config).await {
        eprintln!("[MediaMagnet][Server] {e}");
        std::process::exit(1);
    }
}

async fn serve(config: Config) -> Result<(), String> {
    let config_dir = config.data_dir.join("config");
    let app_data_dir = config.data_dir.join("data");

    for dir in [
        config_dir.clone(),
        app_data_dir.clone(),
        app_data_dir.join("cookies"),
        config.download_root.clone(),
    ] {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("could not create {}: {e}", dir.display()))?;
    }

    let settings = mm_core::Settings::load_from(&config_dir);
    settings
        .save_to(&config_dir)
        .map_err(|e| format!("could not write settings: {e}"))?;

    let runtime = Arc::new(ToolRuntime::new(RuntimeConfig {
        bundled_tools_dir: env_path("MM_BINARIES_DIR"),
    }));

    let events = crate::events::EventHub::new();
    let policy = if config.allow_custom_args {
        Policy::server_with_custom_args()
    } else {
        Policy::server()
    };

    let engine = Engine::new(runtime, config_dir, app_data_dir, policy, events.sink());

    let state = Arc::new(AppState {
        engine,
        events,
        download_root: config.download_root.clone(),
        token: config.token.clone(),
        static_dir: config.static_dir.clone(),
    });

    println!(
        "[MediaMagnet][Server] v{} listening on http://{}",
        env!("CARGO_PKG_VERSION"),
        config.bind
    );
    println!(
        "[MediaMagnet][Server] download root: {}",
        config.download_root.display()
    );
    println!(
        "[MediaMagnet][Server] static dir:    {}",
        config.static_dir.display()
    );
    println!(
        "[MediaMagnet][Server] auth: {}",
        match &config.token {
            Some(_) => "bearer token required",
            None => "open (set MM_TOKEN to require one)",
        }
    );

    let app = routes::router(state);
    let listener = tokio::net::TcpListener::bind(config.bind)
        .await
        .map_err(|e| format!("could not bind {}: {e}", config.bind))?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| format!("server error: {e}"))
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    println!("[MediaMagnet][Server] shutting down");
}
