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
