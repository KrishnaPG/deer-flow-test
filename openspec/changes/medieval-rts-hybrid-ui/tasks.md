# Medieval RTS Hybrid UI - Task List

## Architecture Note: Modular Reusable Crates

This project emphasizes building systems as **modular, reusable crates** that can be extracted and used in other Bevy projects. Each major system should:
- Have minimal dependencies on deer_gui internals
- Expose a clean public API
- Include comprehensive documentation
- Be configurable via resource structs
- Support Bevy's plugin system for easy integration

---

## 1. Hybrid Camera System

- [x] 1.1 Add CameraMode enum to CinematicCamera component
- [x] 1.2 Implement FirstPerson mode with WASD movement
- [x] 1.3 Implement mouse look with pitch/yaw rotation
- [x] 1.4 Implement ThirdPerson mode with orbit controls
- [x] 1.5 Implement Orbital mode (existing functionality refactor)
- [x] 1.6 Add Tab key mode toggle with smooth lerp transition
- [ ] 1.7 Add camera mode settings to user preferences
- [x] 1.8 Write unit tests for camera mode switching

**Modularity**: Camera system already modular in `apps/deer_gui/src/camera/`. Consider extracting to `crates/hybrid-camera` for reuse.

## 2. Medieval Terrain System

> **Note**: No mature Bevy 0.18 compatible terrain crate exists (`bevy_mesh_terrain` only supports 0.15).
> We continue with our custom implementation in `crates/terrain/` which provides reusable heightmap processing.

- [x] 2.1 Create HeightmapTerrain generator in scene/generators/
- [x] 2.2 Implement heightmap mesh generation from PNG
- [x] 2.3 Add terrain material with texture splatting (basic)
- [x] 2.6 Implement basic LOD system for terrain chunks
- [ ] 2.4 Create terrain texture assets (grass, dirt, rock, snow)
- [ ] 2.5 Create splat mask texture format
- [ ] 2.7 Add terrain to scene descriptor RON format
- [ ] 2.8 Write integration tests for terrain loading
- [ ] 2.9 Extract `crates/terrain` as standalone reusable crate
- [ ] 2.10 Document terrain API for external use

## 3. Procedural Vegetation (Using bevy_feronia)

> **Decision**: Replace custom `crates/vegetation/` with **`bevy_feronia`** (0.8.2, Bevy 0.18 compatible).
> `bevy_feronia` provides GPU instancing, wind simulation, and scattering tools out of the box.

**Dependencies**: `bevy_feronia = "0.8.2"`

- [ ] 3.1 Add `bevy_feronia` dependency to Cargo.toml
- [ ] 3.2 Create `FoliagePlugin` wrapper for reusability in other projects
- [ ] 3.3 Define biome-to-foliage mappings (Meadow, Forest, Rocky, etc.)
- [ ] 3.4 Configure scattering density per biome type
- [ ] 3.5 Create glTF tree models (oak, pine, birch) compatible with bevy_feronia
- [ ] 3.6 Create glTF bush and grass models
- [ ] 3.7 Configure wind animation parameters
- [ ] 3.8 Add vegetation to scene descriptor RON format
- [ ] 3.9 Delete `crates/vegetation/` (replaced by bevy_feronia)
- [ ] 3.10 Document `FoliagePlugin` API for reuse in other projects

## 4. Building Placement System

> **Modularity Note**: Building placement should be a reusable `BuildingPlugin` that can be used
> in any project needing structure placement with variants (weathering, faction colors).

- [ ] 4.1 Create `BuildingPlugin` with clean public API
- [ ] 4.2 Define `BuildingDef` struct for building configurations
- [ ] 4.3 Implement building placement with layout presets
- [ ] 4.4 Create glTF medieval building models (house, tower, church)
- [ ] 4.5 Add weathering level support (pristine to ruined) via material variants
- [ ] 4.6 Implement faction color application to buildings
- [ ] 4.7 Add collision shapes to buildings
- [ ] 4.8 Create building cluster layout presets (village, castle, farm)
- [ ] 4.9 Write unit tests for BuildingDef serialization
- [ ] 4.10 Document `BuildingPlugin` API for external reuse

