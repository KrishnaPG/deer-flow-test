//! Deer GUI — Bevy application entry point.
//!
//! Bootstraps the Bevy engine with window configuration, logging,
//! diagnostics, EGui overlay, and the orbital camera system.

mod camera;
mod constants;
mod diagnostics;
mod models;

use bevy::app::App;
use bevy::log::{info, LogPlugin};
use bevy::prelude::*;
use bevy::window::{Window, WindowResolution};
use bevy_egui::EguiPlugin;

use crate::camera::CameraPlugin;
use crate::constants::window::*;
use crate::diagnostics::DiagnosticsPlugin;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    load_dotenv();

    let log_filter = build_log_filter();

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: TITLE.to_string(),
                        resolution: WindowResolution::new(DEFAULT_WIDTH, DEFAULT_HEIGHT)
                            .with_scale_factor_override(1.0),
                        resize_constraints: WindowResizeConstraints {
                            min_width: MIN_WIDTH,
                            min_height: MIN_HEIGHT,
                            ..default()
                        },
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    filter: log_filter,
                    level: bevy::log::Level::TRACE,
                    ..default()
                }),
        )
        .add_plugins(EguiPlugin::default())
        .add_plugins(DiagnosticsPlugin)
        .add_plugins(CameraPlugin)
        .run();
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Loads `.env` from the crate manifest directory first, falling back to cwd.
///
/// Variables already present in the environment are **not** overwritten,
/// so `DEER_GUI_LOG=debug cargo run` always takes precedence.
fn load_dotenv() {
    let manifest_env = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
    if manifest_env.exists() {
        info!("main::load_dotenv — loading {}", manifest_env.display());
        dotenvy::from_path(&manifest_env).ok();
    } else {
        info!("main::load_dotenv — no manifest .env, trying cwd");
        dotenvy::dotenv().ok();
    }
}

/// Builds the `tracing` filter string for [`LogPlugin`].
///
/// Priority (highest wins):
/// 1. `RUST_LOG` — full control, passed through verbatim.
/// 2. `DEER_GUI_LOG` — sets only the `deer_gui` crate level;
///    noisy crates (`wgpu`, `naga`) stay quiet.
/// 3. Fallback — `deer_gui=info,wgpu=error,naga=warn`.
fn build_log_filter() -> String {
    if let Ok(rust_log) = std::env::var("RUST_LOG") {
        eprintln!("[deer_gui] Using RUST_LOG filter: {rust_log}");
        return rust_log;
    }

    if let Ok(level) = std::env::var("DEER_GUI_LOG") {
        let filter = format!("deer_gui={level},wgpu=error,naga=warn");
        eprintln!("[deer_gui] Using DEER_GUI_LOG filter: {filter}");
        return filter;
    }

    let filter = "deer_gui=info,wgpu=error,naga=warn".to_string();
    eprintln!("[deer_gui] Using default log filter: {filter}");
    filter
}
