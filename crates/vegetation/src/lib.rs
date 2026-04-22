//! # deer-vegetation
//!
//! Procedural vegetation placement and instancing system.
//! Provides algorithms for placing trees, bushes, and grass
//! based on density, biome filters, and randomization.
//!
//! ## Features
//!
//! - Procedural placement with density control
//! - Biome-based filtering
//! - Random offset and rotation
//! - GPU-ready instance data generation

pub mod placement;
pub mod biome;
pub mod error;

pub use placement::{VegetationPlacer, VegetationConfig, VegetationInstance, PlacementResult};
pub use biome::{BiomeFilter, BiomeType};
pub use error::VegetationError;

/// Result type for vegetation operations.
pub type Result<T> = std::result::Result<T, VegetationError>;
