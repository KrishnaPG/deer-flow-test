//! Generator registry — maps type names to factory functions.

use std::collections::HashMap;

use bevy::asset::Assets;
use bevy::ecs::system::Commands;
use bevy::log::{debug, info, trace};
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Entity, Mesh, Resource};

use crate::scene::descriptor::GeneratorParams;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Signature for a procedural generator factory function.
pub type GeneratorFn = fn(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    root: Entity,
    params: &GeneratorParams,
);

// ---------------------------------------------------------------------------
// GeneratorRegistry
// ---------------------------------------------------------------------------

/// Resource mapping generator type names to factory functions.
#[derive(Resource)]
pub struct GeneratorRegistry {
    factories: HashMap<String, GeneratorFn>,
}

impl GeneratorRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        trace!("GeneratorRegistry::new — empty");
        Self {
            factories: HashMap::new(),
        }
    }

    /// Register a generator factory by name.
    pub fn register(&mut self, name: &str, factory: GeneratorFn) {
        info!("GeneratorRegistry::register — '{name}'");
        self.factories.insert(name.to_string(), factory);
    }

    /// Look up a generator by name.
    pub fn get(&self, name: &str) -> Option<&GeneratorFn> {
        let result = self.factories.get(name);
        trace!(
            "GeneratorRegistry::get — '{name}' found={}",
            result.is_some()
        );
        result
    }

    /// List all registered generator names.
    pub fn available(&self) -> Vec<&str> {
        self.factories.keys().map(|s| s.as_str()).collect()
    }

    /// Create a registry pre-loaded with all built-in generators.
    pub fn with_builtins() -> Self {
        let mut registry = Self::new();
        registry.register("starfield", super::starfield::gen_starfield);
        registry.register("spiral_trails", super::spiral_trails::gen_spiral_trails);
        registry.register("river_barges", super::river_barges::gen_river_barges);
        registry.register(
            "path_travellers",
            super::path_travellers::gen_path_travellers,
        );
        registry.register("cloud_layer", super::cloud_layer::gen_cloud_layer);
        registry.register("drop_pods", super::drop_pods::gen_drop_pods);
        registry.register(
            "static_glow_cluster",
            super::static_glow::gen_static_glow_cluster,
        );
        registry.register("gltf_subscene", super::gltf_subscene::gen_gltf_subscene);
        // Medieval terrain for open world scenes
        registry.register("medieval_terrain", super::terrain::gen_medieval_terrain);
        // Vegetation for biome-based foliage scattering
        registry.register("vegetation", super::vegetation::gen_vegetation);
        debug!(
            "GeneratorRegistry::with_builtins — {} generators",
            registry.factories.len(),
        );
        registry
    }
}

impl Default for GeneratorRegistry {
    fn default() -> Self {
        Self::with_builtins()
    }
}
