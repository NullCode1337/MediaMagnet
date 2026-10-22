//! Standalone MediaMagnet server binary.

fn main() {
    let config = match mm_server::Config::from_env() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("[MediaMagnet][Server] {e}");
            std::process::exit(2);
        }
    };

    if let Err(e) = mm_server::run_blocking(config) {
        eprintln!("[MediaMagnet][Server] {e}");
        std::process::exit(1);
    }
}
