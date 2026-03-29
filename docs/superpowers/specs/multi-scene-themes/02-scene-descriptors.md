# PR B — Scene & Theme Descriptors (Data-Driven Loading)

**Parent:** [00-index.md](./00-index.md)
**Status:** Approved
**Depends on:** PR A

---

## Problem

Scenes and themes are currently hard-coded in Rust. Adding a new scene
requires recompilation. The user requires that scenes and themes be
**dynamically definable** via descriptor files (RON/JSON/TOML) loaded at
runtime without recompilation.

## Design Principles

- Descriptors define **what** to spawn (entity types, counts, colors, audio).
- Procedural generators (PR C) define **how** to spawn (mesh construction,
  placement algorithms).
- The engine reads descriptors, resolves generators, and renders. A new scene
  is added by writing a descriptor file — zero Rust code changes.
- Use `serde` for deserialization. RON is the primary format (Bevy ecosystem
  standard). JSON/TOML also supported via feature detection.

## Scene Descriptor Schema

### `src/scene/descriptor.rs` (~120 LOC)

```rust
/// Top-level scene descriptor, deserialized from a RON/JSON/TOML file.
#[derive(Debug, Clone, Deserialize)]
pub struct SceneDescriptor {
    /// Human-readable scene name (used as SceneManager key).
    pub name: String,
    /// Asset path of the ambient audio track.
    pub ambient_audio: String,
    /// Optional path to a glTF/GLB scene file (externally authored).
    /// If present, the engine loads this as the base scene.
    pub gltf_scene: Option<String>,
    /// Procedural generator specifications to layer on top.
    pub generators: Vec<GeneratorSpec>,
    /// Theme name to activate when this scene is active.
    pub theme: String,
}

/// Specification for a single procedural generator invocation.
#[derive(Debug, Clone, Deserialize)]
pub struct GeneratorSpec {
    /// Generator type name (must match a registered factory).
    /// Examples: "starfield", "spiral_trails", "river_barges",
    ///           "cloud_layer", "static_glow_cluster"
    pub generator: String,
    /// Generator-specific parameters as key-value pairs.
    /// Each generator defines its own expected params.
    pub params: GeneratorParams,
}

/// Typed parameters for procedural generators.
/// Each variant corresponds to a specific generator factory.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
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
```

### Theme Descriptor: `src/theme/descriptor.rs` (~80 LOC)

```rust
/// Theme descriptor, deserialized from a RON/JSON/TOML file.
#[derive(Debug, Clone, Deserialize)]
pub struct ThemeDescriptor {
    pub name: String,
    // UI palette
    pub background: [f32; 4],
    pub surface: [f32; 4],
    pub accent: [f32; 4],
    pub accent_secondary: [f32; 4],
    pub text_primary: [f32; 4],
    pub text_secondary: [f32; 4],
    pub success: [f32; 4],
    pub warning: [f32; 4],
    pub error: [f32; 4],
    pub panel_alpha: f32,
    pub panel_rounding: f32,
    // World-material colours
    pub star_emissive: [f32; 4],
    pub monolith_emissive: [f32; 4],
    pub trail_emissive: [f32; 4],
    pub trail_base_color: [f32; 4],
    pub monolith_glow_channels: [f32; 3],
    // Optional: font CSS URL
    pub font_css_url: Option<String>,
}

impl ThemeDescriptor {
    /// Convert to the runtime `Theme` struct.
    pub fn into_theme(self) -> Theme { … }
}
```

## Scene Loader: `src/scene/loader.rs` (~150 LOC)

```rust
/// Loads scene descriptors from disk and caches parsed results.
#[derive(Resource)]
pub struct SceneLoader {
    /// Directory to scan for scene descriptor files.
    scenes_dir: PathBuf,
    /// Cached parsed descriptors, keyed by scene name.
    cache: HashMap<String, SceneDescriptor>,
}

impl SceneLoader {
    pub fn new(scenes_dir: PathBuf) -> Self { … }

    /// Scan the scenes directory and parse all descriptor files.
    /// Called once at startup and on hot-reload signal.
    pub fn scan(&mut self) -> Result<usize, LoaderError> { … }

    /// Get a cached descriptor by scene name.
    pub fn get(&self, name: &str) -> Option<&SceneDescriptor> { … }

    /// List all available scene names.
    pub fn available(&self) -> Vec<&str> { … }
}
```

## Descriptor File Layout

```
assets/scenes/
├── tet.scene.ron
├── precursors.scene.ron
└── descent.scene.ron

assets/themes/
├── tet_orchestrator.theme.ron
├── precursors.theme.ron
└── descent.theme.ron
```

### Example: `tet.scene.ron`

```ron
SceneDescriptor(
    name: "TET",
    ambient_audio: "audio/tet_ambient.ogg",
    gltf_scene: None,
    theme: "TET Orchestrator",
    generators: [
        (generator: "starfield", params: Starfield(
            count: 2000,
            radius: 800.0,
            emissive: (2.0, 2.0, 2.0, 1.0),
        )),
        (generator: "spiral_trails", params: SpiralTrails(
            count: 100,
            speed: 5.0,
            emissive: (0.0, 1.5, 0.8, 1.0),
            base_color: (0.0, 0.8, 0.5, 1.0),
        )),
    ],
)
```

## Integration with Existing SceneConfig

A new `DescriptorSceneConfig` struct implements `SceneConfig` by reading
from a `SceneDescriptor`. This bridges the data-driven descriptor system
with the existing trait-based `SceneManager`:

```rust
pub struct DescriptorSceneConfig {
    descriptor: SceneDescriptor,
    audio_track: &'static str, // leaked from descriptor for &'static str compat
}

impl SceneConfig for DescriptorSceneConfig {
    fn name(&self) -> &str { &self.descriptor.name }
    fn ambient_audio_track(&self) -> &'static str { self.audio_track }
    fn spawn_environment(&self, commands, meshes, materials, theme) -> Entity {
        // Iterate descriptor.generators, resolve each to a generator factory,
        // call the factory to spawn entities under the scene root.
    }
}
```

Note: `audio_track` uses `Box::leak` on the descriptor's audio string to
produce a `&'static str`. This is acceptable because scene descriptors are
loaded once and live for the program's lifetime.

## Tests

New file: `tests/integration/scene_descriptors.rs`

| Test                                       | Verifies                                                       |
| ------------------------------------------ | -------------------------------------------------------------- |
| `t_desc_01_parse_ron_descriptor`           | Deserialize a RON scene descriptor string                      |
| `t_desc_02_parse_theme_descriptor`         | Deserialize a RON theme descriptor string                      |
| `t_desc_03_descriptor_to_scene_config`     | `DescriptorSceneConfig` implements `SceneConfig` correctly     |
| `t_desc_04_loader_scan_finds_descriptors`  | `SceneLoader::scan` finds files in a temp directory            |
| `t_desc_05_unknown_generator_errors`       | Descriptor with unknown generator name produces clear error    |
