//! [`CameraPlugin`] — registers startup and per-frame camera systems.

use bevy::core_pipeline::prepass::{DepthPrepass, NormalPrepass};
use bevy::light::GlobalAmbientLight;
use bevy::log::{debug, info};
use bevy::math::Vec3;
use bevy::pbr::{DistanceFog, FogFalloff, ScreenSpaceAmbientOcclusion};
use bevy::prelude::{
    App, Camera3d, Color, Commands, DirectionalLight, IntoScheduleConfigs, Msaa, Plugin, Query,
    Res, Startup, Transform, Update, With,
};
use bevy::render::view::Hdr;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use super::components::CinematicCamera;
use super::navigation::{camera_sync_snapshot_system, CameraSyncState};
use super::systems::{
    camera_focus_system, camera_input_system, camera_interpolation_system,
    camera_mode_toggle_system, camera_preferences_sync_system, camera_shake_system,
    first_person_movement_system, mouse_look_system, viewport_navigation_system,
};
use crate::constants::camera::ORBIT_RADIUS;
use crate::preferences::UserPreferences;
use crate::scene::generators::atmosphere::SceneFogConfig;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Registers all camera-related systems.
///
/// **Startup:** spawns the camera entity and basic scene lighting.
/// **Update (chained):** input → interpolation → shake → focus.
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        info!("CameraPlugin::build — registering camera systems");

        app.add_systems(Startup, camera_spawn_system)
            .init_resource::<CameraSyncState>()
            .add_systems(
                Update,
                (
                    camera_mode_toggle_system,
                    camera_preferences_sync_system,
                    camera_input_system,
                    mouse_look_system,
                    first_person_movement_system,
                    camera_interpolation_system,
                    camera_shake_system,
                    viewport_navigation_system,
                    camera_focus_system,
                    camera_sync_snapshot_system,
                )
                    .chain(),
            )
            .add_systems(EguiPrimaryContextPass, camera_debug_overlay);
    }
}

// ---------------------------------------------------------------------------
// Spawn system
// ---------------------------------------------------------------------------

/// Spawns the primary camera entity with [`CinematicCamera`] state,
/// HDR, SSAO, and optional distance fog.
fn camera_spawn_system(
    mut commands: Commands,
    prefs: Option<Res<UserPreferences>>,
    fog_config: Option<Res<SceneFogConfig>>,
) {
    let mut cam = CinematicCamera::default();
    // Apply saved camera mode from user preferences if available.
    if let Some(prefs) = prefs {
        cam.mode = prefs.camera_mode;
        cam.target_mode = prefs.camera_mode;
    }
    cam.mode_transition = 1.0; // No transition.

    let look_at = cam.compute_look_at();
    let offset = cam.compute_position(ORBIT_RADIUS);
    let position = look_at + offset;

    debug!(
        "camera_spawn: pos={:?} look_at={:?} mode={:?}",
        position, look_at, cam.mode
    );

    // Build camera bundle.
    let camera_bundle = (
        Camera3d::default(),
        Hdr::default(),                         // Enable HDR rendering
        Msaa::Off,                              // Required for SSAO
        DepthPrepass,                           // Required for SSAO
        NormalPrepass,                          // Required for SSAO
        ScreenSpaceAmbientOcclusion::default(), // Enable SSAO
        cam,
        Transform::from_translation(position).looking_at(look_at, Vec3::Y),
    );

    // Add distance fog if configured by the active scene.
    if let Some(config) = fog_config {
        if config.enabled {
            commands.spawn((
                camera_bundle,
                DistanceFog {
                    color: config.color,
                    falloff: FogFalloff::Exponential {
                        density: config.density,
                    },
                    ..Default::default()
                },
            ));
            info!(
                "camera_spawn: camera with distance fog (density={})",
                config.density
            );
        } else {
            commands.spawn(camera_bundle);
        }
    } else {
        commands.spawn(camera_bundle);
    }

    // Fallback directional light for scenes without atmosphere generator.
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_translation(Vec3::new(50.0, 200.0, 100.0)).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Fallback ambient fill light.
    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
        affects_lightmapped_meshes: true,
    });

    info!("camera_spawn: camera + lighting ready");
}

/// Displays camera debug info in an egui overlay.
fn camera_debug_overlay(
    mut contexts: EguiContexts,
    camera: Query<&CinematicCamera>,
    cameras: Query<&Transform, With<Camera3d>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let Ok(cam) = camera.single() else {
        return;
    };
    let Ok(cam_transform) = cameras.single() else {
        return;
    };

    let pos = cam_transform.translation;
    let effective_radius = match cam.mode {
        super::components::CameraMode::Orbital | super::components::CameraMode::Cinematic => {
            200.0 * cam.zoom
        }
        super::components::CameraMode::ThirdPerson => cam.third_person.distance * cam.zoom,
        super::components::CameraMode::FirstPerson => 0.0,
    };

    egui::Window::new("Camera Debug")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::RIGHT_TOP, [-10.0, 10.0])
        .show(ctx, |ui| {
            ui.label(format!(
                "Position: {:.1}, {:.1}, {:.1}",
                pos.x, pos.y, pos.z
            ));
            ui.label(format!(
                "Look At: {:.1}, {:.1}, {:.1}",
                cam.focus_target.unwrap_or(Vec3::new(0.0, 10.0, 0.0)).x,
                cam.focus_target.unwrap_or(Vec3::new(0.0, 10.0, 0.0)).y,
                cam.focus_target.unwrap_or(Vec3::new(0.0, 10.0, 0.0)).z
            ));
            ui.label(format!("Yaw: {:.1}°", cam.yaw_deg));
            ui.label(format!("Pitch: {:.1}°", cam.pitch_deg));
            ui.label(format!("Zoom: {:.2}x", cam.zoom));
            ui.label(format!("Radius: {:.1}m", effective_radius));
            ui.label(format!("Mode: {:?} (Press TAB to toggle)", cam.mode));
        });
}
