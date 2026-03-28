# Deer GUI: Bevy 0.18 Cinematic Control Center — Implementation Plan

## Executive Summary

Migrate `apps/deer_gui` from an eframe/egui chat app to a **Bevy 0.18.1** cinematic game engine
with a 3D world, ECS-driven entity system, HUD overlay via `bevy_egui`, GPU particle effects via
`bevy_hanabi`, and spatial audio via `bevy_kira_audio`. The existing `bridge.rs` and `models.rs`
are preserved as the data layer. The current 759-line `app.rs` is replaced entirely.

---

## Constraints (from AGENTS.md)

- Files < 400 LOC, functions < 50 LOC
- Strong typing for domain models
- TDD — every system testable stand-alone with real functionality (no mocks)
- Dynamic tracing/debug-logs on all methods
- Zero-copy memory buffers where possible
- Low coupling — every module reusable across projects
- Constants centralized in one place
- Design for future extensibility

---

## Plugin Stack (Bevy 0.18.1 Compatible, No External System Deps)

| Crate | Version | Purpose |
|-------|---------|---------|
| `bevy` | `0.18.1` | Game engine (3D renderer, ECS, asset pipeline) |
| `bevy_egui` | `0.39` | egui HUD overlay on Bevy render |
| `bevy_hanabi` | `0.18` | GPU particle effects (agents, trails, weather) |
| `bevy_kira_audio` | `0.25` | Spatial audio (ambient loops, event sounds) |
| `bevy_tweening` | `0.15` | Smooth camera interpolation, UI animations |
| `bevy_prototype_lyon` | `0.16` | 2D shape drawing (minimap, HUD decorations) |

**Note:** `bevy_kira_audio` requires disabling Bevy's default `bevy_audio` + `vorbis` features.
Parallax is implemented manually (transform-based depth layers) since `bevy-parallax` doesn't
support Bevy 0.18.

---

## Cargo.toml Dependencies

```toml
[dependencies]
bevy = { version = "0.18.1", default-features = false, features = [
    "bevy_asset", "bevy_core_pipeline", "bevy_pbr", "bevy_render",
    "bevy_winit", "bevy_gizmos", "bevy_input", "bevy_log",
    "bevy_state", "bevy_scene", "bevy_picking", "bevy_mesh",
    "png", "x11", "wayland",
] }
bevy_egui = "0.39"
bevy_hanabi = "0.18"
bevy_kira_audio = "0.25"
bevy_tweening = "0.15"
bevy_prototype_lyon = "0.16"

# Kept from existing
chrono = { version = "0.4.44", features = ["clock", "serde"] }
crossbeam-channel = "0.5.15"
directories = "6.0.0"
dotenvy = "0.15.7"
env_logger = "0.11.10"
log = "0.4.29"
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.149"
uuid = { version = "1.23.0", features = ["serde", "v4"] }
```

**Removed:** `eframe`, `egui` (direct), `rfd`, `opener` — replaced by Bevy equivalents.

---

## File Structure (Target)

