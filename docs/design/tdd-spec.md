# Deer GUI: TDD Specification & API Contracts

This document defines **every interface, trait, struct, enum, and test case** for the Bevy 0.18.1
Cinematic Control Center. Code must conform to these contracts. Tests must be written before or
alongside implementation (TDD per AGENTS.md).

---

## Table of Contents

1. [Module: `constants`](#1-module-constants)
2. [Module: `models`](#2-module-models)
3. [Module: `bridge`](#3-module-bridge)
4. [Module: `world`](#4-module-world)
5. [Module: `camera`](#5-module-camera)
6. [Module: `scene`](#6-module-scene)
7. [Module: `hud`](#7-module-hud)
8. [Module: `theme`](#8-module-theme)
9. [Module: `audio`](#9-module-audio)
10. [Module: `picking`](#10-module-picking)
11. [Module: `diagnostics`](#11-module-diagnostics)
12. [Cross-Cutting Concerns](#12-cross-cutting-concerns)
13. [Test Matrix](#13-test-matrix)

---

## 1. Module: `constants`

**File:** `src/constants.rs`  
**Purpose:** Single source of truth for all magic numbers, colors, timings, and thresholds.  
**Coupling:** None (leaf dependency).  
**LOC Budget:** ~80

### Contract

```rust
// ─── Z-Depth Layers ───
pub const Z_WORLD_FAR: f32 = -1000.0;
pub const Z_WORLD_MID: f32 = -500.0;
pub const Z_WORLD_NEAR: f32 = -100.0;
pub const Z_HUD: f32 = 100.0;

// ─── Camera ───
pub const CAMERA_DEFAULT_YAW: f32 = 0.0;
pub const CAMERA_DEFAULT_PITCH: f32 = -15.0;
pub const CAMERA_DEFAULT_ZOOM: f32 = 1.0;
pub const CAMERA_MIN_ZOOM: f32 = 0.5;
pub const CAMERA_MAX_ZOOM: f32 = 5.0;
pub const CAMERA_MIN_PITCH: f32 = -60.0;
pub const CAMERA_MAX_PITCH: f32 = 10.0;
pub const CAMERA_INTERPOLATION_SPEED: f32 = 4.0;
pub const CAMERA_SHAKE_DECAY: f32 = 5.0;
pub const CAMERA_YAW_SENSITIVITY: f32 = 0.3;
pub const CAMERA_PITCH_SENSITIVITY: f32 = 0.2;
pub const CAMERA_ZOOM_SENSITIVITY: f32 = 0.1;
pub const CAMERA_ORBIT_RADIUS: f32 = 200.0;

// ─── Timing ───
pub const BRIDGE_POLL_INTERVAL_MS: u64 = 16;
pub const EVENT_TICKER_FADE_SECS: f32 = 8.0;
pub const EVENT_TICKER_MAX_ENTRIES: usize = 50;
pub const WEATHER_TRANSITION_SECS: f32 = 3.0;
pub const BEACON_PULSE_HZ: f32 = 2.0;

// ─── Visual ───
pub const AGENT_PARTICLE_SIZE: f32 = 0.3;
pub const HUD_PANEL_ALPHA: f32 = 0.75;
pub const HUD_PANEL_ROUNDING: f32 = 8.0;
pub const MAX_VISIBLE_AGENTS: usize = 500;
pub const TET_STRUCTURE_RADIUS: f32 = 50.0;
pub const TET_GLOW_MIN: f32 = 0.3;
pub const TET_GLOW_MAX: f32 = 1.0;
pub const STARFIELD_COUNT: usize = 2000;
pub const STARFIELD_RADIUS: f32 = 800.0;
pub const DATA_TRAIL_SPEED: f32 = 5.0;
pub const DATA_TRAIL_COUNT: usize = 100;

// ─── Window ───
pub const WINDOW_TITLE: &str = "Deer GUI \u{2014} Control Center";
pub const WINDOW_DEFAULT_WIDTH: f32 = 1480.0;
pub const WINDOW_DEFAULT_HEIGHT: f32 = 920.0;
pub const WINDOW_MIN_WIDTH: f32 = 1080.0;
pub const WINDOW_MIN_HEIGHT: f32 = 760.0;

// ─── Weather Thresholds ───
pub const WEATHER_FOGGY_LATENCY_MS: f32 = 500.0;
pub const WEATHER_RAINY_LOAD_PCT: f32 = 0.7;
pub const WEATHER_STORMY_ERROR_RATE: f32 = 0.1;

// ─── Aggregation ───
pub const SWARM_AGGREGATION_THRESHOLD: u32 = 10;
pub const SWARM_VISUAL_RATIO: f32 = 0.05; // 100 agents → 5 visible boats
```

### Tests

- **T-CONST-01:** Verify `CAMERA_MIN_ZOOM < CAMERA_DEFAULT_ZOOM < CAMERA_MAX_ZOOM`.
- **T-CONST-02:** Verify `CAMERA_MIN_PITCH < CAMERA_DEFAULT_PITCH < CAMERA_MAX_PITCH`.
- **T-CONST-03:** Verify `Z_WORLD_FAR < Z_WORLD_MID < Z_WORLD_NEAR < Z_HUD`.
- **T-CONST-04:** Verify all `f32` constants are finite (not NaN or Infinity).

---

## 2. Module: `models`

**File:** `src/models.rs`  
**Purpose:** Shared domain types for bridge communication. Serializable with serde.  
**Coupling:** None (leaf dependency). Used by `bridge`, `world`, `hud`.  
**LOC Budget:** ~120 (current: 97, allow growth)  
**Status:** KEEP AS-IS. Already correctly implemented.

### Contract (Existing — No Changes)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThreadSummary {
    pub thread_id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub message_count: usize,
    pub artifacts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TodoItem {
    pub content: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Attachment {
    pub filename: String,
    pub size: Option<u64>,
    pub path: Option<String>,
    pub artifact_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolCall {
    pub id: Option<String>,
    pub name: String,
    pub args: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Usage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub created_at: String,
    pub name: Option<String>,
    pub tool_calls: Vec<ToolCall>,
    pub attachments: Vec<Attachment>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThreadRecord {
    pub thread_id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub messages: Vec<ChatMessage>,
    pub artifacts: Vec<String>,
    pub todos: Vec<TodoItem>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelInfo {
    pub name: String,
    pub display_name: Option<String>,
    pub model: Option<String>,
    pub supports_thinking: Option<bool>,
    pub supports_reasoning_effort: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppStateUpdate {
    pub thread_id: String,
    pub title: Option<String>,
    pub artifacts: Vec<String>,
    pub todos: Vec<TodoItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArtifactInfo {
    pub thread_id: String,
    pub virtual_path: String,
    pub host_path: String,
    pub mime_type: String,
}
```

### Tests

- **T-MODEL-01:** Round-trip serde: `ThreadSummary` → JSON → `ThreadSummary` preserves all fields.
- **T-MODEL-02:** Round-trip serde: `ChatMessage` with all optional fields populated.
- **T-MODEL-03:** Round-trip serde: `ChatMessage` with all optional fields as `None`/empty.
- **T-MODEL-04:** `Default::default()` produces valid (non-panicking) instances for all types.
- **T-MODEL-05:** Deserialize a real bridge JSON payload into `ThreadRecord`.

---

## 3. Module: `bridge`

**Files:** `src/bridge/{mod.rs, client.rs, commands.rs, events.rs, plugin.rs}`  
**Purpose:** Python IPC bridge. Spawns subprocess, sends JSON commands, receives events.  
**Coupling:** Depends on `models`. Exposes `BridgePlugin` to Bevy.  
**LOC Budget:** ~400 total (4 files × ~100)

### 3.1 Events (`bridge/events.rs`)

```rust
/// Inbound events from the Python bridge process.
#[derive(Debug, Clone)]
pub enum BridgeEvent {
    Ready,
    Response { request_id: String, response: BridgeResponse },
    StreamMessage { thread_id: String, message: ChatMessage },
    State { state: AppStateUpdate },
    Suggestions { thread_id: String, suggestions: Vec<String> },
    Done { thread_id: String, usage: Option<Usage> },
    Error { message: String },
    BridgeExited,
}

/// Structured responses to specific commands.
#[derive(Debug, Clone)]
pub enum BridgeResponse {
    Threads(Vec<ThreadSummary>),
    Thread(ThreadRecord),
    CreatedThread(ThreadRecord),
    Models(Vec<ModelInfo>),
    Renamed(ThreadSummary),
    Deleted(String),
    Artifact(ArtifactInfo),
    Ack,
    Raw(serde_json::Value),
}
```

### 3.2 Commands (`bridge/commands.rs`)

```rust
/// Typed outbound commands to the Python bridge.
#[derive(Debug, Clone, Serialize)]
pub struct BridgeCommand {
    pub id: String,
    pub command: String,
    pub payload: serde_json::Value,
}

impl BridgeCommand {
    pub fn list_threads() -> Self;
    pub fn get_thread(thread_id: &str) -> Self;
    pub fn create_thread() -> Self;
    pub fn rename_thread(thread_id: &str, title: &str) -> Self;
    pub fn delete_thread(thread_id: &str) -> Self;
    pub fn list_models() -> Self;
    pub fn send_message(
        thread_id: &str,
        text: &str,
        attachments: &[PathBuf],
        model_name: Option<&str>,
        mode: &str,
        reasoning_effort: Option<&str>,
    ) -> Self;
    pub fn resolve_artifact(thread_id: &str, virtual_path: &str) -> Self;
}
```

### 3.3 Client (`bridge/client.rs`)

```rust
pub struct BridgeClient {
    stdin: Arc<Mutex<ChildStdin>>,
    events_rx: Receiver<BridgeEvent>,
    _child: Child,
}

impl BridgeClient {
    /// Spawn the Python bridge subprocess. Returns error if Python not found.
    pub fn spawn() -> Result<Self, BridgeError>;

    /// Access the event receiver (crossbeam channel).
    pub fn events(&self) -> &Receiver<BridgeEvent>;

    /// Send a typed command. Returns the request ID.
    pub fn send(&self, command: BridgeCommand) -> Result<String, BridgeError>;
}

/// Typed bridge errors (replaces String errors).
#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("Python binary not found: {0}")]
    PythonNotFound(String),
    #[error("Failed to spawn bridge process: {0}")]
    SpawnFailed(String),
    #[error("Failed to write to bridge stdin: {0}")]
    WriteFailed(String),
    #[error("Bridge process exited unexpectedly")]
    ProcessExited,
    #[error("Serialization error: {0}")]
    SerializationError(String),
}
```

### 3.4 Plugin (`bridge/plugin.rs`)

```rust
pub struct BridgePlugin;

impl Plugin for BridgePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BridgeClientResource(None))
           .add_event::<BridgeEventReceived>()
           .add_systems(Startup, bridge_startup_system)
           .add_systems(Update, bridge_poll_system);
    }
}

#[derive(Resource)]
pub struct BridgeClientResource(pub Option<BridgeClient>);

#[derive(Event, Debug)]
pub struct BridgeEventReceived(pub BridgeEvent);

/// Startup: spawn BridgeClient, store in resource.
fn bridge_startup_system(mut res: ResMut<BridgeClientResource>) { ... }

/// Update: drain crossbeam channel → EventWriter<BridgeEventReceived>.
/// Max events per frame: 64 (prevent frame stalls).
fn bridge_poll_system(
    res: Res<BridgeClientResource>,
    mut writer: EventWriter<BridgeEventReceived>,
) { ... }
```

### Tests

- **T-BRIDGE-01:** `BridgeCommand::list_threads()` produces valid JSON with `command: "list_threads"`.
- **T-BRIDGE-02:** `BridgeCommand::send_message(...)` includes all fields in payload.
- **T-BRIDGE-03:** `BridgeCommand` request IDs are unique UUIDs.
- **T-BRIDGE-04:** Deserialize a mock Python JSON line `{"kind":"ready"}` into `BridgeEvent::Ready`.
- **T-BRIDGE-05:** Deserialize `{"kind":"response","request_id":"...","data":{"threads":[...]}}` → `BridgeResponse::Threads`.
- **T-BRIDGE-06:** Deserialize `{"kind":"event","event":"stream_message",...}` → `BridgeEvent::StreamMessage`.
- **T-BRIDGE-07:** `BridgeError` variants produce meaningful error messages via `Display`.
- **T-BRIDGE-08:** (Integration) `BridgePlugin` registers resource and event type in test `App`.
- **T-BRIDGE-09:** (Integration) `bridge_poll_system` with no client does not panic.
- **T-BRIDGE-10:** (Integration) `bridge_poll_system` caps at 64 events per frame.

---

## 4. Module: `world`

**Files:** `src/world/{mod.rs, state.rs, components.rs, spatial.rs, systems.rs, plugin.rs}`  
**Purpose:** The persistent data-driven simulation. Entities exist whether viewed or not.  
**Coupling:** Depends on `models`, `bridge/events`, `constants`.  
**LOC Budget:** ~500 total (5 files × ~100)

### 4.1 Components (`world/components.rs`)

```rust
/// Unique identifier for world entities (maps to bridge thread/agent IDs).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityId(pub String);

/// The type and state of a world entity.
#[derive(Debug, Clone, PartialEq)]
pub enum WorldEntityType {
    Agent(AgentState),
    Swarm { agent_count: u32 },
    Mission { progress: f32, title: String },
    Artifact { kind: ArtifactKind, path: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AgentState {
    Idle,
    Working,
    Error,
    ApprovalNeeded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArtifactKind {
    Document,
    Index,
    Embedding,
    Raw,
}

/// Core component linking a Bevy entity to the world simulation.
#[derive(Component, Debug)]
pub struct WorldEntity {
    pub entity_id: EntityId,
    pub entity_type: WorldEntityType,
}

// ─── Marker Components (for efficient queries) ───
#[derive(Component, Debug, Default)] pub struct AgentMarker;
#[derive(Component, Debug, Default)] pub struct SwarmMarker;
#[derive(Component, Debug, Default)] pub struct MissionMarker;
#[derive(Component, Debug, Default)] pub struct ArtifactMarker;

// ─── Interaction Components ───
#[derive(Component, Debug, Default)] pub struct Selectable;
#[derive(Component, Debug, Default)] pub struct Selected;

/// Pulsing beacon for HITL approval-needed entities.
#[derive(Component, Debug)]
pub struct PulsingBeacon {
    pub frequency_hz: f32,
    pub phase: f32,
}
```

### 4.2 State (`world/state.rs`)

```rust
/// Global world state resource. Updated from bridge events.
#[derive(Resource, Debug, Default)]
pub struct WorldState {
    /// Map from EntityId to Bevy Entity for lookup.
    pub entity_map: HashMap<EntityId, Entity>,
    /// Aggregated system health metrics.
    pub system_health: SystemHealth,
}

/// System-wide health metrics driving atmospheric effects.
#[derive(Debug, Clone, Default)]
pub struct SystemHealth {
    pub active_agents: u32,
    pub retrying_agents: u32,
    pub failed_agents: u32,
    pub tokens_per_sec: f32,
    pub cost_per_hour: f32,
    pub nats_saturation: f32,      // 0.0..1.0
    pub state_server_lag_ms: f32,
    pub error_rate: f32,           // 0.0..1.0
    pub system_online: bool,
}

impl WorldState {
    /// Register a new entity mapping.
    pub fn register(&mut self, id: EntityId, entity: Entity);
    /// Remove an entity mapping. Returns the Bevy Entity if it existed.
    pub fn unregister(&mut self, id: &EntityId) -> Option<Entity>;
    /// Look up a Bevy Entity by world ID.
    pub fn lookup(&self, id: &EntityId) -> Option<Entity>;
    /// Total entity count.
    pub fn entity_count(&self) -> usize;
}
```

### 4.3 Spatial Index (`world/spatial.rs`)

```rust
/// Grid-based spatial index for efficient 3D entity queries.
pub struct SpatialIndex {
    cell_size: f32,
    cells: HashMap<(i32, i32, i32), Vec<Entity>>,
}

impl SpatialIndex {
    pub fn new(cell_size: f32) -> Self;

    /// Insert an entity at a position.
    pub fn insert(&mut self, entity: Entity, position: Vec3);

    /// Remove an entity from the index.
    pub fn remove(&mut self, entity: Entity, position: Vec3);

    /// Update an entity's position (remove old, insert new).
    pub fn update(&mut self, entity: Entity, old_pos: Vec3, new_pos: Vec3);

    /// Query all entities within a sphere.
    pub fn query_sphere(&self, center: Vec3, radius: f32) -> Vec<Entity>;

    /// Ray intersection: find the closest entity along a ray.
    /// Returns (Entity, distance) or None.
    pub fn raycast(
        &self,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
        entity_radius: f32,
    ) -> Option<(Entity, f32)>;

    /// Clear the entire index.
    pub fn clear(&mut self);

    /// Number of entities tracked.
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

### 4.4 Systems (`world/systems.rs`)

```rust
/// Process bridge events and spawn/update/despawn world entities.
pub fn world_event_handler_system(
    mut commands: Commands,
    mut world_state: ResMut<WorldState>,
    mut events: EventReader<BridgeEventReceived>,
    mut spatial: ResMut<SpatialIndex>,
);

/// Update system health metrics from bridge state events.
pub fn system_health_update_system(
    mut world_state: ResMut<WorldState>,
    mut events: EventReader<BridgeEventReceived>,
);

/// Animate pulsing beacons (agents needing approval).
pub fn beacon_pulse_system(
    time: Res<Time>,
    mut query: Query<(&PulsingBeacon, &mut Transform), With<Selectable>>,
);
```

### 4.5 Plugin (`world/plugin.rs`)

```rust
pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldState>()
           .insert_resource(SpatialIndex::new(10.0))
           .add_systems(Update, (
               world_event_handler_system,
               system_health_update_system,
               beacon_pulse_system,
           ));
    }
}
```

### Tests

- **T-WORLD-01:** `WorldState::register` + `lookup` returns the correct entity.
- **T-WORLD-02:** `WorldState::unregister` removes and returns the entity.
- **T-WORLD-03:** `WorldState::unregister` on non-existent ID returns `None`.
- **T-WORLD-04:** `WorldState::entity_count` reflects registrations and removals.
- **T-WORLD-05:** `SpatialIndex::insert` + `query_sphere` finds entity within radius.
- **T-WORLD-06:** `SpatialIndex::query_sphere` does NOT return entities outside radius.
- **T-WORLD-07:** `SpatialIndex::raycast` finds closest entity along ray direction.
- **T-WORLD-08:** `SpatialIndex::raycast` returns `None` when no entities hit.
- **T-WORLD-09:** `SpatialIndex::update` moves entity between cells correctly.
- **T-WORLD-10:** `SpatialIndex::remove` + `query_sphere` no longer finds entity.
- **T-WORLD-11:** `AgentState` equality: `AgentState::Idle == AgentState::Idle`.
- **T-WORLD-12:** `EntityId` hash consistency for HashMap usage.
- **T-WORLD-13:** (Integration) `WorldPlugin` registers `WorldState` and `SpatialIndex` resources.
- **T-WORLD-14:** (Integration) `beacon_pulse_system` modifies transform scale based on time.

---

## 5. Module: `camera`

**Files:** `src/camera/{mod.rs, components.rs, systems.rs, plugin.rs}`  
**Purpose:** 3D camera with yaw/pitch/zoom, smooth interpolation, shake, and entity focus.  
**Coupling:** Depends on `constants`.  
**LOC Budget:** ~300 total (3 files × ~100)

### 5.1 Components (`camera/components.rs`)

```rust
/// Main camera state component. Attached to the 3D Camera entity.
#[derive(Component, Debug)]
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

impl Default for CinematicCamera {
    fn default() -> Self {
        Self {
            yaw_deg: CAMERA_DEFAULT_YAW,
            target_yaw: CAMERA_DEFAULT_YAW,
            pitch_deg: CAMERA_DEFAULT_PITCH,
            target_pitch: CAMERA_DEFAULT_PITCH,
            zoom: CAMERA_DEFAULT_ZOOM,
            target_zoom: CAMERA_DEFAULT_ZOOM,
            shake: 0.0,
            focus_target: None,
        }
    }
}

impl CinematicCamera {
    /// Calculate the world-space position from yaw, pitch, zoom, and orbit radius.
    pub fn compute_position(&self, orbit_radius: f32) -> Vec3;

    /// Calculate the look-at target (always origin or focus_target).
    pub fn compute_look_at(&self) -> Vec3;

    /// Apply a shake impulse (decays over time).
    pub fn add_shake(&mut self, amplitude: f32);
}
```

### 5.2 Systems (`camera/systems.rs`)

```rust
/// Pure function: interpolate current toward target at speed * dt.
/// Returns the new value, clamped to [min, max].
pub fn smooth_interpolate(current: f32, target: f32, speed: f32, dt: f32) -> f32;

/// Pure function: compute camera Transform from CinematicCamera state.
pub fn compute_camera_transform(camera: &CinematicCamera, orbit_radius: f32) -> Transform;

/// System: read mouse drag → update target_yaw, target_pitch.
pub fn camera_input_system(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut scroll: EventReader<MouseWheel>,
    mut query: Query<&mut CinematicCamera>,
);

/// System: interpolate yaw/pitch/zoom toward targets, apply to Transform.
pub fn camera_interpolation_system(
    time: Res<Time>,
    mut query: Query<(&mut CinematicCamera, &mut Transform)>,
);

/// System: decay shake over time.
pub fn camera_shake_system(
    time: Res<Time>,
    mut query: Query<(&mut CinematicCamera, &mut Transform)>,
);

/// System: when focus_target is set, smoothly rotate camera to face it.
pub fn camera_focus_system(
    time: Res<Time>,
    mut query: Query<&mut CinematicCamera>,
);
```

### 5.3 Plugin (`camera/plugin.rs`)

```rust
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, camera_spawn_system)
           .add_systems(Update, (
               camera_input_system,
               camera_interpolation_system,
               camera_shake_system,
               camera_focus_system,
           ).chain());
    }
}

/// Startup: spawn Camera3d with CinematicCamera, Transform, ambient light.
fn camera_spawn_system(mut commands: Commands);
```

### Tests

- **T-CAM-01:** `smooth_interpolate(0.0, 10.0, 1.0, 1.0)` returns value between 0 and 10.
- **T-CAM-02:** `smooth_interpolate(5.0, 5.0, *, *)` returns 5.0 (no change when at target).
- **T-CAM-03:** `smooth_interpolate` respects clamping: result stays within `[min, max]`.
- **T-CAM-04:** `CinematicCamera::default()` uses values from `constants`.
- **T-CAM-05:** `compute_camera_transform` at yaw=0, pitch=0, zoom=1 produces expected position.
- **T-CAM-06:** `compute_camera_transform` at yaw=90 produces position rotated 90 degrees.
- **T-CAM-07:** `add_shake(1.0)` sets shake to 1.0; after decay it approaches 0.
- **T-CAM-08:** `compute_position` with varying zoom changes distance from origin.
- **T-CAM-09:** (Integration) `CameraPlugin` spawns exactly one entity with `CinematicCamera`.
- **T-CAM-10:** (Integration) `camera_interpolation_system` moves camera toward target over multiple frames.
- **T-CAM-11:** (Integration) `camera_input_system` with no mouse input does not change targets.

---

## 6. Module: `scene`

**Files:** `src/scene/{mod.rs, traits.rs, plugin.rs, common/*, tet/*}`  
**Purpose:** Cinematic scene rendering — TET Orchestrator (default), extensible for future scenes.  
**Coupling:** Depends on `world`, `camera`, `constants`. Uses `bevy_hanabi`.  
**LOC Budget:** ~600 total (8 files × ~75)

### 6.1 Traits (`scene/traits.rs`)

```rust
/// Configuration interface for a cinematic scene.
/// Each scene (TET, Precursors, Descent) implements this trait.
pub trait SceneConfig: Send + Sync + 'static {
    /// Human-readable scene name.
    fn name(&self) -> &str;

    /// Spawn the static environment (meshes, lights, skybox elements).
    fn spawn_environment(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    );

    /// Return a bevy_hanabi EffectAsset for an agent in the given state.
    fn particle_config_for_agent(&self, state: &AgentState) -> EffectAsset;

    /// Map system health metrics to atmospheric rendering state.
    fn map_health_to_atmosphere(&self, health: &SystemHealth) -> AtmosphereState;

    /// Asset path for the ambient audio loop.
    fn ambient_audio_track(&self) -> &str;
}

/// Atmospheric state derived from system health.
#[derive(Debug, Clone, PartialEq)]
pub struct AtmosphereState {
    pub ambient_color: Color,
    pub ambient_intensity: f32,
    pub fog_density: f32,
    pub fog_color: Color,
}
```

### 6.2 Weather (`scene/common/weather.rs`)

```rust
/// Weather states driven by system metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeatherState {
    #[default]
    Clear,
    Foggy,
    Rainy,
    Stormy,
}

/// Weather state machine resource.
#[derive(Resource, Debug)]
pub struct WeatherMachine {
    pub current: WeatherState,
    pub target: WeatherState,
    pub transition_progress: f32, // 0.0..1.0
}

impl WeatherMachine {
    pub fn new() -> Self;

    /// Evaluate system health and determine target weather.
    /// Pure function: no side effects.
    pub fn evaluate_target(health: &SystemHealth) -> WeatherState;

    /// Advance the transition toward the target. Returns true when complete.
    pub fn advance(&mut self, dt: f32) -> bool;

    /// Get interpolated atmosphere between current and target.
    pub fn interpolated_state(&self) -> WeatherState;
}
```

### 6.3 Parallax (`scene/common/parallax.rs`)

```rust
/// Marker component for parallax layers.
#[derive(Component, Debug)]
pub struct ParallaxLayer {
    pub depth: f32,        // multiplier: 0.0 = fixed, 1.0 = moves with camera
    pub base_offset: Vec3, // original position
}

/// System: update parallax layer positions based on camera yaw.
pub fn parallax_update_system(
    camera_query: Query<&CinematicCamera>,
    mut layer_query: Query<(&ParallaxLayer, &mut Transform)>,
);

/// Pure function: compute parallax offset from yaw and depth.
pub fn compute_parallax_offset(yaw_deg: f32, depth: f32, orbit_radius: f32) -> Vec3;
```

### 6.4 Atmosphere (`scene/common/atmosphere.rs`)

```rust
/// System: update ambient light based on weather and system health.
pub fn atmosphere_update_system(
    weather: Res<WeatherMachine>,
    world_state: Res<WorldState>,
    mut ambient: ResMut<AmbientLight>,
);
```

### 6.5 TET Scene (`scene/tet/`)

#### Config (`scene/tet/config.rs`)

```rust
pub struct TetSceneConfig;

impl SceneConfig for TetSceneConfig {
    fn name(&self) -> &str { "TET Orchestrator" }
    fn spawn_environment(...) { /* stars, TET monolith, data trails */ }
    fn particle_config_for_agent(&self, state: &AgentState) -> EffectAsset { ... }
    fn map_health_to_atmosphere(&self, health: &SystemHealth) -> AtmosphereState { ... }
    fn ambient_audio_track(&self) -> &str { "audio/tet_ambient.ogg" }
}
```

#### Setup (`scene/tet/setup.rs`)

```rust
/// Marker components for TET scene entities.
#[derive(Component)] pub struct TetStructure;
#[derive(Component)] pub struct Star;
#[derive(Component)] pub struct DataTrail;

/// Startup system: spawn the TET environment.
pub fn tet_setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
);
```

#### Systems (`scene/tet/systems.rs`)

```rust
/// Animate TET structure glow based on system health.
pub fn tet_glow_system(
    world_state: Res<WorldState>,
    mut query: Query<&mut MeshMaterial3d<StandardMaterial>, With<TetStructure>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
);

/// Animate data trail particles.
pub fn data_trail_system(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DataTrail>>,
);

/// Animate star parallax.
pub fn star_parallax_system(
    camera_query: Query<&CinematicCamera>,
    mut star_query: Query<(&ParallaxLayer, &mut Transform), With<Star>>,
);
```

#### Particles (`scene/tet/particles.rs`)

```rust
/// Create a bevy_hanabi EffectAsset for agent "moths" orbiting the TET.
pub fn agent_moth_effect(state: &AgentState) -> EffectAsset;

/// Create a bevy_hanabi EffectAsset for data stream trails.
pub fn data_stream_effect() -> EffectAsset;

/// Create a bevy_hanabi EffectAsset for weather rain.
pub fn rain_effect() -> EffectAsset;

/// Create a bevy_hanabi EffectAsset for storm lightning flash.
pub fn lightning_effect() -> EffectAsset;
```

### 6.6 Scene Plugin (`scene/plugin.rs`)

```rust
pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WeatherMachine::new())
           .add_systems(Startup, tet_setup_system)
           .add_systems(Update, (
               tet_glow_system,
               data_trail_system,
               star_parallax_system,
               parallax_update_system,
               atmosphere_update_system,
               weather_update_system,
           ));
    }
}
```

### Tests

- **T-SCENE-01:** `WeatherMachine::evaluate_target` with error_rate=0 → `Clear`.
- **T-SCENE-02:** `WeatherMachine::evaluate_target` with error_rate=0.15 → `Stormy`.
- **T-SCENE-03:** `WeatherMachine::evaluate_target` with lag_ms=600 → `Foggy`.
- **T-SCENE-04:** `WeatherMachine::evaluate_target` with load_pct=0.8 → `Rainy`.
- **T-SCENE-05:** `WeatherMachine::advance` with dt > transition time sets progress to 1.0.
- **T-SCENE-06:** `WeatherMachine::advance` returns `true` when transition complete.
- **T-SCENE-07:** `compute_parallax_offset` at yaw=0 returns zero offset.
- **T-SCENE-08:** `compute_parallax_offset` at yaw=90, depth=1.0 returns non-zero offset.
- **T-SCENE-09:** `compute_parallax_offset` at depth=0 always returns zero (fixed layer).
- **T-SCENE-10:** `TetSceneConfig::map_health_to_atmosphere` with healthy metrics → bright ambient.
- **T-SCENE-11:** `TetSceneConfig::map_health_to_atmosphere` with high error rate → dark ambient.
- **T-SCENE-12:** `agent_moth_effect` for `AgentState::Working` → non-zero particle spawn rate.
- **T-SCENE-13:** `agent_moth_effect` for `AgentState::Idle` → low/zero spawn rate.
- **T-SCENE-14:** (Integration) `ScenePlugin` spawns TET structure entity.
- **T-SCENE-15:** (Integration) `tet_setup_system` spawns exactly `STARFIELD_COUNT` star entities.

---

## 7. Module: `hud`

**Files:** `src/hud/{mod.rs, styles.rs, top_bar.rs, left_panel.rs, right_inspector.rs,
bottom_console.rs, event_ticker.rs, center_canvas.rs, modal.rs, plugin.rs}`  
**Purpose:** HUD overlay (Z1) via `bevy_egui`. Semi-transparent glass panels.  
**Coupling:** Depends on `world/state`, `theme`, `bridge`. Uses `bevy_egui`.  
**LOC Budget:** ~900 total (9 files × ~100)

### 7.1 HUD State (`hud/mod.rs` re-exports)

```rust
/// Global HUD state resource.
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
    pub command_mode: CommandMode,
    pub center_mode: CenterCanvasMode,
    pub show_modal: Option<ModalKind>,

    // Thread/chat state (migrated from old app.rs)
    pub threads: Vec<ThreadSummary>,
    pub selected_thread_id: Option<String>,
    pub thread_cache: HashMap<String, ThreadRecord>,
    pub models: Vec<ModelInfo>,
    pub selected_model: Option<String>,
    pub streaming_thread_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MissionSummary {
    pub id: String,
    pub title: String,
    pub progress: f32,
    pub status: MissionStatus,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MissionStatus { Active, Blocked, Idle, Completed }

#[derive(Debug, Clone)]
pub struct EntityInspectorData {
    pub entity_id: EntityId,
    pub entity_type: WorldEntityType,
    pub position: Vec3,
    pub details: InspectorDetails,
}

#[derive(Debug, Clone)]
pub enum InspectorDetails {
    Agent { tokens: u64, context_files: u32, pending_action: Option<String> },
    Mission { workflow_count: u32, artifact_count: u32 },
    Artifact { path: String, mime_type: String, size_bytes: u64 },
}

#[derive(Debug, Clone)]
pub struct EventLogEntry {
    pub timestamp: f64,    // seconds since app start (from Time resource)
    pub severity: EventSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventSeverity { Info, Warning, Error }

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum CommandMode {
    #[default]
    Direct,
    Brainstorm,
    Query,
    Halt,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum CenterCanvasMode {
    #[default]
    WorldView,
    ExpertsMeeting,
    SwarmMonitor,
    ArtifactGraph,
    Forensics,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModalKind {
    Settings,
    AgentTuning,
    ArtifactLineage,
}
```

### 7.2 Styles (`hud/styles.rs`)

```rust
/// Create an egui Frame with glass panel styling.
pub fn glass_panel_frame(alpha: f32, rounding: f32) -> egui::Frame;

/// Create an egui Frame for the top bar.
pub fn top_bar_frame() -> egui::Frame;

/// Create an egui Frame for side panels.
pub fn side_panel_frame() -> egui::Frame;

/// Status color for agent state.
pub fn agent_state_color(state: &AgentState) -> egui::Color32;

/// Status color for mission status.
pub fn mission_status_color(status: MissionStatus) -> egui::Color32;

/// Progress bar styling.
pub fn draw_progress_bar(ui: &mut egui::Ui, progress: f32, color: egui::Color32);
```

### 7.3 Each HUD Panel System Signature

```rust
// top_bar.rs
pub fn top_bar_system(
    mut egui_ctx: Query<&mut EguiContext>,
    hud: Res<HudState>,
    world_state: Res<WorldState>,
);

// left_panel.rs
pub fn left_panel_system(
    mut egui_ctx: Query<&mut EguiContext>,
    mut hud: ResMut<HudState>,
    mut camera_query: Query<&mut CinematicCamera>,
);

// right_inspector.rs
pub fn right_inspector_system(
    mut egui_ctx: Query<&mut EguiContext>,
    hud: Res<HudState>,
    bridge: Res<BridgeClientResource>,
);

// bottom_console.rs
pub fn bottom_console_system(
    mut egui_ctx: Query<&mut EguiContext>,
    mut hud: ResMut<HudState>,
    bridge: Res<BridgeClientResource>,
);

// event_ticker.rs
pub fn event_ticker_system(
    mut egui_ctx: Query<&mut EguiContext>,
    hud: Res<HudState>,
    time: Res<Time>,
);

// center_canvas.rs
pub fn center_canvas_system(
    mut egui_ctx: Query<&mut EguiContext>,
    hud: Res<HudState>,
);

// modal.rs
pub fn modal_system(
    mut egui_ctx: Query<&mut EguiContext>,
    mut hud: ResMut<HudState>,
);
```

### Tests

- **T-HUD-01:** `glass_panel_frame(0.75, 8.0)` produces a Frame with correct alpha.
- **T-HUD-02:** `agent_state_color(Idle)` → green-ish, `Error` → red-ish.
- **T-HUD-03:** `mission_status_color(Blocked)` → red/amber.
- **T-HUD-04:** `EventLogEntry` with timestamp 0.0 and current time 9.0 → visible (within fade window).
- **T-HUD-05:** `EventLogEntry` with timestamp 0.0 and current time 20.0 → faded out.
- **T-HUD-06:** `HudState::default()` produces valid state with empty collections.
- **T-HUD-07:** (Integration) `HudPlugin` registers `HudState` resource.
- **T-HUD-08:** (Integration) All HUD systems run without panic on empty state.

---

## 8. Module: `theme`

**Files:** `src/theme/{mod.rs, theme.rs, tet_theme.rs, plugin.rs}`  
**Purpose:** Runtime-switchable visual themes for HUD + scene atmosphere.  
**Coupling:** Depends on `constants`. Used by `hud`, `scene`.  
**LOC Budget:** ~250 total (3 files × ~83)

### 8.1 Theme Definitions (`theme/theme.rs`)

```rust
/// A complete visual theme definition.
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub panel_bg: Color,
    pub panel_border: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub accent: Color,
    pub accent_dim: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub agent_idle: Color,
    pub agent_working: Color,
    pub agent_error: Color,
    pub agent_approval: Color,
    pub egui_visuals: egui::Visuals,
}

/// Theme manager resource.
#[derive(Resource)]
pub struct ThemeManager {
    pub current: Theme,
    pub available: Vec<Theme>,
}

impl ThemeManager {
    pub fn new(default_theme: Theme) -> Self;
    pub fn switch(&mut self, name: &str) -> bool;
    pub fn current(&self) -> &Theme;
}

/// Event emitted when theme changes.
#[derive(Event)]
pub struct ThemeChanged(pub String);
```

### 8.2 TET Theme (`theme/tet_theme.rs`)

```rust
/// Create the TET Orchestrator theme.
pub fn tet_theme() -> Theme;
```

### Tests

- **T-THEME-01:** `tet_theme()` produces a `Theme` with name "TET Orchestrator".
- **T-THEME-02:** `ThemeManager::switch("nonexistent")` returns `false`.
- **T-THEME-03:** `ThemeManager::switch("TET Orchestrator")` returns `true` and updates current.
- **T-THEME-04:** All colors in `tet_theme()` are valid (alpha > 0 for visible elements).
- **T-THEME-05:** (Integration) `ThemePlugin` registers `ThemeManager` resource.
- **T-THEME-06:** (Integration) `ThemeChanged` event triggers egui `Visuals` update.

---

## 9. Module: `audio`

**Files:** `src/audio/{mod.rs, events.rs, manager.rs, plugin.rs}`  
**Purpose:** Ambient loops per scene, one-shot UI sounds via `bevy_kira_audio`.  
**Coupling:** Depends on `constants`. Standalone otherwise.  
**LOC Budget:** ~250 total (3 files × ~83)

### 9.1 Events (`audio/events.rs`)

```rust
/// Commands to the audio system.
#[derive(Event, Debug, Clone)]
pub enum AudioCommand {
    PlayAmbient { track: String, fade_in_secs: f32 },
    StopAmbient { fade_out_secs: f32 },
    PlayOneShot { sound: UiSound },
    SetMasterVolume { volume: f32 },
    Mute,
    Unmute,
}

/// One-shot UI sound identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiSound {
    Click,
    ApprovalChime,
    ErrorTone,
    AlertPing,
    CameraMove,
    MissionComplete,
}
```

### 9.2 Manager (`audio/manager.rs`)

```rust
/// Audio state resource.
#[derive(Resource)]
pub struct AudioManager {
    pub master_volume: f32,
    pub muted: bool,
    pub current_ambient: Option<String>,
}

impl AudioManager {
    pub fn new() -> Self;
    pub fn effective_volume(&self) -> f32;
}
```

### 9.3 Plugin (`audio/plugin.rs`)

```rust
pub struct DeerAudioPlugin;

impl Plugin for DeerAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioManager>()
           .add_event::<AudioCommand>()
           .add_systems(Update, audio_command_system);
    }
}

/// Process AudioCommand events using bevy_kira_audio channels.
fn audio_command_system(
    mut commands: EventReader<AudioCommand>,
    audio: Res<bevy_kira_audio::Audio>,
    manager: ResMut<AudioManager>,
    assets: Res<AssetServer>,
);
```

### Tests

- **T-AUDIO-01:** `AudioManager::new()` has volume 1.0, not muted.
- **T-AUDIO-02:** `AudioManager::effective_volume()` returns 0.0 when muted.
- **T-AUDIO-03:** `AudioManager::effective_volume()` returns `master_volume` when not muted.
- **T-AUDIO-04:** `AudioCommand::PlayAmbient` can be constructed and cloned.
- **T-AUDIO-05:** (Integration) `DeerAudioPlugin` registers `AudioManager` and `AudioCommand` event.

---

## 10. Module: `picking`

**Files:** `src/picking/{mod.rs, systems.rs, plugin.rs}`  
**Purpose:** 3D ray picking for entity selection.  
**Coupling:** Depends on `world`, `camera`.  
**LOC Budget:** ~200 total (2 files × ~100)

### 10.1 Systems (`picking/systems.rs`)

```rust
/// Event emitted when user clicks on an entity.
#[derive(Event)]
pub struct EntityClicked(pub Entity);

/// Event emitted when selection changes.
#[derive(Event)]
pub struct SelectionChanged {
    pub old: Option<Entity>,
    pub new: Option<Entity>,
}

/// System: on mouse click, raycast from camera through world,
/// query SpatialIndex, emit EntityClicked.
pub fn picking_raycast_system(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<CinematicCamera>>,
    spatial: Res<SpatialIndex>,
    mut click_events: EventWriter<EntityClicked>,
);

/// System: on EntityClicked, update Selected components.
/// Remove Selected from old, add to new. Emit SelectionChanged.
pub fn selection_update_system(
    mut commands: Commands,
    mut click_events: EventReader<EntityClicked>,
    mut selection_events: EventWriter<SelectionChanged>,
    selected_query: Query<Entity, With<Selected>>,
    selectable_query: Query<Entity, With<Selectable>>,
);

/// System: sync selected entity data to HudState.selected_entity.
pub fn selection_sync_system(
    selected_query: Query<(&WorldEntity, &Transform), With<Selected>>,
    mut hud: ResMut<HudState>,
);
```

### Tests

- **T-PICK-01:** (Integration) Clicking with no entities → no `EntityClicked` event.
- **T-PICK-02:** (Integration) `selection_update_system` adds `Selected` to clicked entity.
- **T-PICK-03:** (Integration) `selection_update_system` removes `Selected` from previously selected.
- **T-PICK-04:** (Integration) `SelectionChanged` event has correct old/new values.
- **T-PICK-05:** (Integration) `selection_sync_system` updates `HudState.selected_entity`.

---

## 11. Module: `diagnostics`

**Files:** `src/diagnostics/{mod.rs, tracing.rs, plugin.rs}`  
**Purpose:** Bevy tracing subscriber, FPS counter, debug overlay.  
**Coupling:** None (standalone).  
**LOC Budget:** ~100 total (2 files × ~50)

### Contract

```rust
pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        // Add FrameTimeDiagnosticsPlugin
        // Add EntityCountDiagnosticsPlugin
        // Configure log filter from DEER_GUI_LOG / RUST_LOG env vars
    }
}
```

### Tests

- **T-DIAG-01:** (Integration) `DiagnosticsPlugin` adds frame time diagnostics without panic.

---

## 12. Cross-Cutting Concerns

### 12.1 Error Handling

All modules use typed errors via `thiserror`. No `String` error returns.

```rust
// Example pattern used across modules:
#[derive(Debug, thiserror::Error)]
pub enum ModuleError {
    #[error("description: {0}")]
    VariantName(String),
}
```

### 12.2 Logging

Every public function includes `tracing` spans:

```rust
use bevy::log::{debug, info, warn, error, trace};

pub fn some_system(...) {
    trace!("some_system called");
    // ... logic ...
    debug!("processed {} events", count);
}
```

Log levels:
- `trace` — function entry/exit, loop iterations
- `debug` — state changes, event counts
- `info` — startup, plugin registration, major state transitions
- `warn` — recoverable errors, degraded performance
- `error` — unrecoverable errors, bridge failures

### 12.3 Resource Lifetime

All Bevy resources use `#[derive(Resource)]`. No global mutable state outside ECS.

### 12.4 System Ordering

Systems within a plugin are chained where order matters:
```rust
.add_systems(Update, (
    camera_input_system,
    camera_interpolation_system,  // must run after input
    camera_shake_system,          // must run after interpolation
).chain())
```

Cross-plugin ordering uses `SystemSet` labels where needed:
```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum DeerSets {
    BridgePoll,
    WorldUpdate,
    SceneUpdate,
    HudRender,
}
```

---

## 13. Test Matrix

### Unit Tests (Pure Functions — No Bevy App)

| ID | Module | Function Under Test | Assertion |
|----|--------|-------------------|-----------|
| T-CONST-01..04 | constants | Constant relationships | Ordering, finiteness |
| T-MODEL-01..05 | models | Serde round-trips | Field preservation |
| T-BRIDGE-01..07 | bridge | Command/event serialization | JSON structure, uniqueness |
| T-WORLD-01..12 | world | WorldState, SpatialIndex | Insert/remove/query correctness |
| T-CAM-01..08 | camera | Interpolation, transform math | Numerical correctness |
| T-SCENE-01..13 | scene | Weather FSM, parallax, atmosphere | State transitions, offsets |
| T-HUD-01..06 | hud | Style functions, fade math | Color/alpha correctness |
| T-THEME-01..04 | theme | Theme construction, switching | Name matching, color validity |
| T-AUDIO-01..04 | audio | AudioManager state | Volume, mute behavior |

### Integration Tests (Bevy Test App)

| ID | Module | What's Tested | Setup |
|----|--------|--------------|-------|
| T-BRIDGE-08..10 | bridge | Plugin registration, poll system | `App::new() + BridgePlugin` |
| T-WORLD-13..14 | world | Plugin registration, beacon pulse | `App::new() + WorldPlugin` |
| T-CAM-09..11 | camera | Spawn, interpolation, no-input | `App::new() + CameraPlugin` |
| T-SCENE-14..15 | scene | TET setup, star count | `App::new() + ScenePlugin` |
| T-HUD-07..08 | hud | Plugin registration, empty render | `App::new() + HudPlugin + EguiPlugin` |
| T-THEME-05..06 | theme | Plugin registration, theme change | `App::new() + ThemePlugin` |
| T-AUDIO-05 | audio | Plugin registration | `App::new() + DeerAudioPlugin` |
| T-PICK-01..05 | picking | Click events, selection | `App::new() + PickingPlugin + WorldPlugin` |
| T-DIAG-01 | diagnostics | Plugin registration | `App::new() + DiagnosticsPlugin` |

### Total Test Count: ~70 tests

### Test Commands

```bash
# Run all unit tests
cargo test --lib -p deer-gui

# Run all integration tests
cargo test --test '*' -p deer-gui

# Run tests for a specific module
cargo test --lib -p deer-gui world::
cargo test --lib -p deer-gui camera::
cargo test --lib -p deer-gui scene::

# Run with logs
RUST_LOG=debug cargo test --lib -p deer-gui -- --nocapture
```

---

## Appendix: Bridge JSON Protocol (Existing — Unchanged)

### Outbound (GUI → Python)

```json
{"id": "<uuid>", "command": "list_threads", "payload": {}}
{"id": "<uuid>", "command": "get_thread", "payload": {"thread_id": "..."}}
{"id": "<uuid>", "command": "create_thread", "payload": {}}
{"id": "<uuid>", "command": "send_message", "payload": {
    "thread_id": "...",
    "text": "...",
    "attachments": ["path1", "path2"],
    "model_name": "...",
    "mode": "thinking",
    "reasoning_effort": "medium"
}}
{"id": "<uuid>", "command": "resolve_artifact", "payload": {"thread_id": "...", "virtual_path": "..."}}
```

### Inbound (Python → GUI)

```json
{"kind": "ready"}
{"kind": "response", "request_id": "...", "data": {"threads": [...]}}
{"kind": "response", "request_id": "...", "data": {"thread": {...}}}
{"kind": "event", "event": "stream_message", "thread_id": "...", "message": {...}}
{"kind": "event", "event": "state", "state": {...}}
{"kind": "event", "event": "suggestions", "thread_id": "...", "suggestions": [...]}
{"kind": "event", "event": "done", "thread_id": "...", "usage": {...}}
{"kind": "event", "event": "error", "message": "..."}
```
