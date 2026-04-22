//! # deer-water
//!
//! Water plane generation with wave animation for 3D scenes.
//! Provides modular water rendering with configurable waves, flow, and appearance.
//!
//! ## Features
//!
//! - Flat water planes with configurable dimensions
//! - Wave animation (sine-based, Gerstner waves)
//! - Flow direction for rivers
//! - Water plane mesh generation

pub mod plane;
pub mod waves;
pub mod error;

pub use plane::{WaterPlane, WaterConfig, WaterMeshData};
pub use waves::{WaveConfig, WaveType, WaveParams};
pub use error::WaterError;

/// Result type for water operations.
pub type Result<T> = std::result::Result<T, WaterError>;
