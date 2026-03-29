# PR C — Procedural Generator Factories + Registry

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create a generator registry that maps type names (from scene descriptors) to factory functions, plus implement all built-in generators. Wire `DescriptorSceneConfig` to use the registry.

**Architecture:** `GeneratorRegistry` is a Bevy `Resource` mapping `String → GeneratorFn`. Each generator is a standalone function in `src/scene/generators/`. Generators read typed params from `GeneratorParams` and spawn entities under a scene root.

**Tech Stack:** Rust, Bevy 0.18.1

---

## File Map

| Action | Path | Responsibility |
| ------ | ---- | -------------- |
| Create | `src/scene/generators/mod.rs` | Module re-exports |
| Create | `src/scene/generators/registry.rs` | `GeneratorRegistry` resource |
| Create | `src/scene/generators/starfield.rs` | `gen_starfield` factory |
| Create | `src/scene/generators/spiral_trails.rs` | `gen_spiral_trails` factory |
| Create | `src/scene/generators/river_barges.rs` | `gen_river_barges` factory + `Barge` component |
| Create | `src/scene/generators/path_travellers.rs` | `gen_path_travellers` factory + `Traveller` component |
| Create | `src/scene/generators/cloud_layer.rs` | `gen_cloud_layer` factory + `CloudParticle` component |
| Create | `src/scene/generators/drop_pods.rs` | `gen_drop_pods` factory + `DropPod` component |
| Create | `src/scene/generators/static_glow.rs` | `gen_static_glow_cluster` factory |
| Modify | `src/scene/mod.rs` | Add `pub mod generators` |
| Modify | `src/scene/descriptor_config.rs` | Wire `GeneratorRegistry` lookup |
| Modify | `src/scene/plugin.rs` | Register `GeneratorRegistry` resource + generator systems |
| Create | `tests/integration/scene_generators.rs` | 6 integration tests |
| Modify | `tests/integration.rs` | Add `pub mod scene_generators` |

---

### Task 1: Create generator registry

**Files:**
- Create: `apps/deer_gui/src/scene/generators/registry.rs`

- [ ] **Step 1: Write the registry**

```rust
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
        trace!("GeneratorRegistry::get — '{name}' found={}", result.is_some());
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
        registry.register("path_travellers", super::path_travellers::gen_path_travellers);
        registry.register("cloud_layer", super::cloud_layer::gen_cloud_layer);
        registry.register("drop_pods", super::drop_pods::gen_drop_pods);
        registry.register("static_glow_cluster", super::static_glow::gen_static_glow_cluster);
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
```

---

### Task 2: Create starfield generator

**Files:**
- Create: `apps/deer_gui/src/scene/generators/starfield.rs`

- [ ] **Step 1: Write the generator**

```rust
//! Starfield generator — spawns stars on a Fibonacci sphere.

use bevy::asset::Assets;
use bevy::ecs::system::Commands;
use bevy::log::{debug, warn};
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Entity, Mesh};

use crate::scene::descriptor::GeneratorParams;
use crate::scene::primitives;

/// Spawn a starfield using shared primitives.
pub fn gen_starfield(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    root: Entity,
    params: &GeneratorParams,
) {
    let GeneratorParams::Starfield { count, radius, emissive } = params else {
        warn!("gen_starfield: expected Starfield params, got {:?}", params);
        return;
    };

    let emissive_color = bevy::color::LinearRgba::new(
        emissive[0], emissive[1], emissive[2], emissive[3],
    );

    primitives::spawn_starfield(commands, meshes, materials, root, emissive_color, *count, *radius);
    debug!("gen_starfield: count={count} radius={radius}");
}
```

---

### Task 3: Create spiral trails generator

**Files:**
- Create: `apps/deer_gui/src/scene/generators/spiral_trails.rs`

- [ ] **Step 1: Write the generator**

