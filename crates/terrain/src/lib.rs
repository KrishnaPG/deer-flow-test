//! # deer-terrain
//!
//! Core terrain algorithms for heightmap processing and mesh data generation.
//! This crate provides the data structures and algorithms; the app layer
//! handles Bevy ECS integration and rendering.
//!
//! ## Features
//!
//! - Load and process heightmaps from image data
//! - Generate terrain mesh data (positions, normals, UVs, indices)
//! - LOD level calculations
//! - Splat mask generation for texture blending
//! - Modular and reusable across different projects

pub mod error;
pub mod heightmap;
pub mod lod;
pub mod mesh_gen;
pub mod splat;

pub use error::TerrainError;
pub use heightmap::{Heightmap, HeightmapConfig};
pub use lod::{calculate_lod, LodConfig, TerrainLod};
pub use mesh_gen::{generate_terrain_mesh, MeshGenConfig, TerrainMeshData};
pub use splat::{
    generate_splat_from_height, generate_splat_from_height_and_slope, SplatConfig,
    SplatGenerationMode, SplatMask, MAX_TERRAIN_LAYERS,
};

/// Result type for terrain operations.
pub type Result<T> = std::result::Result<T, TerrainError>;
