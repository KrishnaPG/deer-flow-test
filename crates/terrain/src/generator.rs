//! Terrain generator for Bevy scene system.
//!
//! Provides Bevy systems and components for terrain generation
//! from heightmap assets.

use bevy::asset::Assets;
use bevy::ecs::system::Commands;
use bevy::log::{debug, info};
use bevy::math::Vec3;
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

use crate::lod::{LodConfig, TerrainChunkLod};
use crate::Terrain;

/// Marker component for terrain generator entities.
#[derive(Component, Debug)]
pub struct TerrainGenerator {
    /// Configuration for terrain generation.
    pub config: TerrainConfig,
}

/// Configuration for terrain generation.
#[derive(Debug, Clone)]
pub struct TerrainConfig {
    /// Path to the heightmap asset.
    pub heightmap_path: String,
    /// World-space size of the terrain (x, z).
    pub world_size: (f32, f32),
    /// Maximum height in meters.
    pub height_scale: f32,
    /// Height offset (base terrain level).
    pub height_offset: f32,
    /// Mesh resolution per chunk.
    pub resolution: u32,
    /// Chunk size in world units.
    pub chunk_size: f32,
    /// Number of chunks per side (total chunks = chunks_per_side^2).
    pub chunks_per_side: u32,
    /// Whether to invert heightmap.
    pub invert_heightmap: bool,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            heightmap_path: String::new(),
            world_size: (1000.0, 1000.0),
            height_scale: 100.0,
            height_offset: 0.0,
            resolution: 64,
            chunk_size: 100.0,
            chunks_per_side: 10,
            invert_heightmap: false,
        }
    }
}

/// A terrain chunk entity.
#[derive(Component, Debug)]
pub struct TerrainChunk {
    /// Grid coordinates of this chunk.
    pub grid_pos: (i32, i32),
    /// World-space bounds (min, max).
    pub bounds: (Vec3, Vec3),
}

/// Setup terrain assets and resources.
///
/// Call this once during startup to initialize the terrain system.
pub fn setup_terrain_assets(mut commands: Commands) {
    // Initialize LOD config if not present
    if !commands.world().contains_resource::<LodConfig>() {
        commands.insert_resource(LodConfig::default());
    }

    info!("terrain: initialized terrain system resources");
}

/// System to generate terrain chunks from TerrainGenerator components.
pub fn terrain_chunk_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    generators: Query<(Entity, &TerrainGenerator), Added<TerrainGenerator>>,
) {
    for (entity, generator) in generators.iter() {
        debug!(
            "terrain_chunk_system: generating terrain for entity {:?}",
            entity
        );

        let config = &generator.config;

        // Generate chunks
        for z in 0..config.chunks_per_side {
            for x in 0..config.chunks_per_side {
                spawn_terrain_chunk(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    entity,
                    (x as i32, z as i32),
                    config,
                );
            }
        }

        // Add LOD config if not present
        if !commands.world().contains_resource::<LodConfig>() {
            commands.insert_resource(LodConfig::default());
        }
    }
}

/// Spawn a single terrain chunk.
fn spawn_terrain_chunk(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    parent: Entity,
    grid_pos: (i32, i32),
    config: &TerrainConfig,
) {
    let chunk_size = config.chunk_size;

    // Calculate chunk bounds
    let min_x = grid_pos.0 as f32 * chunk_size - (config.world_size.0 / 2.0);
    let max_x = min_x + chunk_size;
    let min_z = grid_pos.1 as f32 * chunk_size - (config.world_size.1 / 2.0);
    let max_z = min_z + chunk_size;

    let bounds = (
        Vec3::new(min_x, -config.height_scale, min_z),
        Vec3::new(max_x, config.height_scale, max_z),
    );

    // Create a simple plane mesh for the chunk
    let mesh = create_chunk_mesh(chunk_size, config.resolution);
    let mesh_handle = meshes.add(mesh);

    // Create material
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.5, 0.2),
        perceptual_roughness: 0.9,
        ..Default::default()
    });

    // Spawn chunk entity
    let center = Vec3::new(min_x + chunk_size / 2.0, 0.0, min_z + chunk_size / 2.0);

    commands.spawn((
        Terrain,
        TerrainChunk { grid_pos, bounds },
        TerrainChunkLod::new(grid_pos, chunk_size),
        Mesh3d(mesh_handle),
        MeshMaterial3d(material),
        Transform::from_translation(center),
        ChildOf(parent),
    ));

    debug!(
        "spawn_terrain_chunk: chunk ({}, {}) at {:?}",
        grid_pos.0, grid_pos.1, center
    );
}