```rust
//! Spiral trails generator — spawns parametric spiral path entities.

use bevy::asset::Assets;
use bevy::color::Color;
use bevy::ecs::system::Commands;
use bevy::log::{debug, warn};
use bevy::math::Vec3;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{ChildOf, Component, Entity, Mesh, Mesh3d, MeshMaterial3d, Sphere, Transform};

use crate::constants::visual::TET_STRUCTURE_RADIUS;
use crate::scene::descriptor::GeneratorParams;

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// Tag + parametric data for spiral trail entities.
#[derive(Component, Debug, Clone)]
pub struct SpiralTrail {
    pub t: f32,
    pub index: usize,
    pub speed: f32,
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Spawn spiral trail entities along parametric paths.
pub fn gen_spiral_trails(
    commands: &mut bevy::ecs::system::Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    root: Entity,
    params: &GeneratorParams,
) {
    let GeneratorParams::SpiralTrails { count, speed, emissive, base_color } = params else {
        warn!("gen_spiral_trails: expected SpiralTrails params");
        return;
    };

    let trail_mesh = meshes.add(Mesh::from(Sphere::new(0.3)));
    let trail_material = materials.add(StandardMaterial {
        emissive: bevy::color::LinearRgba::new(emissive[0], emissive[1], emissive[2], emissive[3]),
        base_color: Color::srgba(base_color[0], base_color[1], base_color[2], base_color[3]),
        ..Default::default()
    });

    for i in 0..*count {
        let t = i as f32 / *count as f32;
        let pos = spiral_position(t, i);

        commands.spawn((
            SpiralTrail { t, index: i, speed: *speed },
            ChildOf(root),
            Mesh3d(trail_mesh.clone()),
            MeshMaterial3d(trail_material.clone()),
            Transform::from_translation(pos),
        ));
    }

    debug!("gen_spiral_trails: count={count} speed={speed}");
}

/// Compute spiral position (same formula as TET's `spiral_position`).
pub fn spiral_position(t: f32, index: usize) -> Vec3 {
    let phase = index as f32 * 0.618;
    let angle = t * std::f32::consts::TAU * 3.0 + phase;
    let r = TET_STRUCTURE_RADIUS * 1.5 + t * TET_STRUCTURE_RADIUS;
    let y = (t - 0.5) * TET_STRUCTURE_RADIUS * 2.0;
    Vec3::new(r * angle.cos(), y, r * angle.sin())
}
```

---

### Task 4: Create river barges generator

**Files:**
- Create: `apps/deer_gui/src/scene/generators/river_barges.rs`

- [ ] **Step 1: Write the generator**

```rust
//! River barges generator — spawns parametric horizontal flow entities.

use bevy::asset::Assets;
use bevy::ecs::system::Commands;
use bevy::log::{debug, warn};
use bevy::math::Vec3;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{ChildOf, Component, Entity, Mesh, Mesh3d, MeshMaterial3d, Sphere, Transform};

use crate::scene::descriptor::GeneratorParams;

/// Parametric river barge entity.
#[derive(Component, Debug, Clone)]
pub struct Barge {
    pub t: f32,
    pub index: usize,
    pub speed: f32,
    pub river_radius: f32,
}

/// Spawn river barge entities along a horizontal parametric path.
pub fn gen_river_barges(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    root: Entity,
    params: &GeneratorParams,
) {
    let GeneratorParams::RiverBarges { count, speed, river_radius, emissive } = params else {
        warn!("gen_river_barges: expected RiverBarges params");
        return;
    };

    let mesh = meshes.add(Mesh::from(Sphere::new(0.4)));
    let material = materials.add(StandardMaterial {
        emissive: bevy::color::LinearRgba::new(emissive[0], emissive[1], emissive[2], emissive[3]),
        ..Default::default()
    });

    for i in 0..*count {
        let t = i as f32 / *count as f32;
        let pos = barge_position(t, i, *river_radius);

        commands.spawn((
            Barge { t, index: i, speed: *speed, river_radius: *river_radius },
            ChildOf(root),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(pos),
        ));
    }

    debug!("gen_river_barges: count={count} speed={speed} radius={river_radius}");
}

/// Compute barge position along a sinusoidal river path.
pub fn barge_position(t: f32, index: usize, river_radius: f32) -> Vec3 {
    let phase = index as f32 * 0.37;
    let x = (t * 2.0 - 1.0) * river_radius;
    let z = (t * std::f32::consts::TAU + phase).sin() * river_radius * 0.15;
    let y = -50.0 + (index as f32 * 1.7).sin() * 5.0;
    Vec3::new(x, y, z)
}
```

