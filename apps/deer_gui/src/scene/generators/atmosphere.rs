//! Atmosphere generator — adds lighting and fog configuration to a scene.
//!
//! Spawns directional light (sun) and configures ambient/fog settings.
//! Note: DistanceFog is a camera component applied by the camera plugin.

use bevy::ecs::system::Commands;
use bevy::log::{debug, info, warn};
use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::prelude::*;

use crate::render::lighting::CinematicLighting;
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

    // Disable auto-sun-position to prevent RenderQualityPlugin from hijacking sun position
    commands.insert_resource(CinematicLighting {
        auto_sun_position: false,
        ..default()
    });

    // Load skybox if specified
    if let Some(path) = skybox_path {
        let path_clone = path.clone();
        commands.queue(move |world: &mut World| {
            let (mesh_handle, material_handle) = {
                let asset_server = world.resource::<AssetServer>();
                let image_handle = asset_server.load(path_clone);

                let mut meshes = world.resource_mut::<Assets<Mesh>>();
                let mut mesh = Sphere::new(4000.0).mesh().build();

                // Invert normals to make them face inward
                if let Some(VertexAttributeValues::Float32x3(normals)) =
                    mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL)
                {
                    for n in normals.iter_mut() {
                        n[0] = -n[0];
                        n[1] = -n[1];
                        n[2] = -n[2];
                    }
                }

                // Reverse triangle winding order
                if let Some(Indices::U32(indices)) = mesh.indices_mut() {
                    for i in (0..indices.len()).step_by(3) {
                        indices.swap(i + 1, i + 2);
                    }
                }
                let mesh_handle = meshes.add(mesh);

                let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
                let material_handle = materials.add(StandardMaterial {
                    base_color_texture: Some(image_handle),
                    unlit: true,
                    cull_mode: None,
                    fog_enabled: false,
                    ..default()
                });

                (mesh_handle, material_handle)
            };

            world
                .spawn((
                    AtmosphereMarker,
                    Name::new("SkyboxSphere"),
                    Mesh3d(mesh_handle),
                    MeshMaterial3d(material_handle),
                    Transform::default(),
                    Visibility::default(),
                    InheritedVisibility::default(),
                ))
                .set_parent_in_place(root);
        });
    }

    debug!("gen_atmosphere: complete");
}