```
apps/deer_gui/src/
├── main.rs                     # Bevy App builder, plugin registration
├── constants.rs                # All magic numbers, colors, z-layers, timings
├── models.rs                   # (KEEP) Domain models - ThreadSummary, ChatMessage, etc.
│
├── bridge/                     # Python IPC bridge
│   ├── mod.rs                  # Module exports
│   ├── client.rs               # (REFACTORED from bridge.rs) BridgeClient struct + spawn
│   ├── commands.rs             # Outbound command types
│   ├── events.rs               # BridgeEvent, BridgeResponse enums
│   └── plugin.rs               # BridgePlugin: Bevy plugin, background thread → EventWriter
│
├── world/                      # The 3D World (Z0)
│   ├── mod.rs                  # Module exports
│   ├── state.rs                # WorldState resource, entity registry
│   ├── components.rs           # ECS components: Agent, Swarm, Mission, Artifact, WorldEntity
│   ├── spatial.rs              # Spatial index (BVH/grid) for picking
│   ├── systems.rs              # Systems: spawn/despawn entities, update positions
│   └── plugin.rs               # WorldPlugin
│
├── camera/                     # 3D Camera System
│   ├── mod.rs
│   ├── components.rs           # CinematicCamera component (yaw, pitch, zoom, shake, targets)
│   ├── systems.rs              # Camera input handling, smooth interpolation, focus
│   └── plugin.rs               # CameraPlugin
│
├── scene/                      # Cinematic Scene Rendering (Z0)
│   ├── mod.rs
│   ├── traits.rs               # SceneConfig trait
│   ├── tet/                    # TET Orchestrator scene
│   │   ├── mod.rs
│   │   ├── config.rs           # TetSceneConfig: implements SceneConfig
│   │   ├── setup.rs            # System: spawn TET structure, starfield, data trails
│   │   ├── systems.rs          # Systems: animate stars (parallax), TET glow, data trails
│   │   └── particles.rs        # bevy_hanabi effect configs for TET
│   ├── common/                 # Shared scene utilities
│   │   ├── mod.rs
│   │   ├── parallax.rs         # Manual parallax layers (transform-based depth scrolling)
│   │   ├── weather.rs          # Weather state machine + particle effects
│   │   └── atmosphere.rs       # Sky gradient, ambient lighting tied to system health
│   └── plugin.rs               # ScenePlugin
│
├── hud/                        # HUD Overlay (Z1) via bevy_egui
│   ├── mod.rs
│   ├── top_bar.rs              # Global vitals: agent counts, token burn, system health
│   ├── bottom_console.rs       # Command input, event ticker, minimap/radar
│   ├── left_panel.rs           # Missions/control groups with progress bars
│   ├── right_inspector.rs      # Entity inspector: details, logs, actions
│   ├── event_ticker.rs         # Scrolling event feed with auto-fade
│   ├── center_canvas.rs        # Glass panel modes: Experts Meeting, Swarm Monitor, etc.
│   ├── modal.rs                # Z2 modal screens
│   ├── styles.rs               # Semi-transparent glass panel styles
│   └── plugin.rs               # HudPlugin
│
├── theme/                      # Theme Engine
│   ├── mod.rs
│   ├── theme.rs                # Theme struct, ThemeManager resource
│   ├── tet_theme.rs            # TET color palette, egui Visuals overrides
│   └── plugin.rs               # ThemePlugin
│
├── audio/                      # Audio System
│   ├── mod.rs
│   ├── manager.rs              # AudioManager resource, ambient loop control
│   ├── events.rs               # AudioEvent enum
│   └── plugin.rs               # AudioPlugin (bevy_kira_audio integration)
│
├── picking/                    # Entity Selection / Ray Picking
│   ├── mod.rs
│   ├── systems.rs              # 3D raycast, spatial index query, selection state
│   └── plugin.rs               # PickingPlugin
│
└── diagnostics/                # Tracing & Debug
    ├── mod.rs
    ├── tracing.rs              # Bevy tracing subscriber config
    └── plugin.rs               # DiagnosticsPlugin
```

**Total: ~35 files across 10 modules. Each file stays under 400 LOC.**

---

## Core Systems & Interfaces

### 1. Constants (`constants.rs`)

All magic numbers centralized:

```rust
// Z-layers for 3D depth ordering
pub const Z_WORLD_FAR: f32 = -1000.0;
pub const Z_WORLD_MID: f32 = -500.0;
pub const Z_WORLD_NEAR: f32 = -100.0;
pub const Z_HUD: f32 = 100.0;

// Camera defaults
pub const CAMERA_DEFAULT_YAW: f32 = 0.0;
pub const CAMERA_DEFAULT_PITCH: f32 = -15.0;
pub const CAMERA_DEFAULT_ZOOM: f32 = 1.0;
pub const CAMERA_INTERPOLATION_SPEED: f32 = 4.0;
pub const CAMERA_SHAKE_DECAY: f32 = 5.0;
pub const CAMERA_MIN_ZOOM: f32 = 0.5;
pub const CAMERA_MAX_ZOOM: f32 = 5.0;
pub const CAMERA_MIN_PITCH: f32 = -60.0;
pub const CAMERA_MAX_PITCH: f32 = 10.0;

// Timing
pub const BRIDGE_POLL_INTERVAL_MS: u64 = 16;
pub const EVENT_TICKER_FADE_SECS: f32 = 8.0;
pub const WEATHER_TRANSITION_SECS: f32 = 3.0;

// Visual
pub const AGENT_PARTICLE_SIZE: f32 = 0.3;
pub const HUD_PANEL_ALPHA: f32 = 0.75;
pub const MAX_VISIBLE_AGENTS: usize = 500;
pub const TET_STRUCTURE_RADIUS: f32 = 50.0;
pub const STARFIELD_COUNT: usize = 2000;
pub const STARFIELD_RADIUS: f32 = 800.0;

// Window
pub const WINDOW_TITLE: &str = "Deer GUI — Control Center";
pub const WINDOW_DEFAULT_WIDTH: f32 = 1480.0;
pub const WINDOW_DEFAULT_HEIGHT: f32 = 920.0;
pub const WINDOW_MIN_WIDTH: f32 = 1080.0;
pub const WINDOW_MIN_HEIGHT: f32 = 760.0;
```

