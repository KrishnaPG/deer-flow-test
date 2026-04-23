## 1. Fix Scene Cleanup

- [ ] 1.1 Change `SceneManager::deactivate()` from `despawn()` to `despawn_recursive()` in `apps/deer_gui/src/scene/manager.rs`
- [ ] 1.2 Verify no orphaned entities remain after scene switch (check via bevy inspector or logging)

## 2. Invoke Generators from DescriptorSceneConfig

- [ ] 2.1 Modify `DescriptorSceneConfig::spawn_environment()` to accept `Res<GeneratorRegistry>`
- [ ] 2.2 Add generator invocation loop: for each spec in descriptor.generators, lookup factory in registry and call it
- [ ] 2.3 Log generator invocations for debugging
- [ ] 2.4 Verify TET scene loads starfield + spiral_trails from `tet.scene.ron`

## 3. Convert TET to Data-Driven

- [ ] 3.1 Remove `TetSceneConfig` from `apps/deer_gui/src/scene/tet/config.rs`
- [ ] 3.2 Remove TET pre-registration from `ScenePlugin::build()`
- [ ] 3.3 Update `scene_startup_system` to load TET from `tet.scene.ron` via `DescriptorSceneConfig::from_file`
- [ ] 3.4 Verify TET scene visually matches hardcoded version (starfield + monolith + trails)

## 4. Convert World Plugins to Generators

- [ ] 4.1 Extract water spawn logic from `WaterPlugin::build()` into `gen_water()` generator function
- [ ] 4.2 Register `gen_water` in `GeneratorRegistry::with_builtins()`
- [ ] 4.3 Extract foliage spawn logic from `FoliagePlugin::build()` into `gen_foliage()` generator function
- [ ] 4.4 Register `gen_foliage` in `GeneratorRegistry::with_builtins()`
- [ ] 4.5 Extract NPC spawn logic from `NpcPlugin::build()` into `gen_npcs()` generator function
- [ ] 4.6 Register `gen_npcs` in `GeneratorRegistry::with_builtins()`
- [ ] 4.7 Remove `Startup` spawn systems from WaterPlugin, FoliagePlugin, NpcPlugin

## 5. Update Medieval Scene Descriptor

- [ ] 5.1 Add `water` generator to `medieval_open.scene.ron`
- [ ] 5.2 Add `vegetation` generators (Meadow, Forest) to `medieval_open.scene.ron`
- [ ] 5.3 Add `npcs` generator to `medieval_open.scene.ron`
- [ ] 5.4 Remove hardcoded `medieval_open` pre-registration from `ScenePlugin::build()`

## 6. Handle Universal Lighting

- [ ] 6.1 Keep `setup_landscape` in `main.rs` as global startup (sun + ambient light)
- [ ] 6.2 Verify lighting works correctly across all scenes

## 7. Validation

- [ ] 7.1 Run `cargo check -p deer-gui` and fix all compiler errors
- [ ] 7.2 Test `--scene tet`: verify starfield + monolith + trails, no water/foliage/npc
- [ ] 7.3 Test `--scene descent`: verify cloud layer + drop pods + glow clusters, no water/foliage/npc
- [ ] 7.4 Test `--scene precursors`: verify river barges + travellers + glow clusters, no water/foliage/npc
- [ ] 7.5 Test `--scene medieval_open`: verify terrain + vegetation + water + NPCs
- [ ] 7.6 Test scene switching: tet → medieval → descent → precursors → tet (check for leaks)
- [ ] 7.7 Verify no placeholder PNG files remain in `apps/deer_gui/assets/textures/terrain/`