## 5. NPC Population System (Using bevior_tree)

> **Decision**: Use **`bevior_tree`** (0.10.0) for NPC behavior trees.
> This provides a reusable behavior tree implementation for any Bevy project.
> Also consider **`bevy_bae`** (0.1.0) for "Behavior as Entities" pattern.

**Dependencies**: `bevior_tree = "0.10.0"` (optional: `bevy_bae = "0.1.0"`)

- [ ] 5.1 Add `bevior_tree` dependency to Cargo.toml
- [ ] 5.2 Create `NpcPlugin` with configurable spawning parameters
- [ ] 5.3 Define `NpcType` enum (Knight, Peasant, Guard, etc.)
- [ ] 5.4 Implement NPC spawning with count and radius
- [ ] 5.5 Create glTF character models (knight, peasant)
- [ ] 5.6 Create glTF horse model
- [ ] 5.7 Implement skeletal animation controller
- [ ] 5.8 Add idle, walk, work animation states
- [ ] 5.9 Define behavior trees for NPC types:
       - [ ] 5.9.1 Idle behavior tree
       - [ ] 5.9.2 Wander behavior tree (waypoint navigation)
       - [ ] 5.9.3 Work behavior tree
- [ ] 5.10 Create reusable `NpcBehaviorTreeBuilder` for other projects
- [ ] 5.11 Document `NpcPlugin` and behavior tree API

## 6. Water Plane System (Using bevy_water)

> **Decision**: Replace custom `crates/water/` with **`bevy_water`** (0.18.1, Bevy 0.18 compatible).
> `bevy_water` provides dynamic ocean material with wave animations and `bevy_easings` integration.

**Dependencies**: `bevy_water = "0.18.1"`, `bevy_easings = "0.18.0"`

- [ ] 6.1 Add `bevy_water` and `bevy_easings` dependencies to Cargo.toml
- [ ] 6.2 Create `WaterPlugin` wrapper for reusable configuration
- [ ] 6.3 Configure water plane for rivers and lakes (not just ocean)
- [ ] 6.4 Set up wave animation parameters (amplitude, frequency, speed)
- [ ] 6.5 Add flow direction configuration for rivers
- [ ] 6.6 Integrate `bevy_water` with terrain heightmap (shore detection)
- [ ] 6.7 Add splash particle effect on water contact (using bevy_hanabi)
- [ ] 6.8 Implement water collision for swimming
- [ ] 6.9 Delete `crates/water/` (replaced by bevy_water)
- [ ] 6.10 Document `WaterPlugin` configuration API for reuse

## 7. Faction Theme Engine

> **Modularity Note**: Faction theming should be extractable to `crates/faction-themes` for reuse
> in any game needing faction-based UI styling with smooth transitions.

- [ ] 7.1 Create `FactionThemePlugin` with clean public API
- [ ] 7.2 Define `FactionTheme` struct (colors, border_style, symbol)
- [ ] 7.3 Define `FactionId` enum (English, French, Byzantine, Mongol)
- [ ] 7.4 Create `FactionPreset` resource for faction definitions
- [ ] 7.5 Implement smooth faction transition system (2-4s, EaseInOutCubic)
- [ ] 7.6 Implement color interpolation with purple midpoint transition
- [ ] 7.7 Implement border style morphing during transitions
- [ ] 7.8 Implement heraldry crossfade effect
- [ ] 7.9 Create faction symbol assets (SVG/texture)
- [ ] 7.10 Add faction selector UI to settings
- [ ] 7.11 Write unit tests for FactionTheme serialization
- [ ] 7.12 Write integration tests for faction transitions
- [ ] 7.13 Document `FactionThemePlugin` API for external reuse

## 8. Medieval HUD Components

> **Modularity Note**: Each HUD component should be a standalone egui widget that can be
> extracted to `crates/medieval-widgets` for use in any egui-based application.

