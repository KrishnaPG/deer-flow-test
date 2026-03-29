# PR A — Shared Scene Primitives + TET Refactor

**Parent:** [00-index.md](./00-index.md)
**Status:** Approved

---

## Problem

`spawn_tet_environment` contains private helpers (`spawn_starfield`,
`pseudo_random_sphere_point`, `random_scale`) that will be re-used by
Precursors and Descent. Duplicating them across scenes creates diverging
implementations.

## New Module: `src/scene/primitives.rs`

A single, scene-independent file exposing reusable spawn helpers. All
functions accept minimal arguments and return nothing (they side-effect
`Commands`/`Assets`). Each function is < 50 LOC.

### Public API

```rust
/// Spawn a SceneRoot entity and return its Entity id.
pub fn spawn_root(commands: &mut Commands) -> Entity

/// Spawn `count` stars distributed on a Fibonacci sphere of `radius`.
/// Emissive color from `emissive`. Each star gets `ParallaxLayer`.
/// All entities are `ChildOf(root)`.
pub fn spawn_starfield(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    root: Entity,
    emissive: LinearRgba,
    count: usize,
    radius: f32,
)

/// Spawn a single ambient-light entity parented to root.
pub fn spawn_scene_ambient_light(
    commands: &mut Commands,
    root: Entity,
    brightness: f32,
    color: LinearRgba,
)

/// Deterministic Fibonacci-sphere point at index `i` of `total`.
pub fn fibonacci_sphere_point(index: usize, total: usize, radius: f32) -> Vec3

/// Deterministic per-entity scale in [0.3, 1.0] using the index.
pub fn entity_scale(index: usize) -> f32
```

### What Stays in `tet/setup.rs`

`spawn_monolith`, `spawn_data_trails`, and `spiral_position` remain
TET-specific — they use TET-specific constants and components
(`TetMonolith`, `DataTrail`).

### What Moves

| From (`tet/setup.rs`)         | To (`primitives.rs`)    |
| ----------------------------- | ----------------------- |
| `pseudo_random_sphere_point`  | `fibonacci_sphere_point` (renamed) |
| `random_scale`                | `entity_scale` (renamed)           |
| `Star` component              | Moved; re-exported from `tet/setup.rs` for backward compat |
| Private `spawn_starfield`     | Replaced by `primitives::spawn_starfield` call             |

### Export

`src/scene/mod.rs` adds:
```rust
pub mod primitives;
pub use primitives::{spawn_root, spawn_starfield, spawn_scene_ambient_light,
                     fibonacci_sphere_point, entity_scale};
```

## TET Refactor

`tet/setup.rs::spawn_tet_environment` is updated to:
1. Call `primitives::spawn_root` instead of inline `commands.spawn((SceneRoot, …))`.
2. Call `primitives::spawn_starfield` instead of the private version.
3. Remove redundant private `pseudo_random_sphere_point` and `random_scale`.

Behaviour is identical. All existing TET tests remain green.

## Tests

New file: `tests/integration/scene_primitives.rs`

| Test                                       | Verifies                                                    |
| ------------------------------------------ | ----------------------------------------------------------- |
| `t_prim_01_spawn_root_creates_scene_root`  | `spawn_root` returns entity with `SceneRoot` component      |
| `t_prim_02_spawn_starfield_count`          | `spawn_starfield(…, count=50)` produces 50 `Star` entities  |
| `t_prim_03_fibonacci_sphere_within_radius` | 200 points all within `radius + epsilon`                     |
| `t_prim_04_entity_scale_in_range`          | 200 indices all in `[0.29, 1.01]`                            |
| `t_prim_05_tet_unchanged_after_refactor`   | Activating TET still produces expected Star count + Monolith |
