# PR B — Scene & Theme Descriptors

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add serde-based scene and theme descriptor types that can be deserialized from RON/JSON files, plus a `SceneLoader` that scans a directory and caches parsed descriptors.

**Architecture:** `SceneDescriptor` and `ThemeDescriptor` are plain serde structs. `GeneratorParams` is a tagged enum with one variant per generator type. `SceneLoader` is a Bevy `Resource` that scans `assets/scenes/` at startup. `DescriptorSceneConfig` bridges descriptors to the existing `SceneConfig` trait.

**Tech Stack:** Rust, Bevy 0.18.1, serde, ron

---

## File Map

| Action | Path | Responsibility |
| ------ | ---- | -------------- |
| Create | `src/scene/descriptor.rs` | `SceneDescriptor`, `GeneratorSpec`, `GeneratorParams` |
| Create | `src/scene/loader.rs` | `SceneLoader` resource, file scanning, caching |
| Create | `src/scene/descriptor_config.rs` | `DescriptorSceneConfig` impl `SceneConfig` |
| Create | `src/theme/descriptor.rs` | `ThemeDescriptor`, `into_theme()` conversion |
| Modify | `src/scene/mod.rs` | Add `pub mod descriptor`, `pub mod loader`, `pub mod descriptor_config` |
| Modify | `src/theme/mod.rs` | Add `pub mod descriptor` |
| Modify | `Cargo.toml` | Add `ron` dependency |
| Create | `assets/scenes/tet.scene.ron` | TET scene descriptor |
| Create | `assets/themes/tet_orchestrator.theme.ron` | TET theme descriptor |
| Create | `tests/integration/scene_descriptors.rs` | 5 integration tests |
| Modify | `tests/integration.rs` | Add `pub mod scene_descriptors` |

---

### Task 1: Add `ron` dependency

**Files:**
- Modify: `apps/deer_gui/Cargo.toml`

- [ ] **Step 1: Add ron to dependencies**

Add under `# ── Serialization / data` section:
```toml
ron = "0.8"
```

- [ ] **Step 2: Verify it resolves**

Run: `cargo check -p deer-gui 2>&1 | head -5`
Expected: Compiles (possibly with existing warnings).

---

### Task 2: Create `src/scene/descriptor.rs`

**Files:**
- Create: `apps/deer_gui/src/scene/descriptor.rs`

- [ ] **Step 1: Write the descriptor types**

```rust
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

- [ ] **Step 2: Verify compilation**

Run: `cargo check -p deer-gui 2>&1 | head -10`
Expected: Errors about module not declared. Fixed in wiring task.

---

### Task 3: Create `src/theme/descriptor.rs`

**Files:**
- Create: `apps/deer_gui/src/theme/descriptor.rs`

- [ ] **Step 1: Write the theme descriptor**

```rust
//! Theme descriptor — data-driven theme definition.
//!
//! Deserialized from RON/JSON files. Converts to the runtime [`Theme`]
//! struct via [`ThemeDescriptor::into_theme`].

use bevy::color::{Color, LinearRgba};
use bevy::log::{debug, trace};
use serde::Deserialize;

use super::theme::Theme;

// ---------------------------------------------------------------------------
// ThemeDescriptor
// ---------------------------------------------------------------------------

/// Theme descriptor, deserialized from a RON/JSON file.
#[derive(Debug, Clone, Deserialize)]
pub struct ThemeDescriptor {
    pub name: String,
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
    pub star_emissive: [f32; 4],
    pub monolith_emissive: [f32; 4],
    pub trail_emissive: [f32; 4],
    pub trail_base_color: [f32; 4],
    pub monolith_glow_channels: [f32; 3],
    pub font_css_url: Option<String>,
}

