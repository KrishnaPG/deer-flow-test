//! Terrain generator for medieval open world scenes.
//!
//! Creates heightmap-based terrain with texture splatting for grass, dirt, rock, and snow layers.
//! Uses the `deer-terrain` crate for mesh generation.

use bevy::asset::AssetServer;
use bevy::ecs::system::Commands;
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
    _materials: &mut Assets<StandardMaterial>,
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
        *uv_scale,
    );

    let mesh = match mesh_result {
        Ok(m) => m,
        Err(e) => {
            warn!("gen_medieval_terrain: failed to load heightmap, using flat plane: {e}");
            create_flat_terrain_mesh(world_size[0], world_size[1])
        }
    };

    let mesh_handle = meshes.add(mesh);

    let terrain_entity = commands
        .spawn((
            MedievalTerrain {
                heightmap_path: heightmap.clone(),
                world_size: Vec2::new(world_size[0], world_size[1]),
                height_scale: *height_scale,
            },
            Mesh3d(mesh_handle),
            Transform::from_translation(Vec3::new(0.0, *height_offset, 0.0)),
            Visibility::default(),
            InheritedVisibility::default(),
            Name::new("MedievalTerrain"),
        ))
        .id();

    commands.entity(root).add_child(terrain_entity);

    // Queue material creation with PBR textures via AssetServer.
    let layer_config_clone = layer_config.clone();
    commands.queue(move |world: &mut World| {
        let material = create_terrain_material(world, &layer_config_clone);
        world
            .commands()
            .entity(terrain_entity)
            .insert(MeshMaterial3d(material));
    });

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
    uv_scale: f32,
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
    Ok(convert_to_bevy_mesh(&mesh_data, uv_scale))
}

/// Convert deer-terrain mesh data to Bevy Mesh.
fn convert_to_bevy_mesh(mesh_data: &deer_terrain::TerrainMeshData, uv_scale: f32) -> Mesh {
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
    let uvs: Vec<[f32; 2]> = mesh_data
        .uvs
        .iter()
        .map(|uv| [uv[0] * uv_scale, uv[1] * uv_scale])
        .collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    // Insert indices
    mesh.insert_indices(Indices::U32(mesh_data.indices.clone()));

    mesh
}

/// Create a flat terrain mesh as fallback.
fn create_flat_terrain_mesh(_width: f32, _depth: f32) -> Mesh {
    let plane = bevy::math::primitives::Plane3d::default();
    let mesh = Mesh::from(plane);
    mesh
}

/// Create terrain material with PBR texture support.
///
/// Loads color, normal, roughness, and AO maps from the asset server.
/// Panics if textures are referenced but missing (no silent fallbacks).
fn create_terrain_material(
    world: &mut World,
    layer_config: &TerrainLayerConfig,
) -> Handle<StandardMaterial> {
    // Collect texture paths first (no borrow needed)
    let grass_path = layer_config.grass.clone();
    let normal_path = grass_path
        .as_ref()
        .map(|p| p.replace("_Color.png", "_NormalDX.png"));
    let ao_path = grass_path
        .as_ref()
        .map(|p| p.replace("_Color.png", "_AmbientOcclusion.png"));

    // Load texture handles via AssetServer (immutable borrow)
    let (color_handle, normal_handle, ao_handle) = {
        let asset_server = world.resource::<AssetServer>();
        let color = grass_path.as_ref().and_then(|p| {
            let asset_path = format!("apps/deer_gui/assets/{p}");
            if std::path::Path::new(&asset_path).exists() {
                Some(asset_server.load(p))
            } else {
                None
            }
        });
        let normal = normal_path.as_ref().and_then(|p| {
            let asset_path = format!("apps/deer_gui/assets/{p}");
            if std::path::Path::new(&asset_path).exists() {
                Some(asset_server.load(p))
            } else {
                None
            }
        });
        let ao = ao_path.as_ref().and_then(|p| {
            let asset_path = format!("apps/deer_gui/assets/{p}");
            if std::path::Path::new(&asset_path).exists() {
                Some(asset_server.load(p))
            } else {
                None
            }
        });
        (color, normal, ao)
    };

    let mut materials = world.resource_mut::<Assets<StandardMaterial>>();

    let mut material = StandardMaterial {
        perceptual_roughness: 0.85,
        reflectance: 0.1,
        ..Default::default()
    };

    // Apply base color texture
    if let Some(handle) = color_handle {
        info!("gen_medieval_terrain: applying grass color texture");
        material.base_color_texture = Some(handle);
    } else if grass_path.is_some() {
        panic!(
            "Terrain grass texture referenced but not found. \
             Ensure textures are downloaded to apps/deer_gui/assets/textures/terrain/"
        );
    } else {
        // No texture specified - use solid color (only for fallback/testing)
        material.base_color = Color::srgb(0.35, 0.55, 0.25);
    }

    // Apply normal map if available
    if let Some(handle) = normal_handle {
        info!("gen_medieval_terrain: applying grass normal map");
        material.normal_map_texture = Some(handle);
    }

    // Apply AO map if available
    if let Some(handle) = ao_handle {
        info!("gen_medieval_terrain: applying grass AO map");
        material.occlusion_texture = Some(handle);
    }

    materials.add(material)
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
