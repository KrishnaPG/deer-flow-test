//! Error types for the water system.

use thiserror::Error;

/// Errors that can occur in the water system.
#[derive(Error, Debug, Clone)]
pub enum WaterError {
    /// Invalid water level.
    #[error("Invalid water level: {0}")]
    InvalidLevel(f32),

    /// Invalid dimensions.
    #[error("Invalid water dimensions: {width} x {height}")]
    InvalidDimensions { width: f32, height: f32 },

    /// Invalid wave parameters.
    #[error("Invalid wave parameters: {0}")]
    InvalidWaveParams(String),
}

/// Result type for water operations.
pub type Result<T> = std::result::Result<T, WaterError>;
