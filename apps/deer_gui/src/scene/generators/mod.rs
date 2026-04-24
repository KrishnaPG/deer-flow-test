//! Procedural generator factories for data-driven scene construction.

pub mod atmosphere;
pub mod cloud_layer;
pub mod drop_pods;
pub mod foliage;
pub mod gltf_subscene;
pub mod monolith;
pub mod npcs;
pub mod path_travellers;
pub mod registry;
pub mod river_barges;
pub mod rocks;
pub mod spiral_trails;
pub mod starfield;
pub mod static_glow;
pub mod systems;
pub mod terrain;
pub mod vegetation;
pub mod vegetation_spawner;
pub mod water;

pub use cloud_layer::CloudParticle;
pub use drop_pods::DropPod;
pub use monolith::Monolith;
pub use path_travellers::Traveller;
pub use registry::GeneratorRegistry;
pub use river_barges::Barge;
pub use spiral_trails::SpiralTrail;
pub use static_glow::GlowEntity;
pub use systems::{barge_system, cloud_system, drop_pod_system, traveller_system};
pub use terrain::{MedievalTerrain, TerrainLayerConfig};
pub use vegetation::{VegetationGenerator, VegetationInstance, VegetationKind};
