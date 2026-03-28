# Deer GUI: Cinematic Control Center

`apps/deer_gui` is **not** a traditional desktop app.  
It is a **Cinematic Game Engine** where the primary data stream is AI research orchestration.

**The Core Paradigm:**
> There is a World beyond the screen.  
> The Camera is your window into that World.  
> What you see on the screen are just HUD elements overlaid on the Camera.

## 1. Architectural Overview: Engine World → Camera → HUD

The `DeerGuiApp` will run a **game loop**. It will render a **World** full of Entities (Agents, Swarms, Missions, Artifacts) viewed through a **Camera**, and overlay a **HUD** (Heads-Up Display) for interaction.

### Layer Stack (from back to front)

```
┌─────────────────────────────────────────────────────┐
│ [Z0] World (The "Game World")                       │
│   - Scene (TET, Precursors, Descent)               │
│   - Entities (Agents, Artifacts, Events)           │
│   - Camera (Yaw, Zoom, Focus)                      │
│   - Atmospheric Effects                            │
├─────────────────────────────────────────────────────┤
│ [Z1] HUD Overlay (Semi-transparent glass panels)    │
│   - Top Bar (Global Vitals)                        │
│   - Bottom Console (Command Deck & Minimap)        │
│   - Left Edge (Control Groups / Missions)          │
│   - Right Edge (Inspector / Target Details)       │
│   - Event Ticker (Chat / Kill Feed)                │
│   - Center Canvas (Glass / Transcluded Content)    │
├─────────────────────────────────────────────────────┤
│ [Z2] Modal Screens (Full-screen overlays)          │
│   - Agent Tuning / Debugging (Character Sheet)     │
│   - Artifact Lineage / Forensics (Map Screen)      │
│   - Settings / Theme Switcher                      │
└─────────────────────────────────────────────────────┘
```

- **Z0 (World):** The scenic, reactive, cinematic background. The primary interface. Not "wallpaper".
- **Z1 (HUD):** Diegetic/semi-transparent. Overlaid on the World. The software controls.
- **Z2 (Modal Screens):** Full-screen panels that block the World (Character Sheet, Skill Tree, Map, etc.).

---

## 2. The World (`WorldState`)

The **World** is the persistent, data-driven simulation. It is NOT a UI state. It exists whether you watch it or not.

### Entities

Entities are game objects, not UI widgets.

```rust
enum EntityType {
    Agent(AgentData),
    Swarm(SwarmData),
    Mission(MissionData),
    Artifact(ArtifactData),
    Event(EventData),
}

struct Entity {
    id: EntityId,
    etype: EntityType,
    // Spatial position in the world (for rendering, picking)
    pos: WorldPos,
    // Visual state (animation, sprite index)
    visual: EntityVisual,
}
```

- **Agent:** A single backend agent. State: idle, working, error, approval_needed.
- **Swarm:** A collection of agents. Visualized as a particle cloud/rather than 10,000 individuals.
- **Mission:** A high-level user goal (Research X, Build Y).
- **Artifact:** An output file/result.
- **Event:** A transient notification (Agent created, Workflow started).

### Visual Mapping

The World is the **primary visualization** of the system's state.

- **Agent counts → Particle density:** 100 active agents in "chunking" stage = 100 particles flowing in the "river" (Precursors) or streaming to the TET (TET Orchestrator).
- **Error rate → World atmosphere:** High error rate = stormy weather, rain on glass, dark clouds, wind. Low error rate = clear sky.
- **Storage saturation → World density:** High write load = blizzard of particles. Low load = sparse drift.
- **HITL approvals → Beacon:** An entity needing approval pulses or highlights in the World.

### Chunks & Spatial Index

For performance and picking, the World is divided into chunks or a spatial index.

- Only entities in the camera's view are rendered.
- Ray picking (click on entity) uses the spatial index for efficiency.

---

## 3. The Camera (`CameraSystem`)

The Camera defines **what part of the World we see**.

```rust
struct Camera {
    // Horizontal rotation (degrees)
    yaw_deg: f32,
    // Target yaw for smooth interpolation
    target_yaw: f32,
    // Vertical tilt (optional, start with locked or limited)
    pitch_deg: f32,
    // Zoom level (1.0 = default, >1.0 = zoomed in)
    zoom: f32,
    // Target zoom for smooth interpolation
    target_zoom: f32,
    // Shake amplitude for effects
    shake: f32,
}
```

### Interaction (1 Degree of Freedom to Start)

- **Mouse Drag:** Rotate camera yaw (pan left/right).
- **Scroll Wheel:** Zoom in/out.
- **Click on Entity:** Raycast → Select entity → Open Inspector.
- **Click on HUD:** Interact with HUD panels.
- **Focus Mode:** When a Control Group is selected in Left Panel, the camera smoothly pans to focus on that region of the world.

---

## 4. The Scenes (Cinematic Themes)

