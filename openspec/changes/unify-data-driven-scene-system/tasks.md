## Phase 1: Fix Scene Cleanup & Verify

- [x] 1.1 Change `SceneManager::deactivate()` to use manual recursive despawn (already done)
- [x] 1.2 Verify `DescriptorSceneConfig::spawn_environment()` invokes generators from registry (already done)
- [ ] 1.3 Test scene switch: tet → medieval → tet, verify no orphaned entities via logging

## Phase 2: Convert TET to Data-Driven

- [ ] 2.1 Delete `apps/deer_gui/src/scene/tet/config.rs` (remove `TetSceneConfig` struct)
- [ ] 2.2 Remove `TetSceneConfig` import and registration from `ScenePlugin::build()`
- [ ] 2.3 Verify `tet.scene.ron` has correct generators: starfield + spiral_trails
- [ ] 2.4 Test `--scene tet`: verify starfield + monolith + trails render correctly

## Phase 3: Make Sci-Fi Systems Scene-Conditional

- [ ] 3.1 Create `ActiveSceneTag` resource in `scene/manager.rs` or `scene/common/mod.rs`
- [ ] 3.2 Set `ActiveSceneTag` in `SceneManager::activate()` and clear in `deactivate()`
- [ ] 3.3 Modify `tet_glow_system` to check `ActiveSceneTag` and early-return if not "TET"
- [ ] 3.4 Modify `data_trail_system` to check `ActiveSceneTag` and early-return if not "TET"
- [ ] 3.5 Modify `barge_system` to check `ActiveSceneTag` and early-return if not "descent"
- [ ] 3.6 Modify `traveller_system` to check `ActiveSceneTag` and early-return if not "descent"
- [ ] 3.7 Modify `cloud_system` to check `ActiveSceneTag` and early-return if not "descent"/"precursors"
- [ ] 3.8 Modify `drop_pod_system` to check `ActiveSceneTag` and early-return if not "descent"
- [ ] 3.9 Test `--scene medieval_open`: verify NO sci-fi elements (no stars, no glow, no trails)

## Phase 4: Convert World Plugins to Generators

### 4A: Water
- [ ] 4A.1 Create `apps/deer_gui/src/scene/generators/water.rs` with `gen_water()` function
- [ ] 4A.2 Extract water body spawn logic from `WaterPlugin::build()` / `setup_water_system`
- [ ] 4A.3 Register `gen_water` in `GeneratorRegistry::with_builtins()`
- [ ] 4A.4 Remove `setup_water_system` from `WaterPlugin::build()` Startup systems
- [ ] 4A.5 Keep Update systems in WaterPlugin (wave animation, swimming, splash) — these are per-frame, not spawn

### 4B: Foliage
- [ ] 4B.1 Create `apps/deer_gui/src/scene/generators/foliage.rs` with `gen_foliage()` function
- [ ] 4B.2 Extract foliage root spawn logic from `FoliagePlugin::build()` / `setup_foliage_system`
- [ ] 4B.3 Register `gen_foliage` in `GeneratorRegistry::with_builtins()`
- [ ] 4B.4 Remove `setup_foliage_system` from `FoliagePlugin::build()` Startup systems
- [ ] 4B.5 Keep `update_wind_system` in FoliagePlugin Update

### 4C: NPCs
- [ ] 4C.1 Create `apps/deer_gui/src/scene/generators/npcs.rs` with `gen_npcs()` function
- [ ] 4C.2 Extract NPC spawn logic from `NpcPlugin::build()` / `setup_npc_system`
- [ ] 4C.3 Register `gen_npcs` in `GeneratorRegistry::with_builtins()`
- [ ] 4C.4 Remove `setup_npc_system` from `NpcPlugin::build()` Startup systems
- [ ] 4C.5 Keep `npc_animation_system` in NpcPlugin Update

## Phase 5: Update Scene Descriptors and Main

- [ ] 5.1 Add `water` generator to `medieval_open.scene.ron` with lake/river params
- [ ] 5.2 Add `foliage` generator to `medieval_open.scene.ron` (will spawn root only until trees exist)
- [ ] 5.3 Add `npcs` generator to `medieval_open.scene.ron`
- [ ] 5.4 Remove hardcoded `medieval_open` pre-registration from `ScenePlugin::build()`
- [ ] 5.5 Update `main.rs`: remove `.add_plugins(FoliagePlugin::default())`
- [ ] 5.6 Update `main.rs`: remove `.add_plugins(WaterPlugin { ... })`
- [ ] 5.7 Keep `WorldPlugin` in main.rs (it may have other non-spawn systems)
- [ ] 5.8 Verify `setup_landscape` (sun) stays as global Startup

## Phase 6: Fix Terrain PBR Materials

