//! Atmosphere generator — adds lighting and fog configuration to a scene.
//!
//! Spawns directional light (sun) and configures ambient/fog settings.
//! Note: DistanceFog is a camera component applied by the camera plugin.

use bevy::ecs::system::Commands;
use bevy::log::{debug, info, warn};
use bevy::prelude::*;

use crate::scene::descriptor::GeneratorParams;

/// Marker component for atmosphere entities.
#[derive(Component, Debug)]
pub struct AtmosphereMarker;

/// Scene-wide fog configuration (read by camera plugin to apply DistanceFog).
#[derive(Resource, Debug, Clone)]
pub struct SceneFogConfig {
    pub color: Color,
    pub density: f32,
    pub enabled: bool,
}

impl Default for SceneFogConfig {
    fn default() -> Self {
        Self {
            color: Color::srgba(0.7, 0.75, 0.8, 1.0),
            density: 0.005,
            enabled: true,
        }
    }
}

/// Generate atmospheric effects from scene descriptor parameters.
pub fn gen_atmosphere(
    commands: &mut Commands,
    _meshes: &mut Assets<Mesh>,
    _materials: &mut Assets<bevy::pbr::StandardMaterial>,
    root: Entity,
    params: &GeneratorParams,
) {
    let GeneratorParams::Atmosphere {
        fog_color,
        fog_density,
        sun_direction,
        sun_color,
        sun_intensity,
        skybox_path,
    } = params
    else {
        warn!("gen_atmosphere: expected Atmosphere params");
        return;
    };

    info!("gen_atmosphere: configuring lighting and fog");

    // Spawn directional light (sun)
    let sun_entity = commands
        .spawn((
            AtmosphereMarker,
            Name::new("Sun"),
            DirectionalLight {
                color: Color::srgb(sun_color[0], sun_color[1], sun_color[2]),
                illuminance: *sun_intensity,
                shadows_enabled: true,
                ..default()
            },
            Transform::from_xyz(sun_direction[0], sun_direction[1], sun_direction[2])
                .looking_at(Vec3::ZERO, Vec3::Y),
            Visibility::default(),
        ))
        .id();

    commands.entity(root).add_child(sun_entity);

    // Set global ambient light
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.8, 0.9, 1.0),
        brightness: 500.0,
        affects_lightmapped_meshes: true,
    });

    // Insert fog config resource for camera plugin to pick up
    commands.insert_resource(SceneFogConfig {
        color: Color::srgba(fog_color[0], fog_color[1], fog_color[2], 1.0),
        density: *fog_density,
        enabled: true,
    });

    // Load skybox if specified
    if let Some(path) = skybox_path {
        debug!("gen_atmosphere: loading skybox from {path}");
        // Skybox will be added to camera via asset loading
        // For now, just log it
    }

    debug!("gen_atmosphere: complete");
}
