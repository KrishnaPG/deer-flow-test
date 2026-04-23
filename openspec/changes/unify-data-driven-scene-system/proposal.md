## Why

The scene system has a "split-brain" architecture: `.scene.ron` files define scenes declaratively but the runtime ignores them. `TetSceneConfig` hardcodes spawning, `DescriptorSceneConfig` logs generators but never invokes them, and `WaterPlugin`/`FoliagePlugin`/`NPCPlugin` spawn globally regardless of scene. All scenes look identical because the data-driven half is never executed. This prevents switching scenes cleanly and violates the project's stated goal of a hybrid medieval/RTS experience.

Additionally, the medieval scene currently has:
- **Zero 3D models** in the repository — all `models/` directories are empty
- **Placeholder terrain material** — green fallback instead of real PBR textures we already downloaded
- **No real rocks, bushes, or ground cover** — only invisible marker entities
- **Sci-fi contamination** — starfield, glow effects, and monolith systems run in Update for ALL scenes including medieval
- **No atmospheric effects** — no fog, no volumetric lighting, no proper autumn color grading

The user explicitly demands **photo-realistic quality** matching a PolyHaven reference (autumn river valley with stone church, stream, rocks, deciduous trees). They have rejected procedural geometry (cones, cylinders) as unacceptable. The path forward is:
1. First complete the data-driven scene architecture
2. Then build the medieval scene without trees using real assets (downloaded rocks, PBR textures)
3. Finally generate proper tree models with Blender and integrate them

## What Changes

### Phase 1: Unify Data-Driven Scene System
- **BREAKING**: Remove `TetSceneConfig` and load TET from `tet.scene.ron` like all other scenes
- **BREAKING**: Convert `WaterPlugin`, `FoliagePlugin`, `NPCPlugin` from global `Startup` spawners to generator functions registered in `GeneratorRegistry`
- **Fix**: `SceneManager::deactivate()` uses recursive despawn to prevent entity leaks (already implemented, verify)
- **Fix**: `DescriptorSceneConfig::spawn_environment()` invokes `GeneratorRegistry` for each generator in the descriptor (already implemented, verify)
- **Fix**: Make sci-fi Update systems (`tet_glow_system`, `data_trail_system`, `barge_system`, `traveller_system`, `cloud_system`, `drop_pod_system`) scene-conditional — only run when their respective scenes are active
- **Update**: Add water/foliage/npc generators to `medieval_open.scene.ron`
- **Remove**: Hardcoded `medieval_open` pre-registration in `ScenePlugin::build()`
- **Update**: `main.rs` — remove global `WaterPlugin` and `FoliagePlugin` adds, keep scene-driven only

### Phase 2: Build Medieval Scene (Without Trees)
- **Fix terrain material**: Load real PBR textures (Grass003 color, normal, roughness, AO) instead of green fallback
- **Download real rocks**: Yughues CC0 photogrammetry rocks from OpenGameArt (3 variants, game-ready LODs)
- **Create `gen_rocks` generator**: Scatter downloaded rock models along stream beds and hillsides with terrain height sampling
- **Add atmospheric fog**: Distance fog with warm autumn color, exponential falloff
- **Fix lighting**: Warm sun angle (golden hour), proper ambient light, sky dome with existing HDRI
- **Camera position**: Set above terrain (Y=50+) looking down valley, 45° pitch
- **Build and verify**: Run medieval scene, confirm terrain, water, rocks, lighting look correct

### Phase 3: Generate Trees with Blender
- **Install Blender**: `brew install --cask blender`
- **Create Python generation script**: Use Blender's Sapling Tree Gen addon + Geometry Nodes to generate:
  - Oak (broad deciduous, autumn orange/green)
  - Birch (white bark, sparse yellow leaves)
  - Pine (coniferous, dark green)
- **Run headless**: Export each tree as GLB with proper materials and LODs
- **Integrate vegetation generator**: Update `gen_vegetation` to load generated GLB models
- **Scene integration**: Add tree placement to `medieval_open.scene.ron`
- **Final validation**: Full scene run, verify against reference image

## Capabilities

### New Capabilities
- `scene-generator-system`: Data-driven generator invocation from `.scene.ron` descriptors
- `scene-cleanup-system`: Recursive entity despawn for clean scene teardown
- `medieval-terrain-system`: PBR heightmap terrain with texture splatting
- `rock-scatter-system`: Procedural rock placement with real photogrammetry models
- `atmospheric-rendering`: Fog, volumetric lighting, HDRI sky dome

### Modified Capabilities
- `vegetation-system`: Now loads real GLB tree models instead of invisible markers
- `water-system`: Now scene-driven instead of global
- `npc-system`: Now scene-driven instead of global
- `scene-switching`: Sci-fi systems only run in sci-fi scenes

## Impact

### Phase 1 Files
- `apps/deer_gui/src/scene/manager.rs`: Verify recursive despawn works
- `apps/deer_gui/src/scene/descriptor_config.rs`: Verify generator invocation
- `apps/deer_gui/src/scene/plugin.rs`: Remove TetSceneConfig registration, make Update systems conditional
- `apps/deer_gui/src/scene/tet/config.rs`: **DELETE** — no longer needed
- `apps/deer_gui/src/world/water.rs`: Extract spawn logic into `gen_water`, remove Startup system
- `apps/deer_gui/src/world/foliage.rs`: Extract spawn logic into `gen_foliage`, remove Startup system
- `apps/deer_gui/src/world/npc.rs`: Extract spawn logic into `gen_npcs`, remove Startup system
- `apps/deer_gui/src/main.rs`: Remove global WaterPlugin/FoliagePlugin adds
- `apps/deer_gui/src/scene/generators/registry.rs`: Register new generators
- `.scene.ron` files: Add generator specs for medieval scene

### Phase 2 Files
- `apps/deer_gui/src/scene/generators/terrain.rs`: Use real PBR textures, add normal/roughness maps
- `apps/deer_gui/src/scene/generators/rocks.rs`: **NEW** — photogrammetry rock scatter generator
- `apps/deer_gui/src/scene/generators/atmosphere.rs`: **NEW** — fog and lighting generator
- `apps/deer_gui/assets/models/rocks/`: Downloaded Yughues rock models
- `apps/deer_gui/assets/scenes/medieval_open.scene.ron`: Add rock and atmosphere generators
- `apps/deer_gui/src/camera/components.rs`: Adjust default camera position

### Phase 3 Files
- `scripts/generate_trees.py`: **NEW** — Blender Python script for tree generation
- `apps/deer_gui/assets/models/foliage/`: Generated tree GLB files
- `apps/deer_gui/src/scene/generators/vegetation.rs`: Load GLB models, panic if missing
- `apps/deer_gui/assets/scenes/medieval_open.scene.ron`: Add vegetation generator with real models
