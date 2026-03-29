# PR F — glTF Runtime Scene Loading

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Wire Bevy's native glTF support into the descriptor system so scenes can load externally authored 3D content (`.glb` / `.gltf` files) at runtime, both as a scene-wide base and as individual subscene fragments via the generator registry.

**Architecture:** `DescriptorSceneConfig::spawn_environment` reads the `gltf_scene` field from `SceneDescriptor` and spawns the glTF as a `SceneRoot` child of the scene root. A new `gen_gltf_subscene` generator factory uses `commands.queue(move |world: &mut World| { … })` to access `AssetServer` from the world (avoiding signature changes to `GeneratorFn`). FBX is NOT natively supported by Bevy — offline conversion only.

**Tech Stack:** Rust, Bevy 0.18.1

---

## File Map

| Action | Path | Responsibility |
| ------ | ---- | -------------- |
| Create | `src/scene/generators/gltf_subscene.rs` | `gen_gltf_subscene` generator factory |
| Modify | `src/scene/generators/mod.rs` | Add `pub mod gltf_subscene` + re-export |
| Modify | `src/scene/generators/registry.rs` | Register `"gltf_subscene"` in `with_builtins()` |
| Modify | `src/scene/descriptor_config.rs` | Load `gltf_scene` field via `AssetServer` |
| Modify | `tests/integration/scene_descriptors.rs` | 2 new glTF integration tests |
| Modify | `tests/integration.rs` | No change needed (module already declared) |

---

### Task 1: Create glTF subscene generator

**Files:**
- Create: `apps/deer_gui/src/scene/generators/gltf_subscene.rs`

- [ ] **Step 1: Write the generator**

```rust
//! glTF subscene generator — loads a glTF/GLB asset and spawns it as
//! a child of the scene root.
//!
//! Uses `commands.queue` to access [`AssetServer`] from the [`World`],
//! avoiding changes to the shared [`GeneratorFn`] signature.

use bevy::asset::Assets;
use bevy::ecs::system::Commands;
use bevy::ecs::world::World;
use bevy::log::{debug, trace, warn};
use bevy::math::Vec3;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{
    AssetServer, ChildOf, Entity, Handle, Mesh, Scene, SceneRoot, Transform,
};

use crate::scene::descriptor::GeneratorParams;

// ---------------------------------------------------------------------------
// Generator factory
// ---------------------------------------------------------------------------

/// Spawn a glTF scene fragment as a child of `root`.
///
/// Reads [`GeneratorParams::GltfSubscene`] for the asset path, optional
/// position, and optional uniform scale. The actual asset load is deferred
/// via `commands.queue` because [`AssetServer`] is a world resource not
/// available through the [`GeneratorFn`] signature.
pub fn gen_gltf_subscene(
    commands: &mut Commands,
    _meshes: &mut Assets<Mesh>,
    _materials: &mut Assets<StandardMaterial>,
    root: Entity,
    params: &GeneratorParams,
) {
    let GeneratorParams::GltfSubscene {
        path,
        transform,
        scale,
    } = params
    else {
        warn!("gen_gltf_subscene: expected GltfSubscene params, got {params:?}");
        return;
    };

    trace!(
        "gen_gltf_subscene: path='{path}' transform={transform:?} scale={scale:?}",
    );

    // Clone values into the closure (Commands::queue moves the closure).
    let path = path.clone();
    let transform_arr = *transform;
    let scale_val = *scale;

    commands.queue(move |world: &mut World| {
        let asset_server = world.resource::<AssetServer>();
        let scene_handle: Handle<Scene> = asset_server.load(format!("{path}#Scene0"));

        let t = build_transform(transform_arr, scale_val);

        world.commands().spawn((
            ChildOf(root),
            SceneRoot(scene_handle),
            t,
        ));

        debug!(
            "gen_gltf_subscene: queued glTF load path='{path}' root={root:?}",
        );
    });
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a [`Transform`] from optional position and scale arrays.
fn build_transform(position: Option<[f32; 3]>, scale: Option<f32>) -> Transform {
    let mut t = Transform::IDENTITY;

    if let Some(pos) = position {
        t = Transform::from_translation(Vec3::new(pos[0], pos[1], pos[2]));
        trace!("build_transform: position=({}, {}, {})", pos[0], pos[1], pos[2]);
    }

    if let Some(s) = scale {
        t = t.with_scale(Vec3::splat(s));
        trace!("build_transform: scale={s}");
    }

    t
}
```

---

### Task 2: Register glTF subscene generator in registry