We will implement the `Scene` trait for different visual themes. Each Scene is a **different world** with its own atmosphere, agents, and scenery.

```rust
trait Scene {
    // Update the scene (weather, particles, effects)
    fn update(&mut self, world: &mut WorldState, dt: f32);
    // Render the scene to the egui Painter
    fn render(&self, painter: &egui::Painter, camera: &Camera, world: &WorldState);
}
```

### Theme 1: TET Orchestrator (Default)

**The Office:** A minimalist, translucent-walled bridge on a lonely space station. Outside the window is deep space. A single, massive structure (TET) floats in the distance.

- **Background:** Stars (parallax), TET structure (central object), Data trails (NATS traffic).
- **Agents:** Distant points of light (particles) or utility drones (sprites) moving with purpose.
- **Visual Mapping:**
  - Active agents = glowing moths around the TET.
  - Storage saturation = density of data trails.
  - System lag = TET glows duller.
- **Atmosphere:** Calm, spacious, contemplative.

### Theme 2: Precursors

**The Office:** A weathered stone watchtower from bronze-age at the edge of a cliff, overlooking a vast river valley.

- **Background:** Sky (gradient for time-of-day), Mountains, River, Path, Distant City.
- **Agents:**
  - Storage writes = River barges / boats flowing downstream.
  - Chunking/Indexing = Travellers on the path or caravans.
  - Idle agents = Campfires in the distance.
- **Visual Mapping:**
  - High write load = River current speeds up, boats become dense.
  - Errors = Whirlpools (stuck barges), stormy weather.
  - Approval needed = Beacon on a barge.
- **Atmosphere:** Natural, organic, serene.

### Theme 3: Descent

**The Office:** The interior of a drop-ship's observation deck, descending through the clouds of an alien world.

- **Background:** Cloud layers (rushing past), Revealed terrain (as mission progresses), Landing zone (Colony).
- **Agents:**
  - Processing agents = Drop-pods shooting down through clouds.
  - Completed artifacts = Structures/beacons on the ground.
- **Visual Mapping:**
  - Processing rate = Descent speed / cloud speed.
  - Artifact creation = New structures visible through clouds.
- **Atmosphere:** Dynamic, dramatic, arrival.

### Theme Engine

- A `ThemeManager` will hold the current `Scene` and apply theme-specific `egui::Style`, `Visuals`, and `AgentVisual` settings.
- **Configurable Agent Visuals:** Can switch between Particles (geometric shapes) and Sprites (iconic representations) on the fly.
- **Weather & Seasons:**
  - Weather state (Clear, Rainy, Stormy, Foggy) can be independent or mapped to external system metrics.
  - Time-of-day matches user's local time.
  - Seasons change color palette and weather frequency.

---

## 5. The HUD Overlay

The HUD is the **player's interface** within the World. It is overlaid on top of the Camera feed.

### Top Bar (Global Vitals)

Like the resource bar in an RTS.

- **Active Agents:** `🟢 142 Active | 🟡 4 Retrying | 🔴 1 Failed`
- **Token Burn:** Tokens/sec, $ cost/hr.
- **System Health:** NATS, State Server, S3 connectivity.
- **Global Alerts:** A concise ticker for critical system-wide events.

### Bottom Console (Command Deck & Minimap)

Like the bottom bar in an RTS.

- **Left - Event Ticker:** Scrolling, auto-fading feed of non-critical system events (analogous to multiplayer chat/kill feed). Overlaid on the world.
- **Center - Command Input:** Glowing text input. The **prompt bar**. Modes: *Command*, *Brainstorm*, *Query*, *Intervene*. Supports drag-and-drop of artifacts.
- **Right - Minimap / Radar:** A stylized placeholder. Could be a radar sweep, a rotating wireframe, or a mini top-down view of the world state.

### Left Edge (Control Groups / Missions)

Like the unit selection panel in an RTS or the party portraits in an RPG.

- **Missions / Swarms:** User's active missions are listed.
- **Status Indicators:** Each mission has a mini progress bar and status icon (🟢 progressing, 🟡 needs approval, 🔴 failed).
- **Focus:** Clicking a mission focuses the camera on that group in the world.

### Right Edge (Inspector / Target Details)

Like the character sheet in an RPG or the unit details in an RTS.

- **Contextual Details:** Shows data for the selected entity (Agent, Mission, Artifact).
- **Tabs:** Overview, Logs, Memory, Artifacts.
- **Actions:** Approve, Halt, Retry, Inspect Memory, View Lineage.
- **Forensics:** Click "View Logs" to open a full-screen modal (Z2) with virtualized logs.

---

## 6. The Center Canvas (Glass)

The central area of the screen is the **window into the World**. It is mostly transparent, letting the World show through. When the user opens a "Meeting" or a specific view, a **glass panel** slides in, transcluding content into the center.