### 2. Bridge Plugin (`bridge/plugin.rs`)

Wraps the existing `BridgeClient` into Bevy's ECS:

```rust
pub struct BridgePlugin;

// Resources
#[derive(Resource)]
pub struct BridgeClientResource(pub Option<BridgeClient>);

// Bevy Events (bridged from crossbeam into Bevy's event system)
#[derive(Event)]
pub struct BridgeEventReceived(pub BridgeEvent);

// Systems:
// - bridge_startup_system: spawns BridgeClient, stores in resource
// - bridge_poll_system: each frame, drains crossbeam channel → EventWriter<BridgeEventReceived>
// - bridge_event_handler_system: reads events, updates WorldState/HUD resources
```

### 3. World Components (`world/components.rs`)

```rust
#[derive(Component)]
pub struct WorldEntity {
    pub entity_id: EntityId,
    pub entity_type: WorldEntityType,
}

#[derive(Clone, Debug)]
pub enum WorldEntityType {
    Agent(AgentState),
    Swarm { agent_count: u32 },
    Mission { progress: f32 },
    Artifact { artifact_type: ArtifactKind },
}

#[derive(Clone, Debug, PartialEq)]
pub enum AgentState { Idle, Working, Error, ApprovalNeeded }

#[derive(Clone, Debug, PartialEq)]
pub enum ArtifactKind { Document, Index, Embedding, Raw }

// Marker components
#[derive(Component)] pub struct Agent;
#[derive(Component)] pub struct Swarm;
#[derive(Component)] pub struct Mission;
#[derive(Component)] pub struct Artifact;
#[derive(Component)] pub struct Selectable;
#[derive(Component)] pub struct Selected;
#[derive(Component)] pub struct PulsingBeacon { pub frequency: f32 }
```

### 4. Camera Components (`camera/components.rs`)

```rust
#[derive(Component)]
pub struct CinematicCamera {
    pub yaw_deg: f32,
    pub target_yaw: f32,
    pub pitch_deg: f32,
    pub target_pitch: f32,
    pub zoom: f32,
    pub target_zoom: f32,
    pub shake: f32,
    pub focus_target: Option<Vec3>,
}
```

### 5. Scene Trait (`scene/traits.rs`)

```rust
pub trait SceneConfig: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn spawn_environment(
        &self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    );
    fn particle_config_for_agent(&self, state: &AgentState) -> EffectAsset;
    fn map_system_health_to_atmosphere(&self, health: &SystemHealth) -> AtmosphereState;
    fn ambient_audio_track(&self) -> &str;
}
```

### 6. HUD State Resources

```rust
#[derive(Resource, Default)]
pub struct HudState {
    pub active_agents: u32,
    pub retrying_agents: u32,
    pub failed_agents: u32,
    pub tokens_per_sec: f32,
    pub cost_per_hour: f32,
    pub system_online: bool,
    pub missions: Vec<MissionSummary>,
    pub selected_entity: Option<EntityInspectorData>,
    pub event_log: Vec<EventLogEntry>,
    pub command_input: String,
    pub center_mode: CenterCanvasMode,
}

#[derive(Default, PartialEq)]
pub enum CenterCanvasMode {
    #[default]
    WorldView,
    ExpertsMeeting,
    SwarmMonitor,
    ArtifactGraph,
    Forensics,
}
```

### 7. Audio Events (`audio/events.rs`)

```rust
#[derive(Event)]
pub enum AudioCommand {
    PlayAmbient { track: String, fade_in_secs: f32 },
    StopAmbient { fade_out_secs: f32 },
    PlayOneShot { sound: UiSound },
    SetVolume { volume: f32 },
}

#[derive(Clone, Debug)]
pub enum UiSound {
    Click, ApprovalChime, ErrorTone, AlertPing, CameraMove,
}
```

---

## Phased Implementation

### Phase 1: Bevy Skeleton & Camera (Foundation)

**Goal:** Bevy app boots, shows a 3D scene with a controllable camera.

