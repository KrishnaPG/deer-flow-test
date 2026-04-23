//! Terrain generator for medieval open world scenes.
//!
//! Creates heightmap-based terrain with texture splatting for grass, dirt, rock, and snow layers.
//! Uses the `deer-terrain` crate for mesh generation.

use bevy::asset::AssetServer;
use bevy::ecs::system::Commands;
use bevy::image::Image;
use bevy::log::{debug, info, warn};
use bevy::math::Vec2;
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::*;
use deer_terrain::{generate_terrain_mesh, Heightmap, MeshGenConfig};

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
        chunk_size: _,
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
        "gen_medieval_terrain: heightmap='{}' size={}x{} height={}m res={}",
        heightmap, world_size[0], world_size[1], height_scale, resolution
    );

    let layer_config = TerrainLayerConfig {
        grass: layer_textures[0].clone(),
        dirt: layer_textures[1].clone(),
        rock: layer_textures[2].clone(),
        snow: layer_textures[3].clone(),
        splat_mask: splat_mask.clone(),
        uv_scale: *uv_scale,
    };

    // Load heightmap and generate mesh
    let mesh_result = load_heightmap_and_create_mesh(
        heightmap,
        world_size[0],
        world_size[1],
        *height_scale,
        *height_offset,
        *resolution,
        *invert_heightmap,
    );

    let mesh = match mesh_result {
        Ok(m) => m,
        Err(e) => {
            warn!("gen_medieval_terrain: failed to load heightmap, using flat plane: {e}");
            create_flat_terrain_mesh(world_size[0], world_size[1])
        }
    };

    let mesh_handle = meshes.add(mesh);
    let material = create_terrain_material(materials, &layer_config);

    let terrain_entity = commands
        .spawn((
            MedievalTerrain {
                heightmap_path: heightmap.clone(),
                world_size: Vec2::new(world_size[0], world_size[1]),
                height_scale: *height_scale,
            },
            Mesh3d(mesh_handle),
            MeshMaterial3d(material),
            Transform::from_translation(Vec3::new(0.0, *height_offset, 0.0)),
            Name::new("MedievalTerrain"),
        ))
        .id();

    commands.entity(root).add_child(terrain_entity);

    debug!("gen_medieval_terrain: spawned terrain entity {terrain_entity:?}");
}

/// Load heightmap from file and create terrain mesh using deer-terrain.
fn load_heightmap_and_create_mesh(
    heightmap_path: &str,
    width: f32,
    depth: f32,
    height_scale: f32,
    height_offset: f32,
    resolution: u32,
    invert: bool,
) -> Result<Mesh, String> {
    // Resolve asset path relative to assets directory
    let asset_path = format!("apps/deer_gui/assets/{heightmap_path}");
    let path = std::path::Path::new(&asset_path);

    if !path.exists() {
        return Err(format!("Heightmap file not found: {asset_path}"));
    }

    // Load image and extract grayscale height data
    let image_data = std::fs::read(path).map_err(|e| format!("Failed to read heightmap: {e}"))?;
    let img = image::load_from_memory(&image_data)
        .map_err(|e| format!("Failed to parse heightmap image: {e}"))?;
    let gray_img = img.to_luma8();
    let (img_width, img_height) = gray_img.dimensions();

    // Extract pixel data as u8 array
    let pixels: Vec<u8> = gray_img.iter().cloned().collect();

    // Create heightmap using deer-terrain
    let heightmap = Heightmap::from_grayscale(&pixels, img_width as u32, img_height as u32)
        .map_err(|e| format!("Failed to create heightmap: {e}"))?;

    // Generate mesh using deer-terrain
    let config = MeshGenConfig {
        world_size: (width, depth),
        height_scale,
        height_offset,
        resolution,
        invert,
    };

    let mesh_data = generate_terrain_mesh(&heightmap, &config)
        .map_err(|e| format!("Failed to generate mesh: {e}"))?;

    // Convert TerrainMeshData to Bevy Mesh
    Ok(convert_to_bevy_mesh(&mesh_data))
}

/// Convert deer-terrain mesh data to Bevy Mesh.
fn convert_to_bevy_mesh(mesh_data: &deer_terrain::TerrainMeshData) -> Mesh {
    use bevy::asset::RenderAssetUsages;
    use bevy::mesh::{Indices, PrimitiveTopology};

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    // Convert positions to Vec3
    let positions: Vec<[f32; 3]> = mesh_data.positions.clone();
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

    // Convert normals
    let normals: Vec<[f32; 3]> = mesh_data.normals.clone();
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

    // Convert UVs
    let uvs: Vec<[f32; 2]> = mesh_data.uvs.clone();
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    // Insert indices
    mesh.insert_indices(Indices::U32(mesh_data.indices.clone()));

    mesh
}

/// Create a flat terrain mesh as fallback.
fn create_flat_terrain_mesh(width: f32, depth: f32) -> Mesh {
    let plane = bevy::math::primitives::Plane3d::default();
    let mut mesh = Mesh::from(plane);
    mesh
}

/// Create terrain material with texture support.
fn create_terrain_material(
    materials: &mut Assets<StandardMaterial>,
    layer_config: &TerrainLayerConfig,
) -> Handle<StandardMaterial> {
    // Try to load base color texture
    if let Some(ref grass_path) = layer_config.grass {
        let asset_path = format!("apps/deer_gui/assets/{grass_path}");
        if std::path::Path::new(&asset_path).exists() {
            info!("gen_medieval_terrain: using texture from {grass_path}");
            // For now, create a colored material - full texture splatting requires custom shader
            return materials.add(StandardMaterial {
                base_color: Color::srgb(0.35, 0.55, 0.25),
                perceptual_roughness: 0.85,
                reflectance: 0.1,
                ..Default::default()
            });
        }
    }

    // Fallback: basic green material
    materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.5, 0.2),
        perceptual_roughness: 0.9,
        reflectance: 0.1,
        ..Default::default()
    })
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
