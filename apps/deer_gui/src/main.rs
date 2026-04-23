//! Deer GUI — Bevy application entry point.
//!
//! Bootstraps the Bevy engine with window configuration, logging,
//! diagnostics, EGui overlay, and the orbital camera system.

use bevy::app::App;
use bevy::log::{info, LogPlugin};
use bevy::prelude::*;
use bevy::window::{Window, WindowResolution};
use bevy_egui::EguiPlugin;
use clap::Parser;

use deer_gui::audio::DeerAudioPlugin;
use deer_gui::bridge::BridgePlugin;
use deer_gui::building::BuildingPlugin;
use deer_gui::camera::CameraPlugin;
use deer_gui::constants::window::*;
use deer_gui::diagnostics::DiagnosticsPlugin;
use deer_gui::hud::HudPlugin;
use deer_gui::picking::PickingPlugin as DeerPickingPlugin;
use deer_gui::preferences::PreferencesPlugin;
use deer_gui::render::{CapabilityQualityPlugin, RenderQualityPlugin};
use deer_gui::scene::{InitialScene, ScenePlugin};
use deer_gui::shell::ShellPlugin;
use deer_gui::theme::ThemePlugin;
use deer_gui::world::{FoliagePlugin, WaterGlobalConfig, WaterPlugin, WorldPlugin};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the scene to load on startup
    #[arg(short, long, default_value = "medieval_open")]
    scene: String,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    load_dotenv();

    let log_filter = build_log_filter();
    let args = Args::parse();

    App::new()
        .insert_resource(InitialScene(args.scene))
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
                    level: bevy::log::Level::INFO,
                    ..default()
                }),
        )
        .add_plugins(EguiPlugin::default())
        .add_plugins(RenderQualityPlugin::default())
        .add_plugins(CapabilityQualityPlugin)
        .add_plugins(ThemePlugin)
        .add_plugins(PreferencesPlugin)
        .add_plugins(BuildingPlugin::default())
        .add_plugins(DiagnosticsPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(ScenePlugin)
        .add_plugins(BridgePlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(FoliagePlugin::default())
        .add_plugins(WaterPlugin {
            config: Some(WaterGlobalConfig {
                water_level: 10.0, // River at 10m elevation
                enable_splash: true,
                enable_swimming: true,
                ..default()
            }),
        })
        .add_systems(Startup, setup_landscape)
        .add_plugins(ShellPlugin)
        .add_plugins(HudPlugin)
        .add_plugins(DeerPickingPlugin)
        .add_plugins(DeerAudioPlugin)
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

/// Sets up the initial landscape scene.
fn setup_landscape(mut commands: Commands) {
    info!("Setting up medieval landscape...");

    // Add directional light (sun)
    commands.spawn((
        Name::new("Sun"),
        DirectionalLight {
            color: Color::srgb(1.0, 0.95, 0.85),
            illuminance: 100_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(100.0, 200.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Add ambient light component to the camera or use default

    info!("Landscape setup complete!");
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