/// Create a simple terrain chunk mesh.
fn create_chunk_mesh(size: f32, resolution: u32) -> Mesh {
    let resolution = resolution.max(2);
    let half_size = size / 2.0;

    let vertex_count = ((resolution + 1) * (resolution + 1)) as usize;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(vertex_count);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(vertex_count);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(vertex_count);
    let mut indices: Vec<u32> = Vec::with_capacity((resolution * resolution * 6) as usize);

    // Generate vertices
    for z in 0..=resolution {
        for x in 0..=resolution {
            let u = x as f32 / resolution as f32;
            let v = z as f32 / resolution as f32;

            let px = u * size - half_size;
            let pz = v * size - half_size;

            positions.push([px, 0.0, pz]);
            normals.push([0.0, 1.0, 0.0]);
            uvs.push([u, v]);
        }
    }

    // Generate indices
    for z in 0..resolution {
        for x in 0..resolution {
            let i0 = (z * (resolution + 1) + x) as u32;
            let i1 = i0 + 1;
            let i2 = i0 + (resolution + 1) as u32;
            let i3 = i2 + 1;

            indices.push(i0);
            indices.push(i2);
            indices.push(i1);

            indices.push(i1);
            indices.push(i2);
            indices.push(i3);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

/// Descriptor for terrain generator (for scene RON files).
#[derive(Debug, Clone, serde::Deserialize)]
pub struct TerrainGeneratorDescriptor {
    /// Heightmap asset path.
    pub heightmap: String,
    /// World size (x, z).
    pub world_size: (f32, f32),
    /// Height scale.
    #[serde(default)]
    pub height_scale: f32,
    /// Height offset.
    #[serde(default)]
    pub height_offset: f32,
    /// Chunk size.
    #[serde(default = "default_chunk_size")]
    pub chunk_size: f32,
    /// Resolution per chunk.
    #[serde(default = "default_resolution")]
    pub resolution: u32,
    /// Invert heightmap.
    #[serde(default)]
    pub invert: bool,
}

fn default_chunk_size() -> f32 {
    100.0
}

fn default_resolution() -> u32 {
    64
}

impl From<TerrainGeneratorDescriptor> for TerrainConfig {
    fn from(desc: TerrainGeneratorDescriptor) -> Self {
        let chunks_per_side = ((desc.world_size.0 / desc.chunk_size).ceil() as u32).max(1);

        Self {
            heightmap_path: desc.heightmap,
            world_size: desc.world_size,
            height_scale: desc.height_scale,
            height_offset: desc.height_offset,
            resolution: desc.resolution,
            chunk_size: desc.chunk_size,
            chunks_per_side,
            invert_heightmap: desc.invert,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_config_default() {
        let config = TerrainConfig::default();
        assert_eq!(config.world_size, (1000.0, 1000.0));
        assert_eq!(config.height_scale, 100.0);
        assert_eq!(config.resolution, 64);
        assert_eq!(config.chunks_per_side, 10);
    }

    #[test]
    fn terrain_chunk_mesh_generation() {
        let mesh = create_chunk_mesh(100.0, 32);
        // (32+1) * (32+1) = 1089 vertices
        let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
        assert_eq!(positions.len(), 1089);
    }

    #[test]
    fn descriptor_to_config() {
        let desc = TerrainGeneratorDescriptor {
            heightmap: "test.png".to_string(),
            world_size: (500.0, 500.0),
            height_scale: 50.0,
            height_offset: 10.0,
            chunk_size: 50.0,
            resolution: 32,
            invert: true,
        };

        let config: TerrainConfig = desc.into();
        assert_eq!(config.chunks_per_side, 10);
        assert!(config.invert_heightmap);
    }
}