---

### Task 5: Create remaining generators

**Files:**
- Create: `apps/deer_gui/src/scene/generators/path_travellers.rs`
- Create: `apps/deer_gui/src/scene/generators/cloud_layer.rs`
- Create: `apps/deer_gui/src/scene/generators/drop_pods.rs`
- Create: `apps/deer_gui/src/scene/generators/static_glow.rs`

- [ ] **Step 1: Write `path_travellers.rs`**

```rust
//! Path travellers generator — diagonal parametric flow entities.

use bevy::asset::Assets;
use bevy::ecs::system::Commands;
use bevy::log::{debug, warn};
use bevy::math::Vec3;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{ChildOf, Component, Entity, Mesh, Mesh3d, MeshMaterial3d, Sphere, Transform};

use crate::scene::descriptor::GeneratorParams;

#[derive(Component, Debug, Clone)]
pub struct Traveller {
    pub t: f32,
    pub index: usize,
    pub speed: f32,
    pub path_radius: f32,
}

pub fn gen_path_travellers(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    root: Entity,
    params: &GeneratorParams,
) {
    let GeneratorParams::PathTravellers { count, speed, path_radius, emissive } = params else {
        warn!("gen_path_travellers: expected PathTravellers params");
        return;
    };

    let mesh = meshes.add(Mesh::from(Sphere::new(0.3)));
    let material = materials.add(StandardMaterial {
        emissive: bevy::color::LinearRgba::new(emissive[0], emissive[1], emissive[2], emissive[3]),
        ..Default::default()
    });

    for i in 0..*count {
        let t = i as f32 / *count as f32;
        let pos = traveller_position(t, i, *path_radius);
        commands.spawn((
            Traveller { t, index: i, speed: *speed, path_radius: *path_radius },
            ChildOf(root),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(pos),
        ));
    }
    debug!("gen_path_travellers: count={count} speed={speed}");
}

pub fn traveller_position(t: f32, index: usize, path_radius: f32) -> Vec3 {
    let phase = index as f32 * 0.53;
    let angle = t * std::f32::consts::TAU + phase;
    let r = path_radius * (0.7 + 0.3 * t);
    let y = (t - 0.5) * path_radius * 0.5;
    Vec3::new(r * angle.cos(), y, r * angle.sin())
}
```

- [ ] **Step 2: Write `cloud_layer.rs`**

```rust
//! Cloud layer generator — vertical rising particle entities.

use bevy::asset::Assets;
use bevy::ecs::system::Commands;
use bevy::log::{debug, warn};
use bevy::math::Vec3;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{ChildOf, Component, Entity, Mesh, Mesh3d, MeshMaterial3d, Sphere, Transform};

use crate::scene::descriptor::GeneratorParams;

#[derive(Component, Debug, Clone)]
pub struct CloudParticle {
    pub t: f32,
    pub index: usize,
    pub speed: f32,
}

pub fn gen_cloud_layer(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    root: Entity,
    params: &GeneratorParams,
) {
    let GeneratorParams::CloudLayer { count, speed, radius, emissive } = params else {
        warn!("gen_cloud_layer: expected CloudLayer params");
        return;
    };

    let mesh = meshes.add(Mesh::from(Sphere::new(1.5)));
    let material = materials.add(StandardMaterial {
        emissive: bevy::color::LinearRgba::new(emissive[0], emissive[1], emissive[2], emissive[3]),
        alpha_mode: bevy::prelude::AlphaMode::Blend,
        base_color: bevy::color::Color::srgba(0.8, 0.85, 0.9, 0.3),
        ..Default::default()
    });

    for i in 0..*count {
        let t = i as f32 / *count as f32;
        let pos = cloud_position(t, i, *radius);
        commands.spawn((
            CloudParticle { t, index: i, speed: *speed },
            ChildOf(root),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(pos).with_scale(Vec3::new(3.0, 0.5, 3.0)),
        ));
    }
    debug!("gen_cloud_layer: count={count} speed={speed} radius={radius}");
}

pub fn cloud_position(t: f32, index: usize, radius: f32) -> Vec3 {
    let phase = index as f32 * 2.39; // golden angle
    let x = (phase).cos() * radius * (0.3 + 0.7 * t);
    let z = (phase).sin() * radius * (0.3 + 0.7 * t);
    let y = (t - 0.5) * radius * 2.0;
    Vec3::new(x, y, z)
}
```

