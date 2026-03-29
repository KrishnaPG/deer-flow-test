# PR A — Shared Primitives + TET Refactor

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Extract reusable spawn helpers from TET into `scene/primitives.rs` and refactor TET to use them, keeping all existing tests green.

**Architecture:** Move `Star` component, `pseudo_random_sphere_point`, `random_scale`, and `spawn_starfield` into a shared `primitives` module. TET delegates to these. No behaviour change.

**Tech Stack:** Rust, Bevy 0.18.1

---

## File Map

| Action | Path | Responsibility |
| ------ | ---- | -------------- |
| Create | `src/scene/primitives.rs` | `Star`, `spawn_root`, `spawn_starfield`, `fibonacci_sphere_point`, `entity_scale`, `spawn_scene_ambient_light` |
| Modify | `src/scene/mod.rs` | Add `pub mod primitives` + re-exports |
| Modify | `src/scene/tet/setup.rs` | Remove duplicated helpers, delegate to primitives |
| Create | `tests/integration/scene_primitives.rs` | 5 integration tests |
| Modify | `tests/integration.rs` | Add `pub mod scene_primitives` |

---

### Task 1: Create `src/scene/primitives.rs`

**Files:**
- Create: `apps/deer_gui/src/scene/primitives.rs`

- [ ] **Step 1: Write the primitives module**

```rust
//! Shared scene primitives — reusable spawn helpers for all scenes.
//!
//! Scene-independent functions for spawning starfields, ambient lights,
//! and computing Fibonacci-sphere distributions.

use bevy::asset::Assets;
use bevy::log::{debug, trace};
use bevy::math::Vec3;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{ChildOf, Component, Entity, Mesh, Mesh3d, MeshMaterial3d, Sphere, Transform};

use crate::scene::common::parallax::ParallaxLayer;
use crate::scene::manager::SceneRoot;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Tag for starfield entities. Scene-independent — used by any scene
/// that spawns a Fibonacci-sphere star distribution.
#[derive(Component, Debug, Default)]
pub struct Star;

// ---------------------------------------------------------------------------
// Spawn helpers
// ---------------------------------------------------------------------------

/// Spawn a [`SceneRoot`] entity and return its [`Entity`] id.
///
/// The root entity serves as the parent for all scene-specific entities.
/// Despawning it recursively removes the entire scene.
pub fn spawn_root(commands: &mut bevy::ecs::system::Commands) -> Entity {
    let root = commands.spawn((SceneRoot, Transform::default())).id();
    debug!("primitives::spawn_root — root={root:?}");
    root
}

/// Spawn `count` stars distributed on a Fibonacci sphere of `radius`.
///
/// Each star is an emissive sphere mesh with a [`ParallaxLayer`] for
/// depth-based parallax scrolling. All entities are `ChildOf(root)`.
pub fn spawn_starfield(
    commands: &mut bevy::ecs::system::Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    root: Entity,
    emissive: bevy::color::LinearRgba,
    count: usize,
    radius: f32,
) {
    let star_mesh = meshes.add(Mesh::from(Sphere::new(0.5)));
    let star_material = materials.add(StandardMaterial {
        emissive,
        ..Default::default()
    });

    for i in 0..count {
        let pos = fibonacci_sphere_point(i, count, radius);
        let depth = (pos.length() / radius).clamp(0.3, 1.0);

        commands.spawn((
            Star,
            ChildOf(root),
            ParallaxLayer::new(depth),
            Mesh3d(star_mesh.clone()),
            MeshMaterial3d(star_material.clone()),
            Transform::from_translation(pos).with_scale(Vec3::splat(entity_scale(i))),
        ));
    }

    debug!("primitives::spawn_starfield — count={count}, radius={radius}, root={root:?}");
}

/// Spawn a single ambient-light entity parented to `root`.
pub fn spawn_scene_ambient_light(
    commands: &mut bevy::ecs::system::Commands,
    root: Entity,
    brightness: f32,
    color: bevy::color::LinearRgba,
) {
    use bevy::pbr::AmbientLight;
    commands.spawn((
        ChildOf(root),
        AmbientLight {
            color: bevy::color::Color::LinearRgba(color),
            brightness,
        },
    ));
    trace!("primitives::spawn_scene_ambient_light — brightness={brightness}, root={root:?}");
}

// ---------------------------------------------------------------------------
// Geometry helpers
// ---------------------------------------------------------------------------

/// Deterministic Fibonacci-sphere point at index `i` of `total`.
///
/// Uses the golden-ratio distribution for uniform coverage.
pub fn fibonacci_sphere_point(index: usize, total: usize, radius: f32) -> Vec3 {
    let golden = (1.0 + 5.0_f32.sqrt()) / 2.0;
    let i = index as f32;
    let n = total as f32;

    let theta = 2.0 * std::f32::consts::PI * i / golden;
    let phi = (1.0 - 2.0 * (i + 0.5) / n).acos();

    // Vary the radius slightly for depth.
    let r = radius * (0.6 + 0.4 * ((i * 7.13).sin() * 0.5 + 0.5));

    Vec3::new(
        r * phi.sin() * theta.cos(),
        r * phi.sin() * theta.sin(),
        r * phi.cos(),
    )
}

/// Deterministic per-entity scale in `[0.3, 1.0]` using the index.
pub fn entity_scale(index: usize) -> f32 {
    let i = index as f32;
    0.3 + 0.7 * ((i * 3.17).sin() * 0.5 + 0.5)
}
```

