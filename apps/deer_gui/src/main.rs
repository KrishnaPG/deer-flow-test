mod app;
mod bridge;
mod models;

use app::DeerGuiApp;

fn main() -> eframe::Result<()> {
    // Load .env file (if present) before anything else.
    // Variables already set in the environment are NOT overwritten,
    // so `OPENAI_API_KEY=xxx cargo run` still takes precedence.
    //
    // Search order: ./apps/deer_gui/.env (CARGO_MANIFEST_DIR), then cwd.
    let manifest_env = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
    if manifest_env.exists() {
        dotenvy::from_path(&manifest_env).ok();
    } else {
        dotenvy::dotenv().ok(); // try cwd
    }

    // Logging controlled by env vars (both work):
    //   DEER_GUI_LOG=debug   — app-specific, recommended
    //   RUST_LOG=deer_gui=debug  — standard Rust convention
    //
    // Levels: error, warn, info, debug, trace
    //   trace = includes raw JSON protocol dumps
    //   debug = command/event flow
    //   info  = lifecycle events (default)
    //
    // DEER_GUI_LOG only sets the level for deer_gui modules (not wgpu/naga/eframe).
    // Use RUST_LOG for fine-grained control over all crates.
    let mut builder = env_logger::Builder::new();
    builder
        .filter_level(log::LevelFilter::Warn) // default: only warnings+ for all crates
        .format_timestamp_millis();

    if let Ok(rust_log) = std::env::var("RUST_LOG") {
        // Full control: user specified RUST_LOG directly
        builder.parse_filters(&rust_log);
    }

    if let Ok(app_level) = std::env::var("DEER_GUI_LOG") {
        // App-specific: only apply to deer_gui modules, keep others at warn
        let filter = format!("deer_gui={app_level}");
        builder.parse_filters(&filter);
    } else if std::env::var("RUST_LOG").is_err() {
        // No log env set at all: default deer_gui to info
        builder.parse_filters("deer_gui=info");
    }

    builder.init();

    log::info!("Starting Deer GUI");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1480.0, 920.0])
            .with_min_inner_size([1080.0, 760.0])
            .with_title("Deer GUI"),
        ..Default::default()
    };

    eframe::run_native(
        "Deer GUI",
        options,
        Box::new(|cc| Ok(Box::new(DeerGuiApp::new(cc)))),
    )
}
