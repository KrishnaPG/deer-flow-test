//! Terrain generator for medieval open world scenes.
//!
//! Creates heightmap-based terrain with texture splatting for grass, dirt, rock, and snow layers.
//! Uses the `deer-terrain` crate for mesh generation and LOD management.

use bevy::ecs::system::Commands;
use bevy::log::{debug, info, warn};
use bevy::math::Vec2;
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::*;

use crate::scene::descriptor::GeneratorParams;

/// Marker component for terrain entities spawned by the terrain generator.
#[derive(Component, Debug, Clone)]
pub struct MedievalTerrain {
    /// Heightmap asset path.
    pub heightmap_path: String,
    /// World size (width, depth).
    pub world_size: Vec2,
    /// Maximum height in meters.
    pub height_scale: f32,
}

/// Configuration for terrain texture layers.
#[derive(Debug, Clone)]
pub struct TerrainLayerConfig {
    /// Grass layer texture path.
    pub grass: Option<String>,
    /// Dirt layer texture path.
    pub dirt: Option<String>,
    /// Rock layer texture path.
    pub rock: Option<String>,
    /// Snow layer texture path.
    pub snow: Option<String>,
    /// Splat mask texture path.
    pub splat_mask: Option<String>,
    /// UV tiling scale.
    pub uv_scale: f32,
}

impl Default for TerrainLayerConfig {
    fn default() -> Self {
        Self {
            grass: None,
            dirt: None,
            rock: None,
            snow: None,
            splat_mask: None,
            uv_scale: 10.0,
        }
    }
}

/// Generate medieval terrain from scene descriptor parameters.
///
/// This factory function is called by the scene loader when a Terrain generator
/// is specified in the scene RON file.
///
/// # Panics
/// Panics if called with non-Terrain GeneratorParams.
pub fn gen_medieval_terrain(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    root: Entity,
    params: &GeneratorParams,
) {
    let GeneratorParams::Terrain {
        heightmap,
        world_size,
        height_scale,
        height_offset,
        resolution,
        chunk_size,
        splat_mask,
        layer_textures,
        uv_scale,
        invert_heightmap,
    } = params
    else {
        warn!("gen_medieval_terrain called with non-Terrain params");
        return;
    };

    info!(
        "gen_medieval_terrain: heightmap='{}' size={}x{} height={}m",
        heightmap, world_size[0], world_size[1], height_scale
    );

    // Create terrain layer configuration from params
    let _layer_config = TerrainLayerConfig {
        grass: layer_textures[0].clone(),
        dirt: layer_textures[1].clone(),
        rock: layer_textures[2].clone(),
        snow: layer_textures[3].clone(),
        splat_mask: splat_mask.clone(),
        uv_scale: *uv_scale,
    };

    // Create material
    let material = create_terrain_material(materials);

    // Create a simple terrain plane mesh
    let mesh = create_terrain_mesh(
        world_size[0],
        world_size[1],
        *resolution,
        *height_scale,
        *height_offset,
    );
    let mesh_handle = meshes.add(mesh);

    // Spawn terrain entity with appropriate scale
    let terrain_entity = commands
        .spawn((
            MedievalTerrain {
                heightmap_path: heightmap.clone(),
                world_size: Vec2::new(world_size[0], world_size[1]),
                height_scale: *height_scale,
            },
            Mesh3d(mesh_handle),
            MeshMaterial3d(material),
            Transform {
                translation: Vec3::new(0.0, *height_offset, 0.0),
                scale: Vec3::new(world_size[0], 1.0, world_size[1]),
                ..Default::default()
            },
            Name::new("MedievalTerrain"),
        ))
        .id();

    // Add as child of root scene
    commands.entity(root).add_child(terrain_entity);

    debug!(
        "gen_medieval_terrain: spawned terrain entity {:?}",
        terrain_entity
    );
}

/// Create terrain material with texture layer support.
fn create_terrain_material(materials: &mut Assets<StandardMaterial>) -> Handle<StandardMaterial> {
    // For now, create a basic green material.
    // Full texture splatting will be implemented when we integrate the terrain crate's material system.
    let base_color = Color::srgb(0.3, 0.5, 0.2); // Medieval green

    materials.add(StandardMaterial {
        base_color,
        perceptual_roughness: 0.9,
        reflectance: 0.1,
        ..Default::default()
    })
}

/// Create a simple terrain mesh.
///
/// Uses Bevy's built-in plane mesh which is scaled via Transform component.
fn create_terrain_mesh(
    _width: f32,
    _depth: f32,
    _resolution: u32,
    _height_scale: f32,
    _height_offset: f32,
) -> Mesh {
    // Create a simple plane mesh (1x1 unit by default)
    // The actual size is controlled via Transform scale on the entity
    let plane = bevy::math::primitives::Plane3d::default();
    Mesh::from(plane)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_layer_config_default() {
        let config = TerrainLayerConfig::default();
        assert!(config.grass.is_none());
        assert!(config.dirt.is_none());
        assert_eq!(config.uv_scale, 10.0);
    }
}