| Step | File(s) | Description |
|------|---------|-------------|
| 1.1 | `Cargo.toml` | Update deps: add Bevy + plugins, remove eframe/egui |
| 1.2 | `constants.rs` | Create centralized constants file |
| 1.3 | `main.rs` | Rewrite: `App::new()`, `DefaultPlugins` (minus `bevy_audio`), register plugins |
| 1.4 | `camera/components.rs` | `CinematicCamera` component |
| 1.5 | `camera/systems.rs` | Mouse drag rotation, scroll zoom, smooth interpolation |
| 1.6 | `camera/plugin.rs` | `CameraPlugin`: startup spawns camera, registers input systems |
| 1.7 | `diagnostics/` | Tracing setup, FPS overlay |

### Phase 2: TET Scene (Z0 World)

**Goal:** Stars, TET structure, data trails rendered behind the camera.

| Step | File(s) | Description |
|------|---------|-------------|
| 2.1 | `scene/traits.rs` | Define `SceneConfig` trait |
| 2.2 | `scene/common/parallax.rs` | Manual parallax: depth layers scroll at different rates |
| 2.3 | `scene/tet/config.rs` | `TetSceneConfig` implementing the trait |
| 2.4 | `scene/tet/setup.rs` | Spawn starfield, TET monolith, data trail paths |
| 2.5 | `scene/tet/systems.rs` | Animate star parallax, TET glow pulsing, data trail movement |
| 2.6 | `scene/tet/particles.rs` | `bevy_hanabi` effects: agent moths, data stream trails |
| 2.7 | `scene/common/atmosphere.rs` | Ambient light mapped to `SystemHealth` |
| 2.8 | `scene/plugin.rs` | `ScenePlugin` |

### Phase 3: Bridge Integration (Data Flow)

**Goal:** Bridge events flow from Python → Bevy ECS → World entities.

| Step | File(s) | Description |
|------|---------|-------------|
| 3.1 | `bridge/events.rs` | Extract `BridgeEvent`, `BridgeResponse` from monolithic bridge.rs |
| 3.2 | `bridge/commands.rs` | Extract `OutboundCommand` types |
| 3.3 | `bridge/client.rs` | Extract `BridgeClient` (spawn, send methods) |
| 3.4 | `bridge/plugin.rs` | `BridgePlugin`: startup spawns client, per-frame poll → `EventWriter` |
| 3.5 | `world/state.rs` | `WorldState` resource: entity registry, system health metrics |
| 3.6 | `world/components.rs` | ECS components for agents, missions, artifacts |
| 3.7 | `world/systems.rs` | Bridge event handler: events → spawn/update/despawn entities |
| 3.8 | `world/plugin.rs` | `WorldPlugin` |

### Phase 4: HUD Overlay (Z1) via bevy_egui

**Goal:** Semi-transparent HUD panels overlaid on the 3D world.

| Step | File(s) | Description |
|------|---------|-------------|
| 4.1 | `hud/styles.rs` | Glass panel frame style, semi-transparent backgrounds |
| 4.2 | `hud/top_bar.rs` | Agent counts, token burn, health |
| 4.3 | `hud/left_panel.rs` | Mission list with progress bars |
| 4.4 | `hud/right_inspector.rs` | Selected entity details, approve/halt/retry buttons |
| 4.5 | `hud/bottom_console.rs` | Command input field, mode selector, execute button |
| 4.6 | `hud/event_ticker.rs` | Scrolling event log with auto-fade |
| 4.7 | `hud/center_canvas.rs` | Glass panel for Experts Meeting, Swarm Monitor, Artifact Graph |
| 4.8 | `hud/modal.rs` | Full-screen Z2 overlays |
| 4.9 | `hud/plugin.rs` | `HudPlugin` |

### Phase 5: Entity Picking & Selection

**Goal:** Click on 3D entities → select → inspector updates.

| Step | File(s) | Description |
|------|---------|-------------|
| 5.1 | `world/spatial.rs` | Spatial index (grid-based) for efficient 3D queries |
| 5.2 | `picking/systems.rs` | Bevy `bevy_picking` for 3D raycast, selection management |
| 5.3 | `picking/plugin.rs` | `PickingPlugin` |
| 5.4 | `camera/systems.rs` | Focus system: smooth pan to entity cluster via `bevy_tweening` |

### Phase 6: Theme Engine

**Goal:** Runtime-switchable visual themes.

| Step | File(s) | Description |
|------|---------|-------------|
| 6.1 | `theme/theme.rs` | `Theme` struct, `ThemeManager` resource |
| 6.2 | `theme/tet_theme.rs` | TET theme: deep blues/blacks/cyans, glass panel colors |
| 6.3 | `theme/plugin.rs` | `ThemePlugin` |

