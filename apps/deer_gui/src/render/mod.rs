//! Render quality plugin — cinematic visuals for medieval RTS.
//!
//! This module provides comprehensive rendering configuration for
//! breathtaking visuals comparable to Horizon Zero Dawn and Crysis 2:
//!
//! - HDR rendering with ACES tonemapping
//! - Bloom post-processing for light glow
//! - Cascaded Shadow Maps (CSM) for terrain shadows
//! - Screen-Space Ambient Occlusion (SSAO)
//! - Volumetric fog and atmospheric scattering
//! - Screen-Space Reflections (SSR) for water
//! - High-quality PBR materials
//! - Realistic sky with procedural clouds
//! - Color grading and tone mapping
//! - Automatic quality detection based on hardware capabilities

pub mod atmosphere;
pub mod auto_detect;
pub mod capability_detect;
pub mod lighting;
pub mod plugin;
pub mod post_processing;
pub mod quality;

pub use atmosphere::AtmosphereConfig;
pub use auto_detect::{AutoQualityPlugin, GpuTier, HardwareProfile};
pub use capability_detect::{CapabilityQualityPlugin, HardwareCapabilities, PerformanceTier};
pub use lighting::CinematicLighting;
pub use plugin::RenderQualityPlugin;
pub use post_processing::PostProcessingConfig;
pub use quality::{QualityPreset, QualitySettings};