- [ ] 8.1 Create `MedievalWidgets` module with reusable egui components
- [ ] 8.2 Implement `medieval_resource_bar` widget
- [ ] 8.3 Implement `command_grid(rows, cols)` widget (configurable grid)
- [ ] 8.4 Implement `selection_panel` widget
- [ ] 8.5 Implement `octagonal_minimap` widget with frame styling
- [ ] 8.6 Implement `event_feed` widget with medieval styling
- [ ] 8.7 Create stone/wood/parchment texture assets
- [ ] 8.8 Create Gothic font assets (or use Noto Sans Gothic)
- [ ] 8.9 Implement faction color application via `FactionTheme` integration
- [ ] 8.10 Create `MedievalStyle` struct for consistent widget theming
- [ ] 8.11 Write unit tests for each widget
- [ ] 8.12 Document `MedievalWidgets` API for external reuse

## 9. Configurable Resources

> **Modularity Note**: Resource system should be extractable to `crates/resource-display` for reuse
> in any game/app needing configurable resource UI with multiple display modes.

- [ ] 9.1 Create `ResourceDisplayPlugin` with configurable modes
- [ ] 9.2 Define `ResourceMode` enum (Traditional, AgentMetrics, Hybrid)
- [ ] 9.3 Define `ResourceConfig` struct for threshold customization
- [ ] 9.4 Implement Traditional resource display (Food, Wood, Stone, Gold, Iron)
- [ ] 9.5 Implement AgentMetrics display (Tokens, Models, Agents, Cost)
- [ ] 9.6 Implement Hybrid layout with collapsible sections
- [ ] 9.7 Create resource icon assets (wheat, log, stone, coin, ingot)
- [ ] 9.8 Add resource threshold warnings (low/critical states)
- [ ] 9.9 Add resource mode to user preferences persistence
- [ ] 9.10 Write unit tests for ResourceConfig serialization
- [ ] 9.11 Write integration tests for display modes
- [ ] 9.12 Document `ResourceDisplayPlugin` API for external reuse

## 10. Time of Day System (Using bevy_skybox + bevy_easings)

> **Decision**: Use **`bevy_skybox`** (0.7.0) for sky rendering and **`bevy_easings`** (0.18.0)
> for smooth time transitions. Combined 150k+ downloads, battle-tested.

**Dependencies**: `bevy_skybox = "0.7.0"`, `bevy_easings = "0.18.0"`

- [ ] 10.1 Add `bevy_skybox` and `bevy_easings` dependencies to Cargo.toml
- [ ] 10.2 Create `DayNightPlugin` with clean public API
- [ ] 10.3 Define `TimeOfDay` resource (hour 0-24, time scale)
- [ ] 10.4 Implement time progression with configurable scale
- [ ] 10.5 Calculate sun position based on time (azimuth/elevation)
- [ ] 10.6 Configure directional light color interpolation (warm dawn/dusk, cool noon)
- [ ] 10.7 Set up skybox textures for day/night cycle
- [ ] 10.8 Add star field for night sky (texture or procedural)
- [ ] 10.9 Add moon rendering for night hours
- [ ] 10.10 Integrate `bevy_easings` for smooth light transitions
- [ ] 10.11 Write unit tests for TimeOfDay calculations
- [ ] 10.12 Document `DayNightPlugin` API for external reuse

## 11. Weather System (Using bevy_hanabi + bevy_exponential_height_fog)

> **Decision**: Use existing **`bevy_hanabi`** (0.18.0, already in project) for particles.
> Add **`bevy_exponential_height_fog`** (0.1.0) for volumetric fog effects.

**Dependencies**: `bevy_hanabi = "0.18.0"` (existing), `bevy_exponential_height_fog = "0.1.0"`

- [ ] 11.1 Add `bevy_exponential_height_fog` dependency to Cargo.toml
- [ ] 11.2 Create `WeatherPlugin` with clean public API
- [ ] 11.3 Define `WeatherState` enum (Clear, Cloudy, Rainy, Foggy, Stormy)
- [ ] 11.4 Define `WeatherConfig` struct for transition timing
- [ ] 11.5 Implement smooth weather transition system (configurable duration)
- [ ] 11.6 Configure rain particle effect using bevy_hanabi
- [ ] 11.7 Configure fog effect using exponential height fog
- [ ] 11.8 Implement storm effect (rain + lightning flashes)
- [ ] 11.9 Add lightning flash light component
- [ ] 11.10 Add thunder audio trigger system
- [ ] 11.11 Create weather ambient audio assets
- [ ] 11.12 Write unit tests for WeatherState serialization
- [ ] 11.13 Document `WeatherPlugin` API for external reuse