### Phase 7: Weather & Atmosphere

**Goal:** Dynamic weather mapped to system metrics.

| Step | File(s) | Description |
|------|---------|-------------|
| 7.1 | `scene/common/weather.rs` | Weather state machine, transition logic |
| 7.2 | `scene/tet/particles.rs` | Weather particle effects (rain, storm lightning) |
| 7.3 | `scene/common/atmosphere.rs` | Ambient light + fog density tied to weather |

### Phase 8: Audio

**Goal:** Ambient loops per scene, one-shot UI sounds.

| Step | File(s) | Description |
|------|---------|-------------|
| 8.1 | `audio/events.rs` | `AudioCommand` event enum |
| 8.2 | `audio/manager.rs` | `AudioManager` resource |
| 8.3 | `audio/plugin.rs` | `AudioPlugin` (bevy_kira_audio integration) |

### Phase 9: Polish & Optimization

| Step | Description |
|------|-------------|
| 9.1 | Frustum culling: only render entities in camera view |
| 9.2 | LOD: distant agents → point sprites, near agents → meshes |
| 9.3 | Entity aggregation: 1000 agents → 1 swarm particle cloud |
| 9.4 | Minimap rendering (bevy_prototype_lyon in egui or render-to-texture) |
| 9.5 | Time-of-day: real clock → ambient light cycle |

---

## Dependency Graph (Build Order)

```
constants.rs ◄── (everything depends on this)
    │
models.rs (unchanged)
    │
bridge/ ◄── models.rs
    │
world/ ◄── bridge/, models.rs, constants.rs
    │
camera/ ◄── constants.rs
    │
scene/ ◄── world/, camera/, constants.rs (+ bevy_hanabi)
    │
picking/ ◄── world/, camera/
    │
theme/ ◄── constants.rs (+ bevy_egui)
    │
hud/ ◄── world/, theme/, bridge/ (+ bevy_egui)
    │
audio/ ◄── scene/, constants.rs (+ bevy_kira_audio)
    │
diagnostics/ ◄── (standalone)
    │
main.rs ◄── all plugins
```

## Plugin Registration Order (`main.rs`)

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin { ... })
            .disable::<bevy::audio::AudioPlugin>())
        .add_plugins(EguiPlugin)
        .add_plugins(HanabiPlugin)
        .add_plugins(bevy_kira_audio::AudioPlugin)
        .add_plugins(TweeningPlugin)
        // Custom plugins
        .add_plugins(DiagnosticsPlugin)
        .add_plugins(ThemePlugin)
        .add_plugins(BridgePlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(ScenePlugin)
        .add_plugins(PickingPlugin)
        .add_plugins(HudPlugin)
        .add_plugins(DeerAudioPlugin)
        .run();
}
```

---

## What's Preserved from Current Codebase

| File | Status |
|------|--------|
| `models.rs` | **Kept as-is** (97 lines, clean domain types) |
| `bridge.rs` | **Refactored** into `bridge/` module (same logic, 4 files < 400 LOC each) |
| `app.rs` | **Replaced entirely** (759 lines → distributed across `hud/`, `camera/`, `world/`) |
| `main.rs` | **Rewritten** (eframe → Bevy App builder) |
| `config.yaml` | **Kept as-is** (Python bridge config) |
| `python/bridge.py` | **Kept as-is** (bridge protocol unchanged) |

## What's Deferred (Future Phases)

- Precursors scene (river valley, stone watchtower)
- Descent scene (drop-ship, cloud layers)
- Artifact Graph visualization (node-based lineage browser)
- Drag-and-drop artifacts onto command console
- File dialog replacement (rfd → native Bevy/egui file picker)
- Dark/light theme toggle (Daylight Lab theme)
- Seasons system
- WASM/web build support

---

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| bevy_hanabi compute shaders not supported on all GPUs | Graceful fallback to simple sprite-based particles |
| 3D asset pipeline complexity | Start with procedural meshes (Bevy primitives) — no external 3D assets |
| Bridge thread safety with Bevy's ECS | crossbeam-channel is thread-safe; `BridgeClientResource` wrapped in `NonSend` if needed |
| HUD z-fighting with 3D world | bevy_egui renders in its own pass after 3D — no z-fighting by design |
| File count / module count feels large | Each file is focused and < 400 LOC per AGENTS.md |