- [ ] 6.1 Update `create_terrain_material()` in `terrain.rs` to load:
  - Grass003 color map as base color texture
  - Grass003 normal map
  - Grass003 roughness map
  - Grass003 AO map
- [ ] 6.2 Use `AssetServer` to load textures asynchronously or create materials with image handles
- [ ] 6.3 Remove green fallback material entirely — panic if textures not found
- [ ] 6.4 Test terrain renders with realistic grass texture instead of flat green

## Phase 7: Download and Integrate Photogrammetry Rocks

- [ ] 7.1 Download Yughues photogrammetry rocks from OpenGameArt (3 variants)
- [ ] 7.2 Convert/extract to GLB format if needed
- [ ] 7.3 Place in `apps/deer_gui/assets/models/rocks/`
- [ ] 7.4 Create `gen_rocks` generator in `apps/deer_gui/src/scene/generators/rocks.rs`
- [ ] 7.5 Implement procedural scatter with terrain height sampling
- [ ] 7.6 Register in `GeneratorRegistry::with_builtins()`
- [ ] 7.7 Add `rocks` generator to `medieval_open.scene.ron`
- [ ] 7.8 Test rocks render correctly in stream bed and hillsides

## Phase 8: Atmospheric Effects

- [ ] 8.1 Create `gen_atmosphere` generator in `apps/deer_gui/src/scene/generators/atmosphere.rs`
- [ ] 8.2 Add exponential distance fog with warm autumn color
- [ ] 8.3 Adjust sun direction and color for golden hour lighting
- [ ] 8.4 Add sky dome using existing HDRI (`sky/meadow.hdr` or `sky/kloofendal_48d_partly_cloudy.hdr`)
- [ ] 8.5 Register `gen_atmosphere` in `GeneratorRegistry::with_builtins()`
- [ ] 8.6 Add `atmosphere` generator to `medieval_open.scene.ron`
- [ ] 8.7 Test fog and lighting create depth and atmosphere

## Phase 9: Camera and Validation (No Trees)

- [ ] 9.1 Update camera default position to Y=50, looking at terrain center
- [ ] 9.2 Run `cargo check -p deer-gui` and fix all compiler errors
- [ ] 9.3 Test `--scene medieval_open`: verify terrain, water, rocks, fog, lighting
- [ ] 9.4 Test `--scene tet`: verify starfield + trails, NO water/foliage/npc/rocks
- [ ] 9.5 Test `--scene descent`: verify clouds + drop pods, NO medieval elements
- [ ] 9.6 Test `--scene precursors`: verify barges + travellers, NO medieval elements
- [ ] 9.7 Test scene switching: tet → medieval → descent → precursors → tet
- [ ] 9.8 Screenshot medieval scene, compare to reference image
- [ ] 9.9 Get user approval before proceeding to trees

## Phase 10: Generate Trees with Blender

- [ ] 10.1 Install Blender: `brew install --cask blender`
- [ ] 10.2 Create `scripts/generate_trees.py` using Sapling Tree Gen addon
- [ ] 10.3 Generate oak tree: broad canopy, autumn orange/green leaves
- [ ] 10.4 Generate birch tree: white bark, sparse yellow leaves
- [ ] 10.5 Generate pine tree: coniferous, dark green needles
- [ ] 10.6 Export each as GLB to `apps/deer_gui/assets/models/foliage/`
- [ ] 10.7 Verify GLB files load correctly in Bevy

## Phase 11: Integrate Trees into Medieval Scene

- [ ] 11.1 Update `gen_vegetation` to use GLB model paths for oak, birch, pine
- [ ] 11.2 Add `SceneRoot` or `Mesh3d` components to vegetation instances (load GLB via AssetServer)
- [ ] 11.3 Panic if tree model files don't exist — no silent fallback
- [ ] 11.4 Update `medieval_open.scene.ron` vegetation generator params for autumn biome
- [ ] 11.5 Adjust tree density and scale for realistic forest edge look
- [ ] 11.6 Test full medieval scene with trees

## Phase 12: Final Polish and Validation

- [ ] 12.1 Screenshot final scene, compare to reference
- [ ] 12.2 Performance check: verify 60fps with all elements
- [ ] 12.3 Test all 4 scenes one final time
- [ ] 12.4 Test scene switching with trees (no leaks)
- [ ] 12.5 Remove any debug logging or unused imports
- [ ] 12.6 Final `cargo check -p deer-gui`
- [ ] 12.7 Mark OpenSpec change as complete

## Critical Constraints

- **NEVER use procedural geometry** (cones, cylinders, spheres) as fallback for missing models
- **NEVER silently skip** missing assets — always panic with clear error message
- **All systems must be scene-driven** — no global spawners
- **Verify each phase** before moving to next (especially Phase 9 before Phase 10)
- **Keep files under 400 LOC**, functions under 50 LOC
- **Follow TDD** — test real features end to end, no mocks
