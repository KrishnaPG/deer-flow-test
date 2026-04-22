## Why

The current Deer GUI uses a Sci-Fi RTS HUD design (Homeworld 2 inspired) that treats the central viewport as a passive 3D scene with orbital camera. This design is suitable for abstract agent visualization but lacks immersive gameplay potential.

**The Opportunity**: Transform Deer GUI into a hybrid FPS/RPG + RTS experience where:
- The central viewport becomes an explorable open-world medieval landscape (first-person or third-person)
- The RTS HUD provides command and control interface around the immersive viewport
- Players can switch between immersive exploration and strategic command modes

This hybrid approach (similar to Mount & Blade, Kingdom Come: Deliverance) creates a unique "commander in the field" experience where the player embodies their agent and directly explores the world while maintaining RTS strategic oversight.

### Modular Architecture Philosophy

**All systems in this change SHALL be built as modular, reusable components** that can be:
- Extracted as standalone crates for use in other Bevy projects
- Configured via Bevy resources (data-driven)
- Documented with examples for external developers
- Integrated as plugins with minimal dependencies on deer_gui internals

**Why Modularity Matters**:
1. **Code Reuse**: Systems like faction theming, weather, or HUD widgets can be used in future games
2. **Maintainability**: Clear public APIs and documentation improve code quality
3. **Testing**: Modular systems are easier to test in isolation
4. **Community**: Reusable crates can be published to crates.io for the Bevy community

## What Changes

- **New Camera System**: Hybrid camera supporting FirstPerson, ThirdPerson, Orbital modes with smooth transitions
- **Medieval Open World**: Heightmap terrain, vegetation, buildings, NPCs for FPS/RPG exploration
- **Faction Theming System**: Dynamic faction themes (English, French, Byzantine, Mongol) that affect UI colors, heraldry, borders
- **Configurable Resources**: Support both traditional RTS resources (Food, Wood, Stone, Gold) and agent system metrics (Tokens, Models, Cost)
- **Weather & Atmosphere**: Time-of-day system, weather effects, seasonal changes
- **BREAKING**: Camera mode replaces pure orbital; some existing CinematicCamera fields become mode-specific

## Capabilities

### New Capabilities

- `medieval-hybrid-camera`: Hybrid camera system supporting FPS, ThirdPerson, Orbital, and Cinematic modes with smooth transitions
- `medieval-open-world`: Heightmap terrain, procedural vegetation, building placement, NPC spawning for explorable medieval landscape
- `faction-theme-system`: Dynamic faction theming affecting UI colors, heraldry symbols, border styles with smooth transitions
- `medieval-hud-components`: RTS-style HUD components adapted for medieval theme (resource bar, command grid, unit panels)
- `configurable-resources`: Resource display system supporting traditional RTS resources and agent metrics with user configuration
- `weather-atmosphere`: Time-of-day, weather effects, atmospheric transitions for immersive environment

### Modified Capabilities

- `scifi-rts-hud`: Extend existing HUD architecture to support medieval theme variants alongside sci-fi

## Impact

### Code Changes
- `apps/deer_gui/src/camera/`: Extend CinematicCamera with mode system
- `apps/deer_gui/src/scene/`: Add medieval scene descriptors and generators
- `apps/deer_gui/src/theme/`: Add faction theme engine
- `apps/deer_gui/src/hud/`: Add medieval HUD components
- `apps/deer_gui/src/world/`: Add terrain, vegetation, NPC components
- `apps/deer_gui/assets/`: Add medieval models, textures, audio

### Battle-Tested Dependencies

Instead of reinventing wheels, we leverage mature Bevy ecosystem crates:

| Purpose | Crate | Downloads | Version |
|---------|-------|-----------|---------|
| Vegetation (GPU instancing, wind) | `bevy_feronia` | 2.5k | 0.8.2 |
| Water (ocean, waves) | `bevy_water` | 40k | 0.18.1 |
| Sky rendering | `bevy_skybox` | 75k | 0.7.0 |
| Smooth transitions | `bevy_easings` | 75k | 0.18.0 |
| NPC behavior trees | `bevior_tree` | 10k | 0.10.0 |
| Particles | `bevy_hanabi` | 136k | 0.18.0 |
| Volumetric fog | `bevy_exponential_height_fog` | 500 | 0.1.0 |

**Custom crates** (no compatible alternatives exist):
- `crates/terrain/` - Heightmap terrain (Bevy 0.18 terrain crates outdated)
- `crates/hybrid-camera/` - Camera mode system (extracted for reuse)
- `crates/faction-themes/` - Faction theming (extracted for reuse)
- `crates/medieval-widgets/` - Egui medieval widgets (extracted for reuse)
- `crates/resource-display/` - Resource UI (extracted for reuse)

### Dependencies
- Medieval 3D assets (glTF models, PBR textures)
- Skybox textures for day/night cycle
- Audio assets (ambient, music, sfx)

### Affected Systems
- Camera plugin (mode switching)
- Scene plugin (terrain/vegetation generators)
- Theme plugin (faction theming)
- HUD plugin (medieval components)
- World plugin (terrain entities)
