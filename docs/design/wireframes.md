# Control Center Wireframes (Textual)

## Architectural Paradigm
The `apps/deer_gui` desktop app is evolving from a traditional "chat app" into a **Tactical Command Center**. The UI must reflect the architecture defined in `state-server.md` (HOTL/HITL, massive agent swarms, immutable artifact lineage). It is a cinematic, spatial operations dashboard, not a dense document editor.

### Core UX Model
- **Mission:** The top-level unit the user cares about (replaces "Thread").
- **Workflow:** A durable run or branch under a mission.
- **Agent Swarm:** Many agents contributing to a workflow (spatial/aggregated visualization).
- **Artifact Graph:** Outputs, references, datasets, logs, and embeddings lineage.
- **Attention Queue:** Approvals, failures, retries, ambiguity, ready artifacts (replaces "Task Board").
- **Focus Lens:** The current zoom level, from executive summary down to raw logs.

### Interaction Paradigm (HITL vs HOTL)
- **HITL (Human-In-The-Loop):** Interruptive but bounded cards with clear action, deadline, and consequence; approval should be 1-click with a drill-in option.
- **HOTL (Human-On-The-Loop):** Passive monitoring cards with confidence/progress/state. User watches without being dragged into noise.
- **Aggregation:** 1000 agents finishing a task is one progress bar, not 1000 rows.
- **Actionable Alerts:** Promote only actionable items; auto-silence repetitive success noise and dedup repeated failures.

### Visual Aesthetic
- **"The Director's Window":** A corner-office view overlooking a bustling, majestic automated shipyard/city. Cinematic depth with the feel of a lot of interesting activity in the distance.
- **Themes:** Supports hot-swappable themes.
    - Default: Cinematic Sci-Fi (deep blues/blacks/cyans with glowing accents).
    - Secondary: Daylight Lab (light mode, clean, medical).
- **HUD Layers:**
    - Deep Background: Dynamic, living visualization of `State Server` traffic (parallax, geometric constellations, particles).
    - Midground (Canvas): The active work area (Experts Meeting, Artifact Graph, Swarm Topology).
    - Foreground HUD: Anchored, high-contrast framing panels (Cockpit/RTS style).

---

## 1. Top Bar (Global Resource Tracker)
*RTS Resource Bar style.*

```text
┌───────────────────────────────────────────────────────────────────────────────┐
│ DEER FLOW OPS  |  🟢 12,400 Active Agents  🟡 42 Retrying  🔴 3 Blocked     │
│ 14:05:22 UTC   |  Compute Burn: 42k t/s ($1.20/hr) | State Server: ONLINE     │
└───────────────────────────────────────────────────────────────────────────────┘
```

## 2. Bottom Console (Command Deck & Minimap)
*RTS/FPS Command HUD style.*

```text
┌───────────────────────┬───────────────────────────────────┬───────────────────┐
│ ATTENTION QUEUE       │ COMMAND CONSOLE                   │ MINIMAP / RADAR   │
│ ───────────────────── │ ───────────────────────────────── │ ───────────────── │
│ [!] Agent 42 Approval │  [Direct] [Brainstorm] [Halt]     │    / \   / \      │
│ [i] L3 Chunks Ready   │ ┌───────────────────────────────┐ │   | 🟢 |-| 🟡 |   │
│ [x] S3 Timeout Retry  │ │ Type prompt, drop artifacts,  │ │    \ /   \ /      │
│ [i] Workflow Started  │ │ or issue swarm directives...  │ │     |     |       │
│                       │ └───────────────────────────────┘ │    / \   / \      │
│ 14 more hidden...     │       [ EXECUTE COMMAND ]         │   | 🟢 |-| 🔴 |   │
└───────────────────────┴───────────────────────────────────┴───────────────────┘
```

## 3. Left Edge (Control Groups / Missions)
*RPG Party/Squad UI style.*

```text
┌───────────────────────┐
│ MISSIONS & VIEWS      │
│ ───────────────────── │
│ ◉ Deep Research Alpha │
│   [▓▓▓▓▓▓▓░░░] 70%    │
│   Status: Active      │
│ ───────────────────── │
│ ◉ UI Translation      │
│   [▓▓▓░░░░░░░] 30%    │
│   Status: BLOCKED [!] │
│ ───────────────────── │
│ ◉ Daily Ingest        │
│   [▓▓▓▓▓▓▓▓▓▓] 100%   │
│   Status: Idle        │
└───────────────────────┘
```