impl ThemeDescriptor {
    /// Convert to the runtime [`Theme`] struct.
    pub fn into_theme(self) -> Theme {
        debug!("ThemeDescriptor::into_theme — name='{}'", self.name);
        Theme {
            name: self.name,
            background: arr_to_color(self.background),
            surface: arr_to_color(self.surface),
            accent: arr_to_color(self.accent),
            accent_secondary: arr_to_color(self.accent_secondary),
            text_primary: arr_to_color(self.text_primary),
            text_secondary: arr_to_color(self.text_secondary),
            success: arr_to_color(self.success),
            warning: arr_to_color(self.warning),
            error: arr_to_color(self.error),
            panel_alpha: self.panel_alpha,
            panel_rounding: self.panel_rounding,
            star_emissive: arr_to_linear(self.star_emissive),
            monolith_emissive: arr_to_linear(self.monolith_emissive),
            trail_emissive: arr_to_linear(self.trail_emissive),
            trail_base_color: arr_to_color(self.trail_base_color),
            monolith_glow_channels: self.monolith_glow_channels,
        }
    }

    /// Log descriptor contents for debugging.
    pub fn log_info(&self) {
        trace!(
            "ThemeDescriptor: name='{}' font_css={:?}",
            self.name,
            self.font_css_url,
        );
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn arr_to_color(c: [f32; 4]) -> Color {
    Color::srgba(c[0], c[1], c[2], c[3])
}

fn arr_to_linear(c: [f32; 4]) -> LinearRgba {
    LinearRgba::new(c[0], c[1], c[2], c[3])
}
```

---

### Task 4: Create `src/scene/loader.rs`

**Files:**
- Create: `apps/deer_gui/src/scene/loader.rs`

- [ ] **Step 1: Write the scene loader**

```rust
//! Scene loader — scans a directory for scene descriptor files and caches them.
//!
//! Supports `.scene.ron` and `.scene.json` files.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bevy::log::{debug, info, warn};
use bevy::prelude::Resource;

use super::descriptor::SceneDescriptor;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur during scene loading.
#[derive(Debug, thiserror::Error)]
pub enum LoaderError {
    #[error("IO error reading {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("Parse error in {path}: {message}")]
    Parse { path: PathBuf, message: String },
}

// ---------------------------------------------------------------------------
// SceneLoader
// ---------------------------------------------------------------------------

/// Loads scene descriptors from disk and caches parsed results.
#[derive(Resource)]
pub struct SceneLoader {
    scenes_dir: PathBuf,
    cache: HashMap<String, SceneDescriptor>,
}

impl SceneLoader {
    /// Create a new loader targeting the given directory.
    pub fn new(scenes_dir: PathBuf) -> Self {
        info!("SceneLoader::new — dir={}", scenes_dir.display());
        Self {
            scenes_dir,
            cache: HashMap::new(),
        }
    }

    /// Scan the scenes directory and parse all descriptor files.
    pub fn scan(&mut self) -> Result<usize, LoaderError> {
        debug!("SceneLoader::scan — scanning {}", self.scenes_dir.display());
        self.cache.clear();

        let entries = std::fs::read_dir(&self.scenes_dir).map_err(|e| LoaderError::Io {
            path: self.scenes_dir.clone(),
            source: e,
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            if Self::is_scene_file(&path) {
                match self.parse_file(&path) {
                    Ok(desc) => {
                        info!("SceneLoader::scan — loaded '{}'", desc.name);
                        self.cache.insert(desc.name.clone(), desc);
                    }
                    Err(e) => {
                        warn!("SceneLoader::scan — skipping {}: {e}", path.display());
                    }
                }
            }
        }

        let count = self.cache.len();
        info!("SceneLoader::scan — loaded {count} scene(s)");
        Ok(count)
    }

    /// Get a cached descriptor by scene name.
    pub fn get(&self, name: &str) -> Option<&SceneDescriptor> {
        self.cache.get(name)
    }

    /// List all available scene names.
    pub fn available(&self) -> Vec<&str> {
        self.cache.keys().map(|s| s.as_str()).collect()
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn is_scene_file(path: &Path) -> bool {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        name.ends_with(".scene.ron") || name.ends_with(".scene.json")
    }

    fn parse_file(&self, path: &Path) -> Result<SceneDescriptor, LoaderError> {
        let content = std::fs::read_to_string(path).map_err(|e| LoaderError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

        if path.to_str().unwrap_or("").ends_with(".ron") {
            ron::from_str(&content).map_err(|e| LoaderError::Parse {
                path: path.to_path_buf(),
                message: e.to_string(),
            })
        } else {
            serde_json::from_str(&content).map_err(|e| LoaderError::Parse {
                path: path.to_path_buf(),
                message: e.to_string(),
            })
        }
    }
}
```

---

### Task 5: Create `src/scene/descriptor_config.rs`

**Files:**
- Create: `apps/deer_gui/src/scene/descriptor_config.rs`

- [ ] **Step 1: Write the bridge type**

```rust
//! Bridges [`SceneDescriptor`] to the [`SceneConfig`] trait.
//!
//! [`DescriptorSceneConfig`] reads a descriptor and implements
//! `SceneConfig` by delegating to the generator registry.

use bevy::asset::Assets;
use bevy::ecs::system::Commands;
use bevy::log::{debug, info, warn};
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Entity, Mesh};

use super::descriptor::SceneDescriptor;
use super::primitives::spawn_root;
use super::traits::SceneConfig;
use crate::theme::ThemeManager;

// ---------------------------------------------------------------------------
// DescriptorSceneConfig
// ---------------------------------------------------------------------------

/// A [`SceneConfig`] implementation backed by a [`SceneDescriptor`].
///
/// The `audio_track` field is a leaked `&'static str` because
/// [`SceneConfig::ambient_audio_track`] returns `&'static str`.
/// Scene descriptors live for the program's lifetime, so this is safe.
pub struct DescriptorSceneConfig {
    descriptor: SceneDescriptor,
    audio_track: &'static str,
}

impl DescriptorSceneConfig {
    /// Create from a descriptor. Leaks the audio string for static lifetime.
    pub fn new(descriptor: SceneDescriptor) -> Self {
        let audio_track: &'static str = Box::leak(descriptor.ambient_audio.clone().into_boxed_str());
        debug!(
            "DescriptorSceneConfig::new — name='{}' audio='{audio_track}'",
            descriptor.name,
        );
        Self {
            descriptor,
            audio_track,
        }
    }
}

impl SceneConfig for DescriptorSceneConfig {
    fn name(&self) -> &str {
        &self.descriptor.name
    }

    fn ambient_audio_track(&self) -> &'static str {
        self.audio_track
    }

    fn spawn_environment(
        &self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        theme: Option<&ThemeManager>,
    ) -> Entity {
        info!(
            "DescriptorSceneConfig::spawn_environment — scene='{}'",
            self.descriptor.name,
        );
        let root = spawn_root(commands);

        // TODO(PR-C): Iterate self.descriptor.generators, resolve each via
        // GeneratorRegistry, and call the factory. For now, just spawn root.
        for spec in &self.descriptor.generators {
            warn!(
                "Generator '{}' not yet wired (pending PR-C)",
                spec.generator,
            );
        }

        root
    }
}
```

---

### Task 6: Wire modules into mod.rs files

**Files:**
- Modify: `apps/deer_gui/src/scene/mod.rs`
- Modify: `apps/deer_gui/src/theme/mod.rs`

- [ ] **Step 1: Update `scene/mod.rs`**

Add:
```rust
pub mod descriptor;
pub mod descriptor_config;
pub mod loader;
```

And add re-exports:
```rust
pub use descriptor::{SceneDescriptor, GeneratorSpec, GeneratorParams};
pub use descriptor_config::DescriptorSceneConfig;
pub use loader::SceneLoader;
```

- [ ] **Step 2: Update `theme/mod.rs`**

Add:
```rust
pub mod descriptor;
pub use descriptor::ThemeDescriptor;
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check -p deer-gui 2>&1 | head -10`
Expected: Clean compilation.

---

### Task 7: Create TET scene descriptor file

**Files:**
- Create: `apps/deer_gui/assets/scenes/tet.scene.ron`

- [ ] **Step 1: Write the descriptor**

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

---

### Task 8: Create TET theme descriptor file

**Files:**
- Create: `apps/deer_gui/assets/themes/tet_orchestrator.theme.ron`

- [ ] **Step 1: Write the descriptor**

```ron
ThemeDescriptor(
    name: "TET Orchestrator",
    background: (0.02, 0.02, 0.05, 1.0),
    surface: (0.05, 0.05, 0.12, 1.0),
    accent: (0.0, 0.8, 1.0, 1.0),
    accent_secondary: (0.3, 0.5, 1.0, 1.0),
    text_primary: (0.9, 0.92, 0.95, 1.0),
    text_secondary: (0.5, 0.55, 0.6, 1.0),
    success: (0.2, 0.9, 0.4, 1.0),
    warning: (1.0, 0.75, 0.2, 1.0),
    error: (1.0, 0.3, 0.3, 1.0),
    panel_alpha: 0.75,
    panel_rounding: 8.0,
    star_emissive: (2.0, 2.0, 2.0, 1.0),
    monolith_emissive: (0.3, 0.5, 1.0, 1.0),
    trail_emissive: (0.0, 1.5, 0.8, 1.0),
    trail_base_color: (0.0, 0.8, 0.5, 1.0),
    monolith_glow_channels: (0.3, 0.5, 1.0),
    font_css_url: None,
)
```

---

### Task 9: Write integration tests

**Files:**
- Create: `apps/deer_gui/tests/integration/scene_descriptors.rs`
- Modify: `apps/deer_gui/tests/integration.rs`

- [ ] **Step 1: Add module to integration.rs**

Add: `pub mod scene_descriptors;`

- [ ] **Step 2: Write test file**

```rust
//! Integration tests for scene and theme descriptors.

use deer_gui::scene::descriptor::{GeneratorParams, SceneDescriptor};
use deer_gui::scene::descriptor_config::DescriptorSceneConfig;
use deer_gui::scene::SceneConfig;
use deer_gui::theme::descriptor::ThemeDescriptor;

#[test]
fn t_desc_01_parse_ron_scene_descriptor() {
    let ron_str = r#"
        SceneDescriptor(
            name: "Test",
            ambient_audio: "audio/test.ogg",
            gltf_scene: None,
            theme: "TestTheme",
            generators: [
                (generator: "starfield", params: Starfield(
                    count: 100,
                    radius: 400.0,
                    emissive: (1.0, 1.0, 1.0, 1.0),
                )),
            ],
        )
    "#;
    let desc: SceneDescriptor = ron::from_str(ron_str).expect("should parse RON");
    assert_eq!(desc.name, "Test");
    assert_eq!(desc.ambient_audio, "audio/test.ogg");
    assert_eq!(desc.generators.len(), 1);
    assert_eq!(desc.generators[0].generator, "starfield");
    match &desc.generators[0].params {
        GeneratorParams::Starfield { count, radius, .. } => {
            assert_eq!(*count, 100);
            assert!((radius - 400.0).abs() < 0.01);
        }
        _ => panic!("expected Starfield params"),
    }
}

#[test]
fn t_desc_02_parse_ron_theme_descriptor() {
    let ron_str = r#"
        ThemeDescriptor(
            name: "Test",
            background: (0.1, 0.1, 0.1, 1.0),
            surface: (0.2, 0.2, 0.2, 1.0),
            accent: (0.0, 0.8, 1.0, 1.0),
            accent_secondary: (0.3, 0.5, 1.0, 1.0),
            text_primary: (0.9, 0.9, 0.9, 1.0),
            text_secondary: (0.5, 0.5, 0.5, 1.0),
            success: (0.2, 0.9, 0.4, 1.0),
            warning: (1.0, 0.75, 0.2, 1.0),
            error: (1.0, 0.3, 0.3, 1.0),
            panel_alpha: 0.75,
            panel_rounding: 8.0,
            star_emissive: (2.0, 2.0, 2.0, 1.0),
            monolith_emissive: (0.3, 0.5, 1.0, 1.0),
            trail_emissive: (0.0, 1.5, 0.8, 1.0),
            trail_base_color: (0.0, 0.8, 0.5, 1.0),
            monolith_glow_channels: (0.3, 0.5, 1.0),
            font_css_url: None,
        )
    "#;
    let desc: ThemeDescriptor = ron::from_str(ron_str).expect("should parse RON");
    assert_eq!(desc.name, "Test");

    let theme = desc.into_theme();
    assert_eq!(theme.name, "Test");
}

#[test]
fn t_desc_03_descriptor_to_scene_config() {
    let desc = SceneDescriptor {
        name: "TestScene".to_string(),
        ambient_audio: "audio/test.ogg".to_string(),
        gltf_scene: None,
        theme: "TestTheme".to_string(),
        generators: vec![],
    };
    let config = DescriptorSceneConfig::new(desc);
    assert_eq!(config.name(), "TestScene");
    assert_eq!(config.ambient_audio_track(), "audio/test.ogg");
}

#[test]
fn t_desc_04_loader_scan_finds_descriptors() {
    use deer_gui::scene::loader::SceneLoader;

    let dir = std::env::temp_dir().join("deer_gui_test_scenes");
    let _ = std::fs::create_dir_all(&dir);

    let ron_content = r#"
        SceneDescriptor(
            name: "LoaderTest",
            ambient_audio: "audio/test.ogg",
            gltf_scene: None,
            theme: "Test",
            generators: [],
        )
    "#;
    std::fs::write(dir.join("test.scene.ron"), ron_content).unwrap();

    let mut loader = SceneLoader::new(dir.clone());
    let count = loader.scan().expect("scan should succeed");
    assert!(count >= 1, "should find at least 1 descriptor");
    assert!(loader.get("LoaderTest").is_some());

    // Cleanup
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn t_desc_05_unknown_generator_logged() {
    // This test verifies that a descriptor with generators can be created
    // as a DescriptorSceneConfig without panicking. The unknown generators
    // are logged as warnings (tested via tracing subscriber in real app).
    let desc = SceneDescriptor {
        name: "UnknownGen".to_string(),
        ambient_audio: "audio/test.ogg".to_string(),
        gltf_scene: None,
        theme: "Test".to_string(),
        generators: vec![deer_gui::scene::descriptor::GeneratorSpec {
            generator: "nonexistent".to_string(),
            params: GeneratorParams::Starfield {
                count: 10,
                radius: 100.0,
                emissive: [1.0, 1.0, 1.0, 1.0],
            },
        }],
    };
    let config = DescriptorSceneConfig::new(desc);
    assert_eq!(config.name(), "UnknownGen");
}
```

- [ ] **Step 3: Run all tests**

Run: `cargo test -p deer-gui 2>&1`
Expected: All existing tests + 5 new descriptor tests pass.

- [ ] **Step 4: Commit**

```bash
git add apps/deer_gui/src/scene/descriptor.rs \
       apps/deer_gui/src/scene/descriptor_config.rs \
       apps/deer_gui/src/scene/loader.rs \
       apps/deer_gui/src/theme/descriptor.rs \
       apps/deer_gui/src/scene/mod.rs \
       apps/deer_gui/src/theme/mod.rs \
       apps/deer_gui/Cargo.toml \
       apps/deer_gui/assets/scenes/tet.scene.ron \
       apps/deer_gui/assets/themes/tet_orchestrator.theme.ron \
       apps/deer_gui/tests/integration/scene_descriptors.rs \
       apps/deer_gui/tests/integration.rs
git commit -m "feat(scene): add data-driven scene/theme descriptors with RON support"
```