**Files:**
- Modify: `apps/deer_gui/src/scene/generators/registry.rs`

- [ ] **Step 1: Add `gltf_subscene` to `with_builtins()`**

In `GeneratorRegistry::with_builtins()`, add after the existing `register` calls:

```rust
registry.register("gltf_subscene", super::gltf_subscene::gen_gltf_subscene);
```

The updated `with_builtins()` becomes:

```rust
    pub fn with_builtins() -> Self {
        let mut registry = Self::new();
        registry.register("starfield", super::starfield::gen_starfield);
        registry.register("spiral_trails", super::spiral_trails::gen_spiral_trails);
        registry.register("river_barges", super::river_barges::gen_river_barges);
        registry.register("path_travellers", super::path_travellers::gen_path_travellers);
        registry.register("cloud_layer", super::cloud_layer::gen_cloud_layer);
        registry.register("drop_pods", super::drop_pods::gen_drop_pods);
        registry.register("static_glow_cluster", super::static_glow::gen_static_glow_cluster);
        registry.register("gltf_subscene", super::gltf_subscene::gen_gltf_subscene);
        debug!(
            "GeneratorRegistry::with_builtins — {} generators",
            registry.factories.len(),
        );
        registry
    }
```

---

### Task 3: Wire generators module to export glTF subscene

**Files:**
- Modify: `apps/deer_gui/src/scene/generators/mod.rs`

- [ ] **Step 1: Add module declaration and re-export**

Add after the existing `pub mod static_glow;` line:

```rust
pub mod gltf_subscene;
```

The updated `mod.rs` becomes:

```rust
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

pub use cloud_layer::CloudParticle;
pub use drop_pods::DropPod;
pub use path_travellers::Traveller;
pub use registry::GeneratorRegistry;
pub use river_barges::Barge;
pub use spiral_trails::SpiralTrail;
pub use static_glow::GlowEntity;
```

---

### Task 4: Enhance `descriptor_config.rs` to load base glTF scene

**Files:**
- Modify: `apps/deer_gui/src/scene/descriptor_config.rs`

- [ ] **Step 1: Update `spawn_environment` to handle `gltf_scene` field**

The current `spawn_environment` spawns a root and logs generator warnings. Replace it with a version that also queues a base glTF load when `gltf_scene` is `Some(…)`.

Replace the existing `spawn_environment` method body with:

```rust
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

        // ── Base glTF scene ────────────────────────────────────────
        if let Some(gltf_path) = &self.descriptor.gltf_scene {
            debug!(
                "DescriptorSceneConfig: loading base glTF scene '{gltf_path}'",
            );
            let path = gltf_path.clone();
            commands.queue(move |world: &mut World| {
                let asset_server = world.resource::<AssetServer>();
                let scene_handle: Handle<Scene> =
                    asset_server.load(format!("{path}#Scene0"));

                world.commands().spawn((
                    ChildOf(root),
                    SceneRoot(scene_handle),
                ));

                debug!(
                    "DescriptorSceneConfig: queued base glTF '{path}' under root={root:?}",
                );
            });
        }

        // ── Procedural generators ──────────────────────────────────
        // TODO(PR-C): Iterate self.descriptor.generators, resolve each
        // via GeneratorRegistry, and call the factory. For now, log.
        for spec in &self.descriptor.generators {
            warn!(
                "Generator '{}' not yet wired (pending PR-C)",
                spec.generator,
            );
        }

        root
    }
```

- [ ] **Step 2: Add required imports to `descriptor_config.rs`**

Add at the top of the file, alongside the existing imports:

```rust
use bevy::ecs::world::World;
use bevy::prelude::{AssetServer, ChildOf, Handle, Scene, SceneRoot};
```

The full import block becomes:

```rust
use bevy::asset::Assets;
use bevy::ecs::system::Commands;
use bevy::ecs::world::World;
use bevy::log::{debug, info, warn};
use bevy::pbr::StandardMaterial;
use bevy::prelude::{AssetServer, ChildOf, Entity, Handle, Mesh, Scene, SceneRoot};

use super::descriptor::SceneDescriptor;
use super::primitives::spawn_root;
use super::traits::SceneConfig;
use crate::theme::ThemeManager;
```

---

### Task 5: Write integration tests

**Files:**
- Modify: `apps/deer_gui/tests/integration/scene_descriptors.rs`

- [ ] **Step 1: Add `t_gltf_01_descriptor_with_gltf_parses` test**

Append to the existing test file:

