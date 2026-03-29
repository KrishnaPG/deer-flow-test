//! Scene descriptor types — data-driven scene definitions.
//!
//! Deserialized from RON/JSON files at runtime. Each descriptor defines
//! a scene's name, audio, optional glTF base, and a list of procedural
//! generator invocations.

use bevy::log::trace;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// SceneDescriptor
// ---------------------------------------------------------------------------

/// Top-level scene descriptor, deserialized from a RON/JSON file.
#[derive(Debug, Clone, Deserialize)]
pub struct SceneDescriptor {
    /// Human-readable scene name (used as SceneManager key).
    pub name: String,
    /// Asset path of the ambient audio track.
    pub ambient_audio: String,
    /// Optional path to a glTF/GLB scene file (externally authored).
    pub gltf_scene: Option<String>,
    /// Procedural generator specifications.
    pub generators: Vec<GeneratorSpec>,
    /// Theme name to activate when this scene is active.
    pub theme: String,
}

impl SceneDescriptor {
    /// Log descriptor contents for debugging.
    pub fn log_info(&self) {
        trace!(
            "SceneDescriptor: name='{}' audio='{}' gltf={:?} generators={} theme='{}'",
            self.name,
            self.ambient_audio,
            self.gltf_scene,
            self.generators.len(),
            self.theme,
        );
    }
}

// ---------------------------------------------------------------------------
// GeneratorSpec
// ---------------------------------------------------------------------------

/// Specification for a single procedural generator invocation.
#[derive(Debug, Clone, Deserialize)]
pub struct GeneratorSpec {
    /// Generator type name (must match a registered factory).
    pub generator: String,
    /// Generator-specific parameters.
    pub params: GeneratorParams,
}

// ---------------------------------------------------------------------------
// GeneratorParams
// ---------------------------------------------------------------------------

/// Typed parameters for procedural generators.
/// Each variant corresponds to a specific generator factory.
#[derive(Debug, Clone, Deserialize)]
pub enum GeneratorParams {
    Starfield {
        count: usize,
        radius: f32,
        emissive: [f32; 4],
    },
    SpiralTrails {
        count: usize,
        speed: f32,
        emissive: [f32; 4],
        base_color: [f32; 4],
    },
    RiverBarges {
        count: usize,
        speed: f32,
        river_radius: f32,
        emissive: [f32; 4],
    },
    PathTravellers {
        count: usize,
        speed: f32,
        path_radius: f32,
        emissive: [f32; 4],
    },
    CloudLayer {
        count: usize,
        speed: f32,
        radius: f32,
        emissive: [f32; 4],
    },
    DropPods {
        count: usize,
        speed: f32,
        emissive: [f32; 4],
    },
    StaticGlowCluster {
        count: usize,
        emissive: [f32; 4],
        position: [f32; 3],
        spread: f32,
    },
    GltfSubscene {
        path: String,
        transform: Option<[f32; 3]>,
        scale: Option<f32>,
    },
}
