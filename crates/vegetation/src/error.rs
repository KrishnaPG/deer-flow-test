//! Error types for the vegetation system.

use thiserror::Error;

/// Errors that can occur in the vegetation system.
#[derive(Error, Debug, Clone)]
pub enum VegetationError {
    /// Invalid density value.
    #[error("Invalid density: {0}, must be between 0.0 and 1.0")]
    InvalidDensity(f32),

    /// Invalid placement area.
    #[error("Invalid placement area: width={width}, height={height}")]
    InvalidArea { width: f32, height: f32 },

    /// No valid placement positions found.
    #[error("No valid placement positions found for vegetation")]
    NoValidPositions,

    /// Biome filter error.
    #[error("Biome filter error: {0}")]
    BiomeFilterError(String),
}

/// Result type for vegetation operations.
pub type Result<T> = std::result::Result<T, VegetationError>;
