## Why

The current Deer GUI uses a Sci-Fi RTS HUD design (Homeworld 2 inspired) that treats the central viewport as a passive 3D scene with orbital camera. This design is suitable for abstract agent visualization but lacks immersive gameplay potential.

**The Opportunity**: Transform Deer GUI into a hybrid FPS/RPG + RTS experience where:
- The central viewport becomes an explorable open-world medieval landscape (first-person or third-person)
- The RTS HUD provides command and control interface around the immersive viewport
- Players can switch between immersive exploration and strategic command modes

This hybrid approach (similar to Mount & Blade, Kingdom Come: Deliverance) creates a unique "commander in the field" experience where the player embodies their agent and directly explores the world while maintaining RTS strategic oversight.

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

### Dependencies
- `bevy_terrain` or custom heightmap system for open world terrain
- `bevy_hanabi` already included for particles (dust, fire, smoke)
- Medieval 3D assets (glTF models, PBR textures)

### Affected Systems
- Camera plugin (mode switching)
- Scene plugin (terrain/vegetation generators)
- Theme plugin (faction theming)
- HUD plugin (medieval components)
- World plugin (terrain entities)
