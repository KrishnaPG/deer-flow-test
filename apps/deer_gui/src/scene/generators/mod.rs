//! Procedural generator factories for data-driven scene construction.

pub mod cloud_layer;
pub mod drop_pods;
pub mod path_travellers;
pub mod registry;
pub mod river_barges;
pub mod spiral_trails;
pub mod starfield;
pub mod static_glow;

pub use cloud_layer::CloudParticle;
pub use drop_pods::DropPod;
pub use path_travellers::Traveller;
pub use registry::GeneratorRegistry;
pub use river_barges::Barge;
pub use spiral_trails::SpiralTrail;
pub use static_glow::GlowEntity;