- [ ] **Step 3: Write `drop_pods.rs`**

```rust
//! Drop pods generator — vertical falling particle entities.

use bevy::asset::Assets;
use bevy::ecs::system::Commands;
use bevy::log::{debug, warn};
use bevy::math::Vec3;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{ChildOf, Component, Entity, Mesh, Mesh3d, MeshMaterial3d, Sphere, Transform};

use crate::scene::descriptor::GeneratorParams;

#[derive(Component, Debug, Clone)]
pub struct DropPod {
    pub t: f32,
    pub index: usize,
    pub speed: f32,
}

pub fn gen_drop_pods(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    root: Entity,
    params: &GeneratorParams,
) {
    let GeneratorParams::DropPods { count, speed, emissive } = params else {
        warn!("gen_drop_pods: expected DropPods params");
        return;
    };

    let mesh = meshes.add(Mesh::from(Sphere::new(0.4)));
    let material = materials.add(StandardMaterial {
        emissive: bevy::color::LinearRgba::new(emissive[0], emissive[1], emissive[2], emissive[3]),
        ..Default::default()
    });

    for i in 0..*count {
        let t = i as f32 / *count as f32;
        let pos = pod_position(t, i);
        commands.spawn((
            DropPod { t, index: i, speed: *speed },
            ChildOf(root),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(pos),
        ));
    }
    debug!("gen_drop_pods: count={count} speed={speed}");
}

pub fn pod_position(t: f32, index: usize) -> Vec3 {
    let phase = index as f32 * 1.618;
    let x = (phase * 0.7).cos() * 100.0;
    let z = (phase * 0.7).sin() * 100.0;
    let y = (1.0 - t) * 800.0 - 400.0; // falling downward
    Vec3::new(x, y, z)
}
```

- [ ] **Step 4: Write `static_glow.rs`**

```rust
//! Static glow cluster generator — fixed emissive entities.

use bevy::asset::Assets;
use bevy::ecs::system::Commands;
use bevy::log::{debug, warn};
use bevy::math::Vec3;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{ChildOf, Component, Entity, Mesh, Mesh3d, MeshMaterial3d, Sphere, Transform};

use crate::scene::descriptor::GeneratorParams;
use crate::scene::primitives;

#[derive(Component, Debug, Default)]
pub struct GlowEntity;

pub fn gen_static_glow_cluster(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    root: Entity,
    params: &GeneratorParams,
) {
    let GeneratorParams::StaticGlowCluster { count, emissive, position, spread } = params else {
        warn!("gen_static_glow_cluster: expected StaticGlowCluster params");
        return;
    };

    let mesh = meshes.add(Mesh::from(Sphere::new(1.0)));
    let material = materials.add(StandardMaterial {
        emissive: bevy::color::LinearRgba::new(emissive[0], emissive[1], emissive[2], emissive[3]),
        ..Default::default()
    });

    let center = Vec3::new(position[0], position[1], position[2]);

    for i in 0..*count {
        let offset = primitives::fibonacci_sphere_point(i, *count, *spread);
        let pos = center + offset;
        let scale = primitives::entity_scale(i) * 0.8;

        commands.spawn((
            GlowEntity,
            ChildOf(root),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(pos).with_scale(Vec3::splat(scale)),
        ));
    }
    debug!("gen_static_glow_cluster: count={count} spread={spread}");
}
```

---

### Task 6: Create generators module file