- [ ] **Step 2: Verify the file compiles**

Run: `cargo check -p deer-gui 2>&1 | head -20`
Expected: Errors about `primitives` not being declared in `mod.rs` yet. That's fine — next step.

---

### Task 2: Wire `primitives` into `scene/mod.rs`

**Files:**
- Modify: `apps/deer_gui/src/scene/mod.rs`

- [ ] **Step 1: Add primitives module and re-exports**

Add after `pub mod common;`:
```rust
pub mod primitives;
```

Add to the re-exports at the bottom:
```rust
pub use primitives::{
    spawn_root, spawn_starfield, spawn_scene_ambient_light,
    fibonacci_sphere_point, entity_scale, Star,
};
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check -p deer-gui 2>&1 | head -20`
Expected: May have warnings about unused imports or `Star` being defined in two places. Fix in next task.

---

### Task 3: Refactor `tet/setup.rs` to use primitives

**Files:**
- Modify: `apps/deer_gui/src/scene/tet/setup.rs`

- [ ] **Step 1: Replace `Star` with re-export from primitives**

Remove the `Star` component definition from `tet/setup.rs`. Add a re-export:
```rust
pub use crate::scene::primitives::Star;
```

- [ ] **Step 2: Replace `spawn_starfield` with primitives call**

Replace the private `spawn_starfield` function body. The function signature stays for backward compat but delegates:
```rust
fn spawn_starfield(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    root: Entity,
    emissive: bevy::color::LinearRgba,
) {
    crate::scene::primitives::spawn_starfield(
        commands, meshes, materials, root, emissive,
        STARFIELD_COUNT, STARFIELD_RADIUS,
    );
}
```

- [ ] **Step 3: Remove `pseudo_random_sphere_point` and `random_scale`**

Delete these two private functions from `tet/setup.rs`. They are now in `primitives.rs` as `fibonacci_sphere_point` and `entity_scale`.

- [ ] **Step 4: Replace `spawn_root` call**

In `spawn_tet_environment`, replace:
```rust
let root = commands.spawn((SceneRoot, Transform::default())).id();
```
with:
```rust
let root = crate::scene::primitives::spawn_root(commands);
```

- [ ] **Step 5: Update unit tests in `tet/setup.rs`**

The existing unit tests reference `pseudo_random_sphere_point` and `random_scale`. Update them to use `crate::scene::primitives::fibonacci_sphere_point` and `crate::scene::primitives::entity_scale`.

- [ ] **Step 6: Verify all existing tests pass**

Run: `cargo test -p deer-gui 2>&1`
Expected: All 139 tests pass. No behaviour change.

---

### Task 4: Write integration tests for primitives

**Files:**
- Create: `apps/deer_gui/tests/integration/scene_primitives.rs`
- Modify: `apps/deer_gui/tests/integration.rs`

- [ ] **Step 1: Add module to integration.rs**

Add to the `mod integration` block:
```rust
pub mod scene_primitives;
```

- [ ] **Step 2: Write the test file**

