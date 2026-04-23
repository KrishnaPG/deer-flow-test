## Context

The Deer GUI application uses a hybrid scene system:
- `.scene.ron` files declare scene content (generators, themes, audio)
- `DescriptorSceneConfig` implements `SceneConfig` but never invokes generators
- `TetSceneConfig` hardcodes spawning and ignores `tet.scene.ron`
- Water, foliage, and NPC plugins spawn globally via independent `Startup` systems

Current behavior: all scenes look identical (rotating sphere + stars + ocean) because the data-driven half is never executed, and the hardcoded half spawns unconditionally.

## Goals / Non-Goals

**Goals:**
- All scenes load from `.scene.ron` files (no hardcoded scene configs)
- GeneratorRegistry actually invokes generators listed in descriptors
- Scene teardown recursively despawns all entities (no leaks)
- Water/foliage/NPC spawn per-scene, not globally
- Visual distinctness: TET, Descent, Precursors, and Medieval scenes look different

**Non-Goals:**
- Runtime scene switching UI (CLI-only for now)
- Async/streaming scene loading
- Save/load of scene state
- Network multiplayer scene sync

## Decisions

### 1. Convert TET to Data-Driven
**Decision**: Remove `TetSceneConfig` and load TET from `tet.scene.ron` via `DescriptorSceneConfig`.
**Rationale**: Consistent with all other scenes. `tet.scene.ron` already exists and defines starfield + spiral_trails generators.
**Alternative**: Keep TetSceneConfig as special case. Rejected - violates DRY and consistency.

### 2. Global Spawners to Generators
**Decision**: Move water/foliage/NPC spawn logic from plugin `Startup` systems into generator functions registered in `GeneratorRegistry`.
**Rationale**: Generators are the data-driven spawning pipeline. Everything that appears in a scene should be declared in its `.scene.ron`.
**Alternative**: Add scene-gating logic to plugins. Rejected - scatters scene knowledge across modules.

### 3. Recursive Despawn
**Decision**: Change `commands.entity(root).despawn()` to `commands.entity(root).despawn_recursive()` in `SceneManager::deactivate()`.
**Rationale**: Bevy's `despawn()` only removes the entity; children become orphaned. `despawn_recursive()` cleans the entire hierarchy.
**Alternative**: Manually track and despawn children. Rejected - error-prone, Bevy provides the tool.

### 4. Universal Lighting
**Decision**: Keep `setup_landscape` (sun + ambient light) as a global startup system.
**Rationale**: All scenes need lighting. Moving it per-scene duplicates effort. If a scene needs custom lighting, it can override via generator.
**Alternative**: Move to generator. Rejected - YAGNI, all current scenes use same lighting.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Breaking existing TET behavior | `tet.scene.ron` already defines identical content; verify visually |
| Generator function signatures mismatch | Use same signature as existing generators; compiler catches errors |
| NPC/Water/Foliage state loss on scene switch | These are scene entities; state should not persist across scenes |
| Performance: generators run synchronously | Current scale is small; async loading is non-goal |

## Migration Plan

1. Fix `despawn_recursive()` (1 line change, immediate bugfix)
2. Invoke generators in `DescriptorSceneConfig` (add registry lookup + call)
3. Verify TET loads from RON (remove TetSceneConfig, test)
4. Convert world plugins to generators (refactor spawn code into generator fns)
5. Update medieval_open.scene.ron to include new generators
6. Test all four scenes for visual distinctness

## Open Questions

1. Should NPC behavior trees persist across scene reloads? (Out of scope for now)
2. Do we need a "loading" state while generators run? (Non-goal for this change)
3. Should water be a separate plugin or fully generator-based? (Fully generator-based for consistency)