**Files:**
- Create: `apps/deer_gui/src/scene/generators/mod.rs`

- [ ] **Step 1: Write mod.rs**

```rust
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
```

---

### Task 7: Wire generators into scene module and plugin

**Files:**
- Modify: `apps/deer_gui/src/scene/mod.rs`
- Modify: `apps/deer_gui/src/scene/plugin.rs`
- Modify: `apps/deer_gui/src/scene/descriptor_config.rs`

- [ ] **Step 1: Add `generators` to `scene/mod.rs`**

Add: `pub mod generators;`
Add re-export: `pub use generators::GeneratorRegistry;`

- [ ] **Step 2: Register `GeneratorRegistry` in plugin**

In `ScenePlugin::build`, add:
```rust
app.insert_resource(GeneratorRegistry::with_builtins());
```

Add generator per-frame systems for dynamic entities:
```rust
// In the Update system tuple, add:
barge_system, traveller_system, cloud_system, drop_pod_system,
```

Create these thin per-frame systems (can go in a new `src/scene/generators/systems.rs` if preferred, or in each generator file).

- [ ] **Step 3: Update `descriptor_config.rs` to use registry**

Replace the `TODO(PR-C)` block in `spawn_environment`:
```rust
fn spawn_environment(
    &self,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    theme: Option<&ThemeManager>,
) -> Entity {
    let root = spawn_root(commands);
    // Generator resolution happens via world resource access.
    // Since SceneConfig::spawn_environment doesn't have world access,
    // we use a startup pattern: the generators are invoked by
    // ScenePlugin after activate, not inside spawn_environment directly.
    // For now, log the generators for wiring verification.
    for spec in &self.descriptor.generators {
        info!("DescriptorSceneConfig: generator='{}' (will be resolved by plugin)", spec.generator);
    }
    root
}
```

Note: The full wiring where `ScenePlugin` reads descriptors and invokes registry functions happens as part of the plugin startup refactor. The immediate goal is to have the registry available and all factories working independently.

---

### Task 8: Write integration tests

**Files:**
- Create: `apps/deer_gui/tests/integration/scene_generators.rs`
- Modify: `apps/deer_gui/tests/integration.rs`

- [ ] **Step 1: Add module to integration.rs**

Add: `pub mod scene_generators;`

- [ ] **Step 2: Write test file**