- **Command Deck:** Transcludes the Command Input (same as bottom command bar but expanded).
- **Experts Meeting:** A glass panel with real-time chat and context chips (files, images, graphs).
- **Swarm Monitor:** A zoomable, grouped view of agents. Could overlay on the world or be a separate Z2 modal.
- **Artifact Graph:** A lineage visualization.
- **Forensics:** Virtualized logs with filters.

---

## 7. Ray Picking & Selection

- **Option A (Procedural 2D):** Each entity has a screen-space bounding box. Click → check bounding boxes.
- **Option B (True 3D):** Maintain a spatial acceleration structure (BVH / Grid). Click → unproject ray → intersect with geometry.

When an entity is picked, the **Right Inspector** updates to show its details.

---

## 8. Audio (`rodio`)

- **Ambient Loops:** Each Scene has an ambient audio track (TET hum, Precursors nature, Descent rumble).
- **Event Sounds:** One-shot sounds for UI interactions, approvals, errors.
- **Dynamic Audio:** Could tie audio to world state (rain sounds for rainy weather).

---

## 9. Data Flow: Bridge → World → Camera → HUD

1.  **Bridge Events:** The `BridgeClient` receives JSON events from the Python bridge.
2.  **World Update:** Events are translated into `Entity` spawns/updates/removals in the `WorldState`.
3.  **HUD Update:** Some events (e.g., approvals, global alerts) also update HUD panels.
4.  **Render Loop:** Every frame, the `WorldRenderer` renders the world based on the current `Camera`. The HUD panels are drawn on top using `egui`.

---

## 10. Technical Implementation: File Structure

```
apps/deer_gui/src/
├── main.rs          # Entry point, eframe setup
├── app.rs           # DeerGuiApp: owns World, Camera, Scene, HUD
├── bridge.rs        # BridgeClient: talks to Python process
├── models.rs        # View models for Bridge data (Thread, Message, etc.)
│
├── world/           # The World
│   ├── mod.rs
│   ├── state.rs     # WorldState, Entity, EntityType
│   ├── spatial.rs   # Spatial index for entities
│   └── ...
│
├── camera/          # The Camera
│   ├── mod.rs
│   ├── camera.rs    # Camera struct, interpolation
│   └── ...
│
├── scene/           # The Scenes
│   ├── mod.rs
│   ├── scene.rs     # Scene trait
│   ├── tet.rs       # TET Orchestrator Scene
│   ├── precursors.rs# Precursors Scene
│   ├── descent.rs   # Descent Scene
│   └── ...
│
├── hud/             # The HUD Overlay
│   ├── mod.rs
│   ├── top_bar.rs
│   ├── bottom_console.rs
│   ├── left_panel.rs
│   ├── right_inspector.rs
│   ├── event_ticker.rs
│   └── ...
│
├── theme/           # Theme Manager
│   ├── mod.rs
│   ├── theme.rs     # Theme definition, ThemeManager
│   └── ...
│
└── audio/           # Audio System
    ├── mod.rs
    ├── manager.rs   # rodio integration
    └── ...
```

---

## 11. Phased Implementation Plan

### Phase 1: World & Camera Foundation
1.  Create `world/state.rs`: Define `WorldState`, `Entity`.
2.  Create `camera/camera.rs`: Define `Camera` with yaw and smooth interpolation.
3.  Create `scene/scene.rs`: Define `Scene` trait.
4.  Create `scene/tet.rs`: Basic implementation of TET Orchestrator.
5.  Modify `app.rs`: Replace central panel with full-screen canvas. Use `Scene` to render. Add mouse-drag camera rotation.

### Phase 2: HUD Prototype over World
1.  Convert Top Bar, Left Panel, Right Panel to use semi-transparent frames.
2.  Implement Event Ticker as a scrolling overlay on the bottom-left.
3.  Implement Command Console as an overlay at the bottom center.

### Phase 3: Entity System
1.  Bridge Events → Entity Updates in `WorldState`.
2.  Render Entities in Scene (particles/sprites).
3.  Implement Ray Picking.
4.  Selection → Update Right Inspector.

### Phase 4: Additional Scenes & Theme Engine
1.  Implement `PrecursorsScene`.
2.  Implement `DescentScene`.
3.  Implement `ThemeManager` for runtime theme switching.

### Phase 5: Polish & Audio
1.  Camera Focus (smooth pan to selected group).
2.  Weather & Time-of-day effects.
3.  Audio (ambient loops, event sounds).
4.  Optimizations (spatial index, culling, virtualization).

---

## 12. Visual Direction: Cinematic, Not Neon

- **Avoid cheap neon.** Go for muted, grounded, realistic sci-fi.
- **Transparent, glass HUD.** Let the World breathe. Panels have subtle borders, not heavy chrome.
- **Data-driven beauty.** The busier the system, the more "alive" the world looks. Calm system = calm world.
- **Focus on atmosphere.** The world should make the user feel like a Director in a command center, not a debugger in an IDE.