## 4. Right Edge (Inspector / Target Details)
*RTS Unit Inspector / Build Menu style.*

```text
┌───────────────────────┐
│ INSPECTOR             │
│ ───────────────────── │
│ TARGET: Agent 42      │
│ ROLE: Synthesizer     │
│ STATE: Awaiting User  │
│ ───────────────────── │
│ TOKENS: 4,021         │
│ CONTEXT: 12 Files     │
│ ───────────────────── │
│ PENDING ACTION:       │
│ "Should I overwrite   │
│  the existing index?" │
│                       │
│  [ APPROVE ] [ REJECT]│
│ ───────────────────── │
│ [ View Raw Logs ]     │
│ [ Inspect Memory ]    │
└───────────────────────┘
```

## 5. Center Canvas Modes (The "Battlefield")
*The central view changes based on the required focus.*

### Mode A: Experts Meeting (The Comms Log)
*RPG Dialogue / Tactical Board style.*

```text
┌───────────────────────────────────────────────────────────────────────────────┐
│ TACTICAL BOARD (Pinned Context)                                               │
│ [Image: UI3.webp]  [File: state-server.md]  [Artifact: Index_v2.bin]          │
├───────────────────────────────────────────────────────────────────────────────┤
│ COMMS LOG                                                                     │
│ ─────────────────────                                                         │
│ DIRECTOR (You): "Analyze UI3.webp and correlate with state-server.md"         │
│                                                                               │
│ SYNTHESIZER (Agent): "Correlating now. I notice the HUD maps to our HOTL..."  │
│                                                                               │
│ DATA SCOUT (Agent): "I've pulled 3 similar HUD examples from the Data Lake."  │
│ [View Attached Examples]                                                      │
└───────────────────────────────────────────────────────────────────────────────┘
```

### Mode B: Swarm Monitor (Aggregated Visualization)
*Cinematic, grouped particle/hex view of background operations.*

```text
┌───────────────────────────────────────────────────────────────────────────────┐
│ SWARM TOPOLOGY                                                                │
│                                                                               │
│       [ INGESTION CLUSTER ]                  [ PROCESSING CLUSTER ]           │
│       1,000 Agents Working                   4,500 Agents Working             │
│       (Pulsing Blue Hexes) ───────────────►  (Fast Moving Cyan Particles)     │
│                                                       │                       │
│                                                       ▼                       │
│                                              [ VECTOR SYNTHESIS ]             │
│                                              42 Agents Retrying               │
│                                              (Flashing Amber Warning)         │
└───────────────────────────────────────────────────────────────────────────────┘
```

### Mode C: Artifact Graph (Lineage Browser)
*Node-based graph tracing prompt -> agent -> artifact.*

```text
┌───────────────────────────────────────────────────────────────────────────────┐
│ ARTIFACT LINEAGE: Prompt_ID_882                                               │
│                                                                               │
│  [ Prompt ] ──► [ Agent: Researcher ] ──► [ L0: Raw Doc ]                     │
│                                                │                              │
│                                                ▼                              │
│                                           [ Agent: Chunker ]                  │
│                                                │                              │
│                                                ▼                              │
│                                           [ L1: Markdown Chunks ]             │
└───────────────────────────────────────────────────────────────────────────────┘
```

---

## 6. Cinematic Background System (Z0)
*The reactive, living world behind the HUD.*

### Architectural Layers

```
┌─────────────────────────────────────────────────────┐
│ [Z4] Top Bar (Global Resources, Health, Alerts)    │
├─────────────────────────────────────────────────────┤
│ [Z3] Right/Left Framing (Control Groups, Inspector)│
├──────────────┬─────────────────────────────────────┤
│              │ [Z2] Center Canvas (The "Glass":     │
│ [Z0] Deep    │   - Experts Meeting (Chat)           │
│   Background │   - Swarm Monitor                   │
│   (Cinematic │   - Artifact Graph                  │
│   World)     │   - Forensics Logs                  │
│              │   )                                 │
│              │                                     │
├──────────────┴─────────────────────────────────────┤
│ [Z1] Bottom Console (Command Deck & Minimap)      │
└─────────────────────────────────────────────────────┘
```

