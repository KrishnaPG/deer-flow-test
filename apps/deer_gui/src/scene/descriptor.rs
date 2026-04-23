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
    Monolith {
        radius: f32,
        emissive: [f32; 4],
    },
    GltfSubscene {
        path: String,
        transform: Option<[f32; 3]>,
        scale: Option<f32>,
    },
    /// Heightmap terrain generator for medieval open world.
    /// Creates terrain from a heightmap image with texture splatting.
    Terrain {
        /// Path to the heightmap image (PNG, grayscale).
        heightmap: String,
        /// World-space size (width, depth) in meters.
        world_size: [f32; 2],
        /// Maximum height in meters (default: 100.0).
        #[serde(default = "default_height_scale")]
        height_scale: f32,
        /// Base terrain elevation offset (default: 0.0).
        #[serde(default)]
        height_offset: f32,
        /// Mesh resolution per chunk (default: 64).
        #[serde(default = "default_terrain_resolution")]
        resolution: u32,
        /// Chunk size in world units (default: 100.0).
        #[serde(default = "default_chunk_size")]
        chunk_size: f32,
        /// Path to splat mask texture for layer blending (optional).
        #[serde(default)]
        splat_mask: Option<String>,
        /// Paths to terrain layer textures [grass, dirt, rock, snow].
        #[serde(default)]
        layer_textures: [Option<String>; 4],
        /// UV scale for terrain tiling (default: 10.0).
        #[serde(default = "default_uv_scale")]
        uv_scale: f32,
        /// Invert heightmap values (default: false).
        #[serde(default)]
        invert_heightmap: bool,
    },
    /// Vegetation generator for forests, meadows, and biomes.
    /// Scatters trees, bushes, and grass based on biome configuration.
    Vegetation {
        /// Biome type (Meadow, Forest, Rocky, Coastal, Alpine, Riverbank).
        biome: String,
        /// Center position for vegetation scatter.
        center: [f32; 3],
        /// Radius in meters for vegetation spread.
        radius: f32,
        /// Density multiplier (default: 1.0).
        #[serde(default = "default_vegetation_density")]
        density: f32,
        /// Enable wind animation (default: true).
        #[serde(default = "default_true_bool")]
        enable_wind: bool,
        /// Maximum vegetation draw distance in meters (default: 500.0).
        #[serde(default = "default_vegetation_draw_distance")]
        draw_distance: f32,
    },
}

fn default_height_scale() -> f32 {
    100.0
}

fn default_terrain_resolution() -> u32 {
    64
}

fn default_chunk_size() -> f32 {
    100.0
}

fn default_uv_scale() -> f32 {
    10.0
}

fn default_vegetation_density() -> f32 {
    1.0
}

fn default_true_bool() -> bool {
    true
}

fn default_vegetation_draw_distance() -> f32 {
    500.0
}
