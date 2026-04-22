//! [`CameraPlugin`] — registers startup and per-frame camera systems.

use bevy::light::GlobalAmbientLight;
use bevy::log::{debug, info};
use bevy::math::Vec3;
use bevy::prelude::{
    App, Camera3d, Color, Commands, DirectionalLight, IntoScheduleConfigs, Plugin, Res, Startup,
    Transform, Update,
};

use super::components::CinematicCamera;
use super::navigation::{camera_sync_snapshot_system, CameraSyncState};
use super::systems::{
    camera_focus_system, camera_input_system, camera_interpolation_system,
    camera_mode_toggle_system, camera_preferences_sync_system, camera_shake_system,
    first_person_movement_system, mouse_look_system, viewport_navigation_system,
};
use crate::constants::camera::ORBIT_RADIUS;
use crate::preferences::UserPreferences;

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
            );
    }
}

// ---------------------------------------------------------------------------
// Spawn system
// ---------------------------------------------------------------------------

/// Spawns the primary camera entity with [`CinematicCamera`] state,
/// a directional light for basic illumination, and an ambient light resource.
fn camera_spawn_system(mut commands: Commands, prefs: Option<Res<UserPreferences>>) {
    let mut cam = CinematicCamera::default();
    // Apply saved camera mode from user preferences if available.
    if let Some(prefs) = prefs {
        cam.mode = prefs.camera_mode;
        cam.target_mode = prefs.camera_mode;
    }
    cam.mode_transition = 1.0; // No transition.

    let position = cam.compute_position(ORBIT_RADIUS);
    let look_at = cam.compute_look_at();

    debug!(
        "camera_spawn: pos={:?} look_at={:?} mode={:?}",
        position, look_at, cam.mode
    );

    // Camera entity.
    commands.spawn((
        Camera3d::default(),
        cam,
        Transform::from_translation(position).looking_at(look_at, Vec3::Y),
    ));

    // Directional light for scene illumination.
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_translation(Vec3::new(50.0, 200.0, 100.0)).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Ambient fill light.
    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
        affects_lightmapped_meshes: true,
    });

    info!("camera_spawn: camera + lighting ready");
}