```rust
//! Integration tests for shared scene primitives.

use bevy::asset::AssetPlugin;
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;

use deer_gui::scene::primitives::{
    entity_scale, fibonacci_sphere_point, spawn_root, spawn_starfield, Star,
};
use deer_gui::scene::SceneRoot;
use deer_gui::scene::common::parallax::ParallaxLayer;
use deer_gui::scene::tet::setup::TetMonolith;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_primitives_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<Mesh>();
    app.init_asset::<StandardMaterial>();
    app
}

fn count_entities<T: Component>(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<Entity, With<T>>()
        .iter(app.world())
        .count()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn t_prim_01_spawn_root_creates_scene_root() {
    let mut app = build_primitives_app();
    let root = app.world_mut().commands().queue(|world: &mut World| {
        let mut commands = world.commands();
        spawn_root(&mut commands)
    });
    // Actually we need to use resource_scope pattern.
    // Simpler approach: direct world manipulation.
    let root = {
        let mut commands = app.world_mut().commands();
        let r = spawn_root(&mut commands);
        r
    };
    app.world_mut().flush();

    assert!(
        app.world().get::<SceneRoot>(root).is_some(),
        "spawn_root should create entity with SceneRoot component"
    );
}

#[test]
fn t_prim_02_spawn_starfield_count() {
    let mut app = build_primitives_app();
    let count = 50;

    app.world_mut()
        .resource_scope(|world: &mut World, mut meshes: Mut<Assets<Mesh>>| {
            world.resource_scope(
                |world: &mut World, mut materials: Mut<Assets<StandardMaterial>>| {
                    let mut commands = world.commands();
                    let root = spawn_root(&mut commands);
                    spawn_starfield(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        root,
                        bevy::color::LinearRgba::new(2.0, 2.0, 2.0, 1.0),
                        count,
                        800.0,
                    );
                },
            );
        });
    app.world_mut().flush();

    let star_count = count_entities::<Star>(&mut app);
    assert_eq!(star_count, count, "should spawn exactly {count} Star entities");

    // Verify parallax layers.
    let parallax_count = count_entities::<ParallaxLayer>(&mut app);
    assert_eq!(parallax_count, count, "every star should have ParallaxLayer");
}

#[test]
fn t_prim_03_fibonacci_sphere_within_radius() {
    let radius = 800.0;
    for i in 0..200 {
        let p = fibonacci_sphere_point(i, 200, radius);
        assert!(
            p.length() <= radius + 1e-3,
            "point {i} at distance {} exceeds radius {radius}",
            p.length()
        );
    }
}

#[test]
fn t_prim_04_entity_scale_in_range() {
    for i in 0..200 {
        let s = entity_scale(i);
        assert!(
            s >= 0.29 && s <= 1.01,
            "entity_scale({i}) = {s} out of range [0.3, 1.0]"
        );
    }
}

#[test]
fn t_prim_05_tet_unchanged_after_refactor() {
    use deer_gui::scene::{SceneConfig, SceneManager, SceneRoot};
    use deer_gui::scene::tet::config::TetSceneConfig;

    let mut app = build_primitives_app();

    let mut mgr = SceneManager::new();
    mgr.register(Box::new(TetSceneConfig));
    app.insert_resource(mgr);

    // Activate TET.
    app.world_mut()
        .resource_scope(|world: &mut World, mut mgr: Mut<SceneManager>| {
            world.resource_scope(|world: &mut World, mut meshes: Mut<Assets<Mesh>>| {
                world.resource_scope(
                    |world: &mut World, mut materials: Mut<Assets<StandardMaterial>>| {
                        let mut commands = world.commands();
                        mgr.activate("TET", &mut commands, &mut meshes, &mut materials, None, None);
                    },
                );
            });
        });
    app.world_mut().flush();

    let root_count = count_entities::<SceneRoot>(&mut app);
    assert_eq!(root_count, 1, "TET should have one SceneRoot");

    let star_count = count_entities::<Star>(&mut app);
    assert!(star_count > 0, "TET should have stars after refactor");

    let monolith_count = count_entities::<TetMonolith>(&mut app);
    assert_eq!(monolith_count, 1, "TET should have one TetMonolith");
}
```

- [ ] **Step 3: Run all tests**

Run: `cargo test -p deer-gui 2>&1`
Expected: All tests pass including the 5 new primitives tests.

- [ ] **Step 4: Commit**

```bash
git add apps/deer_gui/src/scene/primitives.rs \
       apps/deer_gui/src/scene/mod.rs \
       apps/deer_gui/src/scene/tet/setup.rs \
       apps/deer_gui/tests/integration/scene_primitives.rs \
       apps/deer_gui/tests/integration.rs
git commit -m "feat(scene): extract shared primitives from TET, refactor TET to use them"
```