### Camera System
- **Degrees of Freedom:** Start with 1-DOF (yaw/horizontal rotation only).
- **User can drag left/right** to "look around" the window frame.
- **Focus Mode:** When user selects a swarm/mission, camera smoothly interpolates to center that activity in the background.
- **Zoom:** Scroll wheel adjusts distance (future).

### Agent Representation (Configurable)
- **Particle Mode (Default):** Agents are glowing points/shapes. Fast, pure data aesthetic.
- **Sprite Mode:** Agents are iconic representations (boats, walkers, pods). Can be toggled in settings.

### Ray Picking
- **2D Phase (Option A):** Point-in-rect test on agent screen positions. Click to select, opens Inspector.
- **3D Phase (Option B):** True raycast from camera through world space intersects agent meshes.

---

## 7. Themes (Cinematic Atmospheres)

### Theme Structure
```rust
struct Theme {
    name: String,
    background_renderer: Box<dyn BackgroundScene>,
    panel_decorator: Box<dyn PanelDecorator>,
    agent_visual_style: AgentVisual,
    ambient_sounds: Vec<AmbientSound>,
    weather: Option<WeatherState>,
}
```

### Theme 1: "TET Orchestrator" (Default)
*The Office: Minimalist, translucent-walled bridge on a lonely space station.*

- **Background:**
    - Stars Layer (far, slow parallax).
    - TET Structure: large geometric monolith in distance, brightness = swarm activity.
    - Data Trails: thin cyan lines streaming past window = NATS messages.
    - Hover-barges: utility drones carrying glowing containers = Artifacts.
- **Agents:** Distant points of light moving with purpose. Swarms cluster around TET.
- **Visual Style:** Deep blues/blacks/cyans, high-contrast HUD frames, clean geometric.
- **Audio:** Low synthesized hum, wind.

### Theme 2: "Precursors"
*The Office: Weathered stone watchtower at edge of cliff, overlooking vast river valley.*

- **Background:**
    - Sky Layer: Dynamic gradient (dawn/dusk/night based on real-world time).
    - River Layer: Wide, winding river. Data flows = boats/barges (downstream = writes, upstream = reads).
    - The Path: Winding mountain path. Agents = traveller dots moving in groups.
    - The City (State Server): Distant, fortified city on horizon. At night, glows warmly = healthy state.
    - Clouds/Weather: Soft rolling clouds/mist for depth.
- **Agents:** Travellers and river barges. Stuck boat = retrying/failing agent.
- **Visual Style:** Earth tones, stone/wood HUD frames, warm amber glows.
- **Audio:** Birds, water rushing, distant cart wheels.

### Theme 3: "Descent"
*The Office: Drop-ship observation deck, descending through clouds of alien world.*

- **Background:**
    - Cloud Layer: Translucent wisps rushing past. Speed = rate of new write events.
    - Terrain Layer: As mission progresses, clouds part to reveal procedural terrain (mountains, rifts, forests).
    - The Colony (Artifacts): On ground, beacon fires/structures = new L0/L1/L2 artifacts.
- **Agents:** Glowing drop-pods (new agent spawn). Distant swarms on ground.
- **Visual Style:** Oranges/teals, dynamic motion, cockpit HUD overlays.
- **Audio:** Engine rumble, wind.

---

## 8. System State → Visual Mapping

### Mapping Backend Metrics to Cinematic World

| Metric / Event | TET Theme | Precursors Theme | Descent Theme |
|----------------|-----------|------------------|---------------|
| NATS Saturation | Dense cyan trails, fast movement | River current speeds up, more boats | Wind speed increases, ship vibrates |
| State Server Lag | TET glows dull, barges slow | Distant bridge becomes misty/fades | Cockpit HUD shows "SYSTEM LAG" warning |
| Agent Swarm Active | TET surrounded by moths/points | Caravans on path, boats on river | Glow pods descending, ground activity |
| Agents Retrying/Failing | Spinning/flickering lights | Boat stuck in whirlpool/eddy | Flickering drop-pod |
| HITL Approval Needed | Highlighted barge + HUD prompt | Glowing beacon on boat + chime | Highlighted pod + HUD alert |
| Artifact Created | Hover-barge carries crate | Hunter-gather returns to city | Beacon fire lit on ground |