## 12. Season System (Integrates with bevy_feronia)

> **Note**: No dedicated season crate exists. We implement custom season system that
> integrates with `bevy_feronia` vegetation for color/texture changes.
> **Modularity**: Extractable to `crates/season-system` for any project needing seasonal changes.

**Integrates with**: `bevy_feronia` (Task 3), `bevy_exponential_height_fog` (Task 11)

- [ ] 12.1 Create `SeasonPlugin` with clean public API
- [ ] 12.2 Define `SeasonState` enum (Spring, Summer, Autumn, Winter)
- [ ] 12.3 Define `SeasonConfig` struct for transition timing
- [ ] 12.4 Implement smooth season transition system
- [ ] 12.5 Integrate with bevy_feronia for vegetation color changes
- [ ] 12.6 Implement terrain texture blending per season
- [ ] 12.7 Configure fog color/haze changes per season
- [ ] 12.8 Add falling leaves particle effect for Autumn (bevy_hanabi)
- [ ] 12.9 Add snow particle effect for Winter (bevy_hanabi)
- [ ] 12.10 Add snow overlay mesh for Winter ground cover
- [ ] 12.11 Create seasonal texture variants for terrain
- [ ] 12.12 Write unit tests for SeasonState serialization
- [ ] 12.13 Write integration tests for season transitions
- [ ] 12.14 Document `SeasonPlugin` API for external reuse

## 13. Asset Pipeline

> **Modularity Note**: Asset loading patterns should be reusable across projects.
> Consider extracting `crates/asset-helpers` for common loading patterns.

- [ ] 13.1 Create `AssetHelperPlugin` for common loading patterns
- [ ] 13.2 Create medieval scene descriptor RON format
- [ ] 13.3 Create example `medieval_open.scene.ron`
- [ ] 13.4 Create example `tutorial_village.scene.ron`
- [ ] 13.5 Document glTF model export workflow (Blender -> Bevy)
- [ ] 13.6 Create terrain texture atlas with mipmaps
- [ ] 13.7 Create audio asset collection (ambient, music, sfx)
- [ ] 13.8 Document asset naming conventions
- [ ] 13.9 Implement async asset loading with progress tracking
- [ ] 13.10 Create asset loading error handling and fallbacks
- [ ] 13.11 Document `AssetHelperPlugin` API for external reuse

## 14. Integration & Testing

- [ ] 14.1 Integrate all plugins in main.rs
- [ ] 14.2 Create medieval RTS demo scene
- [ ] 14.3 Test camera mode switching in demo
- [ ] 14.4 Test faction theme switching in demo
- [ ] 14.5 Test resource display modes in demo
- [ ] 14.6 Test weather transitions in demo
- [ ] 14.7 Test season transitions in demo
- [ ] 14.8 Performance profiling for open world
- [ ] 14.9 Create user documentation

## 15. Modular Crate Extraction (Optional Future Work)

> **Goal**: Extract reusable systems as standalone crates for other Bevy projects.

- [ ] 15.1 Extract `hybrid-camera` crate from camera system
- [ ] 15.2 Extract `terrain` crate (keep existing, improve API)
- [ ] 15.3 Extract `faction-themes` crate from faction system
- [ ] 15.4 Extract `medieval-widgets` crate from HUD components
- [ ] 15.5 Extract `resource-display` crate from resource system
- [ ] 15.6 Extract `day-night` crate from time of day system
- [ ] 15.7 Extract `weather` crate from weather system
- [ ] 15.8 Extract `season-system` crate from season system
- [ ] 15.9 Extract `building-placement` crate from building system
- [ ] 15.10 Extract `npc-spawner` crate from NPC system
- [ ] 15.11 Create example projects for each extracted crate
- [ ] 15.12 Publish crates to crates.io (if desired)
