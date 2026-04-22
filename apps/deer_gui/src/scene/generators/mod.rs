//! Procedural generator factories for data-driven scene construction.

pub mod cloud_layer;
pub mod drop_pods;
pub mod gltf_subscene;
pub mod path_travellers;
pub mod registry;
pub mod river_barges;
pub mod spiral_trails;
pub mod starfield;
pub mod static_glow;
pub mod systems;
pub mod terrain;
pub mod vegetation;

pub use cloud_layer::CloudParticle;
pub use drop_pods::DropPod;
pub use path_travellers::Traveller;
pub use registry::GeneratorRegistry;
pub use river_barges::Barge;
pub use spiral_trails::SpiralTrail;
pub use static_glow::GlowEntity;
pub use systems::{barge_system, cloud_system, drop_pod_system, traveller_system};
pub use terrain::{MedievalTerrain, TerrainLayerConfig};
pub use vegetation::{spawn_vegetation, VegetationGenerator};
