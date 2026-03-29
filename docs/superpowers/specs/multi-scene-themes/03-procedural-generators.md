# PR C — Procedural Generator Factories + GPU Instancing

**Parent:** [00-index.md](./00-index.md)
**Status:** Approved
**Depends on:** PR A, PR B

---

## Problem

Each scene needs procedurally generated content (starfields, particle streams,
cloud layers). Without a factory system, each scene duplicates spawn logic
with slight variations. The descriptor system (PR B) references generators by
name — this PR provides the registry that resolves those names.

## Design

### Generator Registry: `src/scene/generators/registry.rs` (~100 LOC)

```rust
/// Function signature for a procedural generator factory.
pub type GeneratorFn = fn(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    root: Entity,
    params: &GeneratorParams,
);

/// Registry mapping generator type names to factory functions.
#[derive(Resource)]
pub struct GeneratorRegistry {
    factories: HashMap<String, GeneratorFn>,
}

impl GeneratorRegistry {
    pub fn new() -> Self { … }
    pub fn register(&mut self, name: &str, factory: GeneratorFn) { … }
    pub fn get(&self, name: &str) -> Option<&GeneratorFn> { … }

    /// Register all built-in generators.
    pub fn with_builtins() -> Self { … }
}
```

### Built-in Generators

Each generator is a standalone function in its own file under
`src/scene/generators/`. All use the shared primitives from PR A where
applicable.

```
src/scene/generators/
├── mod.rs               — pub re-exports + GeneratorRegistry
├── registry.rs          — GeneratorRegistry resource
├── starfield.rs         — gen_starfield() — Fibonacci sphere placement
├── spiral_trails.rs     — gen_spiral_trails() — parametric spiral paths
├── river_barges.rs      — gen_river_barges() — horizontal parametric flow
├── path_travellers.rs   — gen_path_travellers() — diagonal path flow
├── cloud_layer.rs       — gen_cloud_layer() — vertical rising particles
├── drop_pods.rs         — gen_drop_pods() — vertical falling particles
├── static_glow.rs       — gen_static_glow_cluster() — fixed emissive cluster
└── gltf_subscene.rs     — gen_gltf_subscene() — load a glTF sub-scene
```

### Generator Contract

Every generator function:
1. Reads its typed params from `GeneratorParams` enum variant.
2. Creates mesh + material assets (or loads glTF handle).
3. Spawns entities as `ChildOf(root)`.
4. Returns nothing (side-effects only).
5. Logs via `trace!`/`debug!` for all operations.
6. Is < 50 LOC.

### GPU Instancing

For high entity counts (starfields, cloud layers, particle fields), generators
use Bevy's native instanced rendering:

- Shared `Handle<Mesh>` and `Handle<StandardMaterial>` across all entities of
  the same type (Bevy batches these automatically).
- For truly massive counts (10k+), use a custom `Material` with WGSL shader
  that reads per-instance data from a storage buffer. This is deferred until
  needed — the current approach (shared handles) already batches well for
  counts under 5k.
- `bevy_hanabi` GPU compute particles for visual effects that need continuous
  emission (agent particles, data streams). These are already in Cargo.toml.

### bevy_hanabi Integration

Generators that produce continuous particle effects (not just static meshes)
create `bevy_hanabi::EffectAsset` instances. The existing `particles.rs`
patterns are reused:

```rust
// In gen_cloud_layer (for example):
// Option A: Static mesh entities (simple, testable)
// Option B: bevy_hanabi particle effect (visual quality, GPU-driven)
// Default to Option A for testability; provide Option B as an
// enhancement layer that composites on top.
```

## Integration with Descriptor System

`DescriptorSceneConfig::spawn_environment` iterates its descriptor's
`generators` list and calls `GeneratorRegistry::get(name)` for each:

```rust
fn spawn_environment(&self, commands, meshes, materials, theme) -> Entity {
    let root = primitives::spawn_root(commands);
    let registry = /* accessed via world */ ;
    for spec in &self.descriptor.generators {
        if let Some(factory) = registry.get(&spec.generator) {
            factory(commands, meshes, materials, root, &spec.params);
        } else {
            warn!("Unknown generator: {}", spec.generator);
        }
    }
    root
}
```

## Tests

New file: `tests/integration/scene_generators.rs`

| Test                                    | Verifies                                                    |
| --------------------------------------- | ----------------------------------------------------------- |
| `t_gen_01_registry_builtins_registered` | `with_builtins()` has all expected generator names           |
| `t_gen_02_starfield_spawns_entities`    | `gen_starfield` produces correct entity count                |
| `t_gen_03_spiral_trails_spawns`         | `gen_spiral_trails` produces correct entity count            |
| `t_gen_04_cloud_layer_spawns`           | `gen_cloud_layer` produces correct entity count              |
| `t_gen_05_unknown_generator_returns_none` | `registry.get("nonexistent")` returns `None`              |
| `t_gen_06_all_entities_child_of_root`   | All spawned entities have `ChildOf(root)`                    |
