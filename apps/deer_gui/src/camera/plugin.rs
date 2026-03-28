//! [`CameraPlugin`] — registers startup and per-frame camera systems.

use bevy::light::GlobalAmbientLight;
use bevy::log::{debug, info};
use bevy::math::Vec3;
use bevy::prelude::{
    App, Camera3d, Color, Commands, DirectionalLight, IntoScheduleConfigs, Plugin, Startup,
    Transform, Update,
};

use super::components::CinematicCamera;
use super::systems::{
    camera_focus_system, camera_input_system, camera_interpolation_system, camera_shake_system,
};
use crate::constants::camera::ORBIT_RADIUS;

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

        app.add_systems(Startup, camera_spawn_system).add_systems(
            Update,
            (
                camera_input_system,
                camera_interpolation_system,
                camera_shake_system,
                camera_focus_system,
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
fn camera_spawn_system(mut commands: Commands) {
    let cam = CinematicCamera::default();

    let position = cam.compute_position(ORBIT_RADIUS);
    let look_at = cam.compute_look_at();

    debug!("camera_spawn: pos={:?} look_at={:?}", position, look_at,);

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
