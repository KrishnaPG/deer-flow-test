## Why

The scene system has a "split-brain" architecture: `.scene.ron` files define scenes declaratively but the runtime ignores them. `TetSceneConfig` hardcodes spawning, `DescriptorSceneConfig` logs generators but never invokes them, and `WaterPlugin`/`FoliagePlugin`/`NPCPlugin` spawn globally regardless of scene. All scenes look identical because the data-driven half is never executed. This prevents switching scenes cleanly and violates the project's stated goal of a hybrid medieval/RTS experience.

## What Changes

- **BREAKING**: Remove `TetSceneConfig` and load TET from `tet.scene.ron` like all other scenes
- **BREAKING**: Convert `WaterPlugin`, `FoliagePlugin`, `NPCPlugin` from global `Startup` spawners to generator functions registered in `GeneratorRegistry`
- **Fix**: Change `SceneManager::deactivate()` from `despawn()` to `despawn_recursive()` to prevent entity leaks
- **Fix**: Make `DescriptorSceneConfig::spawn_environment()` invoke `GeneratorRegistry` for each generator in the descriptor
- **Update**: Add water/foliage/npc generators to `medieval_open.scene.ron`
- **Remove**: Hardcoded `medieval_open` pre-registration in `ScenePlugin::build()`
- **Update**: Move `setup_landscape` sun/light into a generator or universal startup system

## Capabilities

### New Capabilities
- `scene-generator-system`: Data-driven generator invocation from `.scene.ron` descriptors
- `scene-cleanup-system`: Recursive entity despawn for clean scene teardown

### Modified Capabilities
- (none - this is an architecture fix, not a new feature)

## Impact

- `apps/deer_gui/src/scene/manager.rs`: Fix `deactivate()` cleanup
- `apps/deer_gui/src/scene/descriptor_config.rs`: Invoke generators
- `apps/deer_gui/src/scene/plugin.rs`: Remove hardcoded registrations
- `apps/deer_gui/src/scene/tet/`: Remove `TetSceneConfig`, rely on RON
- `apps/deer_gui/src/world/water.rs`: Convert startup spawn to generator
- `apps/deer_gui/src/world/foliage.rs`: Convert startup spawn to generator
- `apps/deer_gui/src/world/npc.rs`: Convert startup spawn to generator
- `apps/deer_gui/src/main.rs`: Remove or scene-gate `setup_landscape`
- `.scene.ron` files: Add generator specs for medieval scene
