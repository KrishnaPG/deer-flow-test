## Context

The Deer GUI application uses a hybrid scene system:
- `.scene.ron` files declare scene content (generators, themes, audio)
- `DescriptorSceneConfig` implements `SceneConfig` but never invokes generators
- `TetSceneConfig` hardcodes spawning and ignores `tet.scene.ron`
- Water, foliage, and NPC plugins spawn globally via independent `Startup` systems
- Sci-fi per-frame systems (glow, trails, barges, clouds, drop pods) run unconditionally in Update

Current behavior: all scenes look identical (rotating sphere + stars + ocean) because the data-driven half is never executed, and the hardcoded half spawns unconditionally. The medieval scene additionally has no real models, placeholder materials, and no atmospheric effects.

## Goals / Non-Goals

**Goals:**
- All scenes load from `.scene.ron` files (no hardcoded scene configs)
- GeneratorRegistry actually invokes generators listed in descriptors
- Scene teardown recursively despawns all entities (no leaks)
- Water/foliage/NPC spawn per-scene, not globally
- Sci-fi Update systems only run in sci-fi scenes
- Visual distinctness: TET, Descent, Precursors, and Medieval scenes look different
- Medieval scene uses real PBR textures for terrain
- Medieval scene has real photogrammetry rocks scattered procedurally
- Medieval scene has atmospheric fog and proper lighting
- Medieval scene WITHOUT trees is fully playable and visually verified before tree generation begins

**Non-Goals:**
- Runtime scene switching UI (CLI-only for now)
- Async/streaming scene loading
- Save/load of scene state
- Network multiplayer scene sync
- Procedural geometry fallbacks (cones, cylinders) — user explicitly rejected these

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
**Decision**: Change `commands.entity(root).despawn()` to manual recursive despawn in `SceneManager::deactivate()`.
**Rationale**: Bevy 0.18 doesn't have `despawn_recursive()` on `EntityCommands`. We implement a `despawn_recursive` helper that walks the hierarchy.
**Alternative**: Use Bevy's built-in recursive despawn if available in newer versions. Rejected - we're on 0.18.

### 4. Universal Lighting
**Decision**: Keep `setup_landscape` (sun + ambient light) as a global startup system.
**Rationale**: All scenes need lighting. Moving it per-scene duplicates effort. If a scene needs custom lighting, it can override via generator.
**Alternative**: Move to generator. Rejected - YAGNI, all current scenes use same lighting.

### 5. Scene-Conditional Sci-Fi Systems
**Decision**: Add a resource `ActiveSceneTag` that stores the current scene name. Sci-fi systems check this tag and early-return if not in their scene.
**Rationale**: Minimal change to existing systems. No need to add/remove systems dynamically.
**Alternative**: Dynamically add/remove systems on scene switch. Rejected - complex, error-prone in Bevy 0.18.

### 6. No Procedural Fallbacks
**Decision**: If a referenced model file doesn't exist, panic with a clear error message. Never silently fall back to primitive geometry.
**Rationale**: User explicitly demanded photo-realistic quality and rejected cones/cylinders.
**Alternative**: Procedural fallback with primitives. Rejected - user will not accept this.

### 7. Build Without Trees First
**Decision**: Complete the full medieval scene (terrain, water, rocks, lighting, fog) and verify it looks good BEFORE generating any trees.
**Rationale**: User wants to see the base scene quality before investing time in tree generation.
**Alternative**: Generate everything at once. Rejected - user wants incremental validation.

### 8. Tree Generation via Blender
**Decision**: Use Blender's Sapling Tree Gen addon (free, built-in) via Python API to generate proper deciduous trees, export as GLB.
**Rationale**: No free CC0 realistic tree models exist online. Blender is the only free path to real geometry.
**Alternative**: Purchase SpeedTree ($299). Rejected - user prefers free solution.
**Alternative**: Use photogrammetry scans (Sketchfab). Rejected - 1M+ polygons, unusable in real-time.

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| Breaking existing TET behavior | `tet.scene.ron` already defines identical content; verify visually |
| Generator function signatures mismatch | Use same signature as existing generators; compiler catches errors |
| NPC/Water/Foliage state loss on scene switch | These are scene entities; state should not persist across scenes |
| Performance: generators run synchronously | Current scale is small; async loading is non-goal |
| Blender not installed or script fails | Check Blender availability first, provide clear error |
| Tree generation takes too long | Sapling is fast (seconds per tree), batch generate all variants |
| GLB export loses materials | Use Blender's glTF exporter with proper material export settings |

## Migration Plan

### Phase 1: Unify Scene System
1. Verify `despawn_recursive()` works correctly (already implemented)
2. Verify generators invoke in `DescriptorSceneConfig` (already implemented)
3. Remove `TetSceneConfig`, load TET from RON
4. Extract water spawn to `gen_water` generator
5. Extract foliage spawn to `gen_foliage` generator
6. Extract NPC spawn to `gen_npcs` generator
7. Make sci-fi Update systems scene-conditional
8. Update `medieval_open.scene.ron` with new generators
9. Fix `main.rs` to remove global plugins
10. Test all four scenes for visual distinctness

### Phase 2: Medieval Base Scene (No Trees)
1. Fix terrain material to load real PBR textures (color, normal, roughness, AO)
2. Download Yughues photogrammetry rocks from OpenGameArt
3. Create `gen_rocks` generator with terrain height sampling
4. Add atmospheric fog generator
5. Adjust camera position for terrain viewing
6. Build and run medieval scene
7. Verify visual quality against reference

### Phase 3: Tree Generation
1. Install Blender (`brew install --cask blender`)
2. Write Python script using Sapling addon
3. Generate oak, birch, pine variants
4. Export as GLB with autumn materials
5. Update vegetation generator to load GLB models
6. Add trees to medieval scene
7. Final validation and quality check

## Open Questions

1. Should NPC behavior trees persist across scene reloads? (Out of scope for now)
2. Do we need a "loading" state while generators run? (Non-goal for this change)
3. Should water be a separate plugin or fully generator-based? (Fully generator-based for consistency)
4. What density of rocks is appropriate for the stream bed? (Test and iterate visually)
5. Should trees have wind animation? (Phase 3, after base scene works)