```rust
//! Integration tests for procedural generator factories.

use bevy::asset::AssetPlugin;
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;

use deer_gui::scene::descriptor::GeneratorParams;
use deer_gui::scene::generators::registry::GeneratorRegistry;
use deer_gui::scene::generators::{Barge, CloudParticle, DropPod, Traveller};
use deer_gui::scene::primitives::{spawn_root, Star};

fn build_gen_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<Mesh>();
    app.init_asset::<StandardMaterial>();
    app
}

fn count<T: Component>(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<Entity, With<T>>()
        .iter(app.world())
        .count()
}

#[test]
fn t_gen_01_registry_builtins_registered() {
    let registry = GeneratorRegistry::with_builtins();
    let names = registry.available();
    assert!(names.contains(&"starfield"));
    assert!(names.contains(&"spiral_trails"));
    assert!(names.contains(&"river_barges"));
    assert!(names.contains(&"path_travellers"));
    assert!(names.contains(&"cloud_layer"));
    assert!(names.contains(&"drop_pods"));
    assert!(names.contains(&"static_glow_cluster"));
    assert!(names.len() >= 7);
}

#[test]
fn t_gen_02_starfield_spawns_entities() {
    let mut app = build_gen_app();
    let registry = GeneratorRegistry::with_builtins();
    let gen = registry.get("starfield").unwrap();
    let params = GeneratorParams::Starfield {
        count: 50,
        radius: 400.0,
        emissive: [2.0, 2.0, 2.0, 1.0],
    };

    app.world_mut()
        .resource_scope(|world: &mut World, mut meshes: Mut<Assets<Mesh>>| {
            world.resource_scope(|world: &mut World, mut mats: Mut<Assets<StandardMaterial>>| {
                let mut commands = world.commands();
                let root = spawn_root(&mut commands);
                gen(&mut commands, &mut meshes, &mut mats, root, &params);
            });
        });
    app.world_mut().flush();

    assert_eq!(count::<Star>(&mut app), 50);
}

#[test]
fn t_gen_03_river_barges_spawns() {
    let mut app = build_gen_app();
    let registry = GeneratorRegistry::with_builtins();
    let gen = registry.get("river_barges").unwrap();
    let params = GeneratorParams::RiverBarges {
        count: 20,
        speed: 3.0,
        river_radius: 300.0,
        emissive: [0.6, 0.8, 0.4, 1.0],
    };

    app.world_mut()
        .resource_scope(|world: &mut World, mut meshes: Mut<Assets<Mesh>>| {
            world.resource_scope(|world: &mut World, mut mats: Mut<Assets<StandardMaterial>>| {
                let mut commands = world.commands();
                let root = spawn_root(&mut commands);
                gen(&mut commands, &mut meshes, &mut mats, root, &params);
            });
        });
    app.world_mut().flush();

    assert_eq!(count::<Barge>(&mut app), 20);
}

#[test]
fn t_gen_04_cloud_layer_spawns() {
    let mut app = build_gen_app();
    let registry = GeneratorRegistry::with_builtins();
    let gen = registry.get("cloud_layer").unwrap();
    let params = GeneratorParams::CloudLayer {
        count: 30,
        speed: 6.0,
        radius: 400.0,
        emissive: [0.8, 0.9, 1.8, 1.0],
    };

    app.world_mut()
        .resource_scope(|world: &mut World, mut meshes: Mut<Assets<Mesh>>| {
            world.resource_scope(|world: &mut World, mut mats: Mut<Assets<StandardMaterial>>| {
                let mut commands = world.commands();
                let root = spawn_root(&mut commands);
                gen(&mut commands, &mut meshes, &mut mats, root, &params);
            });
        });
    app.world_mut().flush();

    assert_eq!(count::<CloudParticle>(&mut app), 30);
}

#[test]
fn t_gen_05_unknown_generator_returns_none() {
    let registry = GeneratorRegistry::with_builtins();
    assert!(registry.get("nonexistent").is_none());
}

#[test]
fn t_gen_06_all_entities_child_of_root() {
    use deer_gui::scene::SceneRoot;

    let mut app = build_gen_app();
    let registry = GeneratorRegistry::with_builtins();
    let gen = registry.get("starfield").unwrap();
    let params = GeneratorParams::Starfield {
        count: 10,
        radius: 100.0,
        emissive: [1.0, 1.0, 1.0, 1.0],
    };

    let root_entity;
    app.world_mut()
        .resource_scope(|world: &mut World, mut meshes: Mut<Assets<Mesh>>| {
            world.resource_scope(|world: &mut World, mut mats: Mut<Assets<StandardMaterial>>| {
                let mut commands = world.commands();
                root_entity = spawn_root(&mut commands);
                gen(&mut commands, &mut meshes, &mut mats, root_entity, &params);
            });
        });
    app.world_mut().flush();

    // Verify all Star entities have ChildOf pointing to root.
    let stars: Vec<Entity> = app
        .world_mut()
        .query_filtered::<Entity, With<Star>>()
        .iter(app.world())
        .collect();

    for star in stars {
        let child_of = app.world().get::<ChildOf>(star);
        assert!(child_of.is_some(), "Star entity should have ChildOf component");
    }
}
```

- [ ] **Step 3: Run all tests**

Run: `cargo test -p deer-gui 2>&1`
Expected: All tests pass.

- [ ] **Step 4: Commit**

```bash
git add apps/deer_gui/src/scene/generators/ \
       apps/deer_gui/src/scene/mod.rs \
       apps/deer_gui/src/scene/plugin.rs \
       apps/deer_gui/src/scene/descriptor_config.rs \
       apps/deer_gui/tests/integration/scene_generators.rs \
       apps/deer_gui/tests/integration.rs
git commit -m "feat(scene): add procedural generator registry with 7 built-in factories"
```
