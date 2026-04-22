## Context

The Deer GUI application currently implements a Sci-Fi RTS HUD with:
- Orbital CinematicCamera for viewing abstract 3D scenes
- Procedural space scenes (starfields, cloud layers, drop pods)
- Glassmorphism HUD panels (top bar, left/right panels, bottom console)
- Theme system with space-themed palettes (Descent, Precursors, Tet)

The user request transforms this into a hybrid FPS/RPG + RTS experience inspired by Age of Empires 4 with:
- Medieval open-world exploration (FPS/RPG view as primary)
- RTS command interface surrounding the viewport
- Dynamic faction theming (English, French, Byzantine, Mongol)
- Configurable resources (traditional RTS + agent metrics)

## Goals / Non-Goals

**Goals:**
- Hybrid camera system: FirstPerson, ThirdPerson, Orbital modes with Tab-key switching
- Open-world medieval terrain: Heightmap-based with biome textures
- Procedural vegetation: Trees, grass, bushes with wind animation
- Building placement: glTF medieval structures with weathering variants
- NPC population: Skeletal animated characters (idle, walk, work)
- Faction theme system: Dynamic colors, heraldry, borders with smooth transitions
- Weather/time system: Day/night cycle, weather effects, atmospheric transitions
- Medieval HUD: Stone/wood textures, parchment areas, Gothic typography

**Non-Goals:**
- Full RPG gameplay mechanics (inventory, skills, quests) - UI only
- Network multiplayer - single player focus
- Complex AI pathfinding - basic waypoint navigation
- Combat system - placeholder only
- Save/load game state - session only

## Decisions

### 1. Camera System Architecture

**Decision**: Extend existing `CinematicCamera` with mode enum rather than creating new camera entities.

```rust
pub enum CameraMode {
    FirstPerson { height: f32, fov: f32 },
    ThirdPerson { distance: f32, height_offset: f32 },
    Orbital { yaw: f32, pitch: f32, zoom: f32 },
    Cinematic { waypoints: Vec<CameraKeyframe> },
}
```

**Rationale**:
- Reuses existing smooth interpolation infrastructure
- Single camera entity simplifies state management
- Mode transitions use existing lerp systems

**Alternatives Considered**:
- Separate camera entities per mode: Rejected - complicates state sync
- Camera component composition: Rejected - over-engineered for 4 modes

### 2. Terrain System

**Decision**: Use custom heightmap terrain with Bevy's `Mesh::from(heightmap)` rather than `bevy_terrain` crate.

**Rationale**:
- Full control over medieval-specific features (biome blending, building placement)
- Avoids external dependency complexity
- Heightmap format matches existing `GeneratorParams::HeightmapTerrain` design

**Alternatives Considered**:
- `bevy_terrain` crate: Rejected - API instability, learning curve
- Voxel terrain: Rejected - performance overhead for open world

### 3. Faction Theme Engine

**Decision**: Extend existing `ThemeManager` with `FactionTheme` struct and transition system.

```rust
pub struct FactionTheme {
    pub id: FactionId,
    pub primary: Color,
    pub secondary: Color,
    pub heraldic: Color,
    pub border_style: BorderStyle,
    pub symbol: &'static str,
}
```

**Rationale**:
- Builds on existing theme infrastructure
- Reuses `AtmosphereConfig` transition pattern
- Single source of truth for faction visuals

**Alternatives Considered**:
- Separate theme plugin: Rejected - duplicates theme logic
- CSS-like theme cascading: Rejected - unnecessary complexity for 4 factions

### 4. Asset Pipeline

**Decision**: glTF 2.0 for all 3D models with Bevy's built-in loader.

**Rationale**:
- Industry standard format
- Bevy has native support
- Blender export workflow is mature

**Alternatives Considered**:
- Custom format: Rejected - reinvention, no tooling
- FBX: Rejected - licensing concerns, conversion overhead

### 5. Resource Display System

**Decision**: `ResourceMode` enum with Traditional, AgentMetrics, Hybrid variants.

```rust
pub enum ResourceMode {
    Traditional,   // Food, Wood, Stone, Gold, Iron
    AgentMetrics,  // Tokens, Models, Agents, Cost
    Hybrid,        // Both (configurable layout)
}
```

**Rationale**:
- Supports both use cases without code duplication
- User-configurable via settings
- Faction colors apply to resource icons

**Alternatives Considered**:
- Separate HUD plugins: Rejected - code duplication
- Runtime switching only: Rejected - no persistence

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| **Performance**: Open world with vegetation may be slow | Chunked LOD, frustum culling, instanced rendering |
| **Asset size**: Medieval models/textures increase bundle | Streaming from remote CDN, compression (Draco) |
| **Camera transitions**: Jarring mode switches | Smooth lerp (0.3-0.5s), blur transition effect |
| **Faction theming**: Color accessibility issues | High contrast mode, WCAG AA compliance check |
| **Terrain LOD**: Visible pop-in at distance | Blend zones, dithering, pre-load adjacent chunks |
| **Memory**: Many glTF models in memory | Asset unloading, LRU cache, reference counting |

## Migration Plan

### Phase 1: Camera System (Week 1)
1. Add `CameraMode` enum to `CinematicCamera`
2. Implement FirstPerson mode with WASD + mouse look
3. Implement Tab key mode switching with lerp transition
4. Test FPS controls in existing space scene

### Phase 2: Medieval Terrain (Week 2-3)
1. Add `HeightmapTerrain` generator to scene system
2. Implement terrain mesh generation from heightmap
3. Add texture splatting (grass, dirt, rock)
4. Add basic LOD system

### Phase 3: Vegetation & Props (Week 3-4)
1. Add `ProceduralVegetation` generator
2. Implement tree/grass instancing
3. Add wind animation shader
4. Add glTF building placement

### Phase 4: Faction Themes (Week 4-5)
1. Create `FactionTheme` struct
2. Implement faction transition system
3. Add faction selector UI
4. Update HUD components with faction styling

### Phase 5: Medieval HUD (Week 5-6)
1. Create stone/wood texture assets
2. Add Gothic typography fonts
3. Implement medieval resource bar
4. Add 5x3 command grid with medieval styling

### Phase 6: Weather & Atmosphere (Week 6-7)
1. Add time-of-day system
2. Implement weather effects (rain, fog)
3. Add day/night lighting transitions
4. Add ambient audio system

### Rollback Strategy
- Feature flags for each phase (disable in production)
- Parallel asset loading (async, non-blocking)
- Graceful degradation (fallback to space theme if assets fail)

## Open Questions

1. **Player Character Model**: Should we use a placeholder capsule or invest in a medieval knight model early?
2. **Terrain Scale**: What's the optimal world size for FPS exploration? 1km²? 4km²?
3. **NPC AI**: Simple waypoint following or behavior tree system?
4. **Save System**: Do we need persistent world state or is session-only sufficient?
5. **Audio**: Should we license medieval music or use procedural generation?
6. **Accessibility**: How much effort for colorblind mode with faction themes?
