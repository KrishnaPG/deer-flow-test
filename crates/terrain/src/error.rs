//! Error types for the terrain system.

use thiserror::Error;

/// Errors that can occur in the terrain system.
#[derive(Error, Debug, Clone)]
pub enum TerrainError {
    /// Failed to load heightmap image.
    #[error("Failed to load heightmap: {0}")]
    HeightmapLoadError(String),

    /// Invalid heightmap dimensions.
    #[error("Invalid heightmap dimensions: {width}x{height}, expected power of two plus one")]
    InvalidHeightmapSize { width: u32, height: u32 },

    /// Heightmap format not supported.
    #[error("Unsupported heightmap format: {0}")]
    UnsupportedFormat(String),

    /// Terrain chunk not found.
    #[error("Terrain chunk not found: ({x}, {z}) at LOD {lod}")]
    ChunkNotFound { x: i32, z: i32, lod: u32 },

    /// Material shader not found.
    #[error("Terrain material shader not found: {0}")]
    ShaderNotFound(String),

    /// Texture loading failed.
    #[error("Failed to load terrain texture: {0}")]
    TextureLoadError(String),

    /// LOD level out of range.
    #[error("LOD level {level} out of range (max: {max})")]
    InvalidLodLevel { level: u32, max: u32 },
}

/// Result type for terrain operations.
pub type Result<T> = std::result::Result<T, TerrainError>;