```rust
#[test]
fn t_gltf_01_descriptor_with_gltf_parses() {
    let ron_str = r#"
        SceneDescriptor(
            name: "GltfTest",
            ambient_audio: "audio/test.ogg",
            gltf_scene: Some("scenes/custom_level.glb"),
            theme: "TestTheme",
            generators: [
                (generator: "starfield", params: Starfield(
                    count: 100,
                    radius: 400.0,
                    emissive: (1.0, 1.0, 1.0, 1.0),
                )),
                (generator: "gltf_subscene", params: GltfSubscene(
                    path: "models/watchtower.glb",
                    transform: Some((0.0, 10.0, 0.0)),
                    scale: Some(2.0),
                )),
            ],
        )
    "#;
    let desc: SceneDescriptor = ron::from_str(ron_str).expect("should parse RON with gltf_scene");
    assert_eq!(desc.name, "GltfTest");
    assert_eq!(desc.gltf_scene, Some("scenes/custom_level.glb".to_string()));
    assert_eq!(desc.generators.len(), 2);

    // Verify GltfSubscene params parsed correctly.
    match &desc.generators[1].params {
        GeneratorParams::GltfSubscene { path, transform, scale } => {
            assert_eq!(path, "models/watchtower.glb");
            assert_eq!(*transform, Some([0.0, 10.0, 0.0]));
            assert!((scale.unwrap() - 2.0).abs() < 0.01);
        }
        _ => panic!("expected GltfSubscene params for second generator"),
    }
}
```

- [ ] **Step 2: Add `t_gltf_02_gltf_subscene_generator_runs` test**

Append to the existing test file:

```rust
#[test]
fn t_gltf_02_gltf_subscene_generator_runs() {
    use bevy::asset::AssetPlugin;
    use bevy::pbr::StandardMaterial;
    use bevy::prelude::*;

    use deer_gui::scene::generators::registry::GeneratorRegistry;
    use deer_gui::scene::primitives::spawn_root;

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<Mesh>();
    app.init_asset::<StandardMaterial>();
    app.init_asset::<Scene>();

    let registry = GeneratorRegistry::with_builtins();
    let gen = registry
        .get("gltf_subscene")
        .expect("gltf_subscene should be registered");

    let params = GeneratorParams::GltfSubscene {
        path: "models/test.glb".to_string(),
        transform: Some([1.0, 2.0, 3.0]),
        scale: Some(0.5),
    };

    // Run the generator — it queues a deferred command.
    app.world_mut()
        .resource_scope(|world: &mut World, mut meshes: Mut<Assets<Mesh>>| {
            world.resource_scope(
                |world: &mut World, mut mats: Mut<Assets<StandardMaterial>>| {
                    let mut commands = world.commands();
                    let root = spawn_root(&mut commands);
                    gen(&mut commands, &mut meshes, &mut mats, root, &params);
                },
            );
        });

    // Flush deferred commands (including the queued world closure).
    app.world_mut().flush();

    // The generator should have spawned a SceneRoot entity as a child.
    // Note: The asset won't actually load in a headless test (no render
    // backend), but the entity with the Handle<Scene> should exist.
    let scene_root_count = app
        .world_mut()
        .query_filtered::<Entity, With<bevy::prelude::SceneRoot>>()
        .iter(app.world())
        .count();

    // 1 from spawn_root (SceneRoot marker) + 1 from the glTF spawn.
    // spawn_root uses deer_gui::scene::SceneRoot (a custom component),
    // while the glTF spawn uses bevy::prelude::SceneRoot (Bevy's built-in).
    // We query Bevy's SceneRoot, so we expect exactly 1.
    assert!(
        scene_root_count >= 1,
        "gen_gltf_subscene should spawn an entity with Bevy SceneRoot; found {scene_root_count}",
    );
}
```

- [ ] **Step 3: Run all tests**

Run: `cargo test -p deer-gui 2>&1`
Expected: All existing tests + 2 new glTF tests pass.

- [ ] **Step 4: Run clippy**

Run: `cargo clippy -p deer-gui -- -D warnings 2>&1`
Expected: No warnings.

- [ ] **Step 5: Commit**

```bash
git add apps/deer_gui/src/scene/generators/gltf_subscene.rs \
       apps/deer_gui/src/scene/generators/mod.rs \
       apps/deer_gui/src/scene/generators/registry.rs \
       apps/deer_gui/src/scene/descriptor_config.rs \
       apps/deer_gui/tests/integration/scene_descriptors.rs
git commit -m "feat(scene): add glTF runtime loading via descriptor system and subscene generator"
```