### Aggregation Strategy
- **Never draw 10,000 widgets.**
- Map agent counts to visual density:
    - 100 batch-chunking agents → 5 visible boats in river.
    - User understands scale through representation, not exact count.

---

## 9. Weather & Seasons

### Weather States
- `Clear`: Normal operation. High visibility.
- `Foggy`: High external API latency. Decreased agent visibility.
- `Rainy`: Increased system load. Subtle particle effect on "window glass".
- `Stormy`: High error rate. Lightning flashes, camera shake.

### Seasons
- **Spring:** Brighter/warmer palette (Precursors).
- **Winter:** Cooler/blue palette, possible snow (Precursors).
- Can map season to real-world calendar or project lifecycle phase.

### Dynamic Mapping
- Weather can be manually toggled or auto-mapped to infrastructure metrics:
    - `Rainy` = External API latency is high.
    - `Stormy` = Error rate exceeds threshold.

---

## 10. Audio Integration

### Ambient Loops (per theme)
- TET: `spaceship_hum.mp3`, low ambient wind.
- Precursors: `nature_ambience.mp3`, water, distant birds.
- Descent: `engine_rumble.mp3`, rush of air.

### One-Shot Sounds
- UI clicks, approval chimes.
- Ambient alert tones for HITL requests.

### Implementation
- Use `rodio` crate for audio playback.
- Audio is optional and configurable via settings.

---

## 11. Technical Implementation Approach

### Phase 1: Foundational Architecture
1. Refactor `DeerGuiApp` into layout modules (`layer_background.rs`, `layer_hud.rs`, `panels/`).
2. Implement `Theme` trait and `ThemeManager` with basic `DarkSciFi` theme.
3. Create simple 2D procedural background (stars & geometric shapes).
4. Implement 1-DOF camera (yaw) with mouse drag.
5. Connect Top Bar and Background to real `SystemState` from bridge events.

### Phase 2: Themes & Sprites
1. Implement `PrecursorsScene` with panoramic 2D layering.
2. Map agent states to moving dots (boats, walkers).
3. Load and render 2D sprite sheets for agents.
4. Implement 2D picking (point-in-rect).
5. Add ambient audio.

### Phase 3: Advanced Visualization
1. Implement `DescentScene` with procedural clouds/terrain.
2. Dynamic zoom tied to write throughput.
3. Weather/season variations.

### Phase 4: Center Canvas Modes
1. Implement Swarm Monitor using `egui::Painter`.
2. Implement Artifact Graph (basic node-link).
3. Integration: clicking canvas entities opens Inspector.

### Phase 5: Polish
1. Camera focus transitions.
2. Smooth animations.
3. Hover effects, transitions.

---

## 12. File Structure (Planned)

```
apps/deer_gui/src/
├── app.rs              # Main app, orchestrates layers
├── scene/              # Background scene implementations
│   ├── mod.rs
│   ├── tet_scene.rs
│   ├── precursors_scene.rs
│   └── descent_scene.rs
├── theme/              # Theme engine
│   ├── mod.rs
│   ├── theme.rs
│   └── manager.rs
├── hud/                # HUD panels
│   ├── mod.rs
│   ├── top_bar.rs
│   ├── bottom_panel.rs
│   ├── left_panel.rs
│   └── right_panel.rs
├── canvas/             # Center canvas modes
│   ├── mod.rs
│   ├── experts_meeting.rs
│   ├── swarm_monitor.rs
│   ├── artifact_graph.rs
│   └── forensics.rs
├── view_models.rs      # System state, visual state
├── audio.rs            # Audio manager (rodio)
├── models.rs           # (existing)
└── bridge.rs           # (existing)
```

---

## 13. Open Design Questions

1. **Inspector Depth:** For deep logs/forensics, should the inspector expand in-place (virtualized scroll), or switch center canvas to a full "Forensics Mode"?**A:** Hybrid (Option C) recommended.

2. **Swarm Default Grouping:** By workflow stage, by mission ID, or by agent type?  
**A:** Workflow Stage (Option A) recommended.

3. **Background Scope:** Show all agents in system, only selected mission, or hybrid (all visible but selected ones highlighted)?  
**A:** Hybrid - Show all agents, visually highlight selected mission's agents (Option C recommended).

4. **Minimap Representation:** Live DAG of workflow stages, or abstract heat-map of activity?  
**A:** Live DAG (Option A) recommended.
