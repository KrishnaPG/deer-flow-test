# Sci-Fi RTS HUD Design Spec

**Project**: `apps/deer_gui`
**Stack**: Bevy 0.18 + bevy_egui 0.39
**Date**: 2026-03-30
**Status**: Ready for writing-plans

---

## 1. Aesthetic Direction

**Style**: Holographic / Homeworld 2
**Character**: Deep space. Calm, intelligent, military precision. The interface of a species that has mastered space travel.

Rules:
- Dissolving-edge transparent panels — no hard border strokes on any primary HUD element
- Cyan chrome on deep space black
- All panels use glassmorphism (see Section 4)
- Solid opaque panels only for Tier 3 modal overlays
- Shape language: chamfered corners (45° cuts), beveled rectangles — not rounded corners
- Subtle hexagonal grid background texture at 5% opacity; scan-line panel texture at 3% opacity

---

## 2. Color Tokens

```rust
// src/ui/theme.rs
use bevy::color::Color;

pub const BACKGROUND:    Color = Color::srgba(0.020, 0.031, 0.063, 1.000); // #050810
pub const SURFACE:       Color = Color::srgba(0.039, 0.055, 0.094, 0.900); // #0A0E18 ~90%
pub const ACCENT_CYAN:   Color = Color::srgba(0.000, 0.800, 0.867, 1.000); // #00CCDD
pub const ACCENT_AMBER:  Color = Color::srgba(0.941, 0.753, 0.251, 1.000); // #F0C040
pub const WARNING_ORANGE:Color = Color::srgba(1.000, 0.533, 0.000, 1.000); // #FF8800
pub const CRITICAL_RED:  Color = Color::srgba(1.000, 0.133, 0.000, 1.000); // #FF2200
pub const SUCCESS_GREEN: Color = Color::srgba(0.000, 1.000, 0.533, 1.000); // #00FF88
pub const SPECIAL_PURPLE:Color = Color::srgba(0.533, 0.267, 1.000, 1.000); // #8844FF
pub const TEXT_PRIMARY:  Color = Color::srgba(0.910, 0.957, 0.973, 1.000); // #E8F4F8
pub const TEXT_DIM:      Color = Color::srgba(0.910, 0.957, 0.973, 0.500); // #E8F4F8 50%
pub const PANEL_FILL:    Color = Color::srgba(0.020, 0.031, 0.063, 0.750); // #050810 75%
pub const PANEL_BORDER:  Color = Color::srgba(0.000, 0.800, 0.867, 0.200); // #00CCDD 20%

// egui equivalents (use in egui Style overrides)
// PANEL_FILL  -> egui::Color32::from_rgba_unmultiplied(5, 8, 16, 191)
// PANEL_BORDER-> egui::Color32::from_rgba_unmultiplied(0, 204, 221, 51)
```

---

## 3. Typography

| Use Case | Font | Size (px) | Weight | Transform |
|---|---|---|---|---|
| Resource counts | JetBrains Mono | 28 | Regular | Numeric |
| Alert text | JetBrains Mono | 20 | Regular | ALL-CAPS |
| Zone headers / status labels | IBM Plex Sans | 16 | Medium | ALL-CAPS |
| Command button labels | IBM Plex Sans | 14 | Regular | ALL-CAPS |
| Unit names / event descriptions | IBM Plex Sans | 14 | Regular | Mixed Case |
| Detail / secondary stats | IBM Plex Sans | 12 | Regular | Mixed Case |
| Timestamps / ticker | JetBrains Mono | 10 | Regular | Mixed Case |

**Rules**:
- Monospace for ALL numeric values without exception — prevents layout shift on value change
- ALL-CAPS for zone headers and status labels
- Mixed case for unit names and narrative event descriptions
- Letter-spacing: 0.05em on ALL-CAPS labels, 0 elsewhere

---

## 4. Glassmorphism Panel Rules

```
Primary panel fill:   rgba(5,   8,  16, 0.75)   alpha must be 0.65–0.85
Primary panel border: rgba(0, 204, 221, 0.20)   1px, cyan ghost
Backdrop blur:        12px                       (Bevy: render-to-texture + separable blur pass)
Corner style:         chamfered 45°              (painter.add(PathShape) — not rounding)
Modal fill:           rgba(5,   8,  16, 0.95)   solid for Tier 3 only
```

---

## 5. Zone Architecture

| # | Zone | egui Panel Type | Width | Height | Visibility Tier | Content |
|---|---|---|---|---|---|---|
| 1 | Resource Bar | `TopBottomPanel::top("resource_bar")` | 100% | 4% | Always | Resources, alerts, fleet summary |
| 2 | Left Event Feed | `SidePanel::left("event_feed")` | 16% | 74% (between bars) | Always (collapsible) | Scrolling combat/build/event log |
| 3 | World Viewport | `CentralPanel` (pass-through) | rem | rem | Always | 3D scene — no permanent overlap |
| 4 | Minimap | `Area` anchored top-right inside right panel | 15% | ~30% of right | Always | Terrain, units, fog, frustum |
| 5 | Fleet Tree | `SidePanel::right("fleet_tree")` upper portion | 15% | ~44% of right | Always (collapsible) | Hierarchical fleet/wing/ship list |
| 6 | Selection Panel | `TopBottomPanel::bottom("selection_panel")` left section | 20% | 18–22% | Tier 2: on select | Portrait, stats, ability icons |
| 7 | Command Console | `TopBottomPanel::bottom("command_console")` center | 45% | 18–22% | Tier 2: on select | Tabbed: Orders / Build / Intel |
| 8 | Build Queue | `TopBottomPanel::bottom("build_queue")` right section | 25% | 18–22% | Tier 2: on select | Queue timers, research progress |

*The bottom bar is a single `TopBottomPanel::bottom` containing three horizontal sections drawn via `ui.columns(3, ...)` or manual layout with `ui.allocate_ui_at_rect`.*

---

## 6. Component Inventory

| Component | Module | Description |
|---|---|---|
| `ResourceBar` | `ui::resource_bar` | Top strip. Renders 4 resource slots (icon + monospace count), center alert banner, right fleet summary from a read-only `ResourceBarProjection`. Data-pulse animation keys off newly applied live projection sequences rather than local mutation events. |
| `EventFeed` | `ui::event_feed` | Left panel. `VecDeque<LogEntryProjection>` max 200. `ScrollArea::vertical` with `stick_to_bottom`. Entries are sequence-keyed, deduped by authoritative `sequence_id`, colored by `LogSeverity`, and collapse to a 32px icon rail with unread badge. |
| `Minimap` | `ui::minimap` | Fixed `egui::Frame` 240×240 in right panel. Painter-driven from `MinimapProjection`: terrain rects, unit dots, fog overlay, white frustum rect. Octagonal clip mask. Click updates local camera framing only; gameplay-affecting orders use explicit intents. |
| `FleetTree` | `ui::fleet_tree` | Right panel below minimap. Collapsible tree: Fleet → Wing → Ship from authoritative fleet projections. Each row: icon + name + thin health bar + status dot. Click updates local focus chrome; double-click focuses the local camera. |
| `SelectionPanel` | `ui::selection_panel` | Bottom-left. Shows when the acknowledged selection/command-focus projection is non-empty. Large portrait `TextureHandle`, unit name/class, health ring (circular painter arc), 2×2 stat grid, 4–6 ability icons with cooldown overlays. Pending targeting/selection visuals render as overlays until confirmed or rejected. |
| `CommandConsole` | `ui::command_console` | Bottom-center. Tab bar (`[O]rders` / `[B]uild` / `[I]ntel`) drawn with painter. Orders tab: 5×3 button grid with hotkey labels. Build tab: queue + progress. Intel tab: full unit data. Button presses emit `PlayerIntent` envelopes only; they never mutate local canonical mission state. |
| `BuildQueue` | `ui::build_queue` | Bottom-right. Two stacked sections: (1) authoritative active build items with countdown timers + cancel intents, (2) active research with progress bar + queued research icon strip. Pending entries may appear as dashed/ghost overlays keyed by `intent_id` until the State Server confirms or rejects them. |

---

## 7. Animation Spec

| Animation | Trigger | Duration | Easing | Replay-safe rule / egui implementation |
|---|---|---|---|---|
| Scan-line sweep | Panel becomes visible from an acknowledged projection change or local chrome toggle | 120ms | Linear | Fire once per visibility epoch. Suppress while `ConnectionState::Rehydrating`. `painter.hline` at `y = panel_top + panel_height * progress`; `ScanLineAnim { panel_key, started_at_sequence, progress }`. |
| Data pulse | Numeric field changes between live projection sequences | 80ms | Ease-out | Cache `(field_key, sequence_id)` and animate only when `sequence_id > last_visual_sequence`. `DataPulse { field_key, intensity }`; `color = lerp(WHITE, normal_color, 1.0 - intensity)`. |
| Threat ping | New authoritative threat marker reaches the live tail | 600ms | Ease-out | Key by `(threat_id, sequence_id)` or `(unit_id, sequence_id)`. Duplicate or replayed sequences do nothing. `ThreatPing { radius, alpha }`; `painter.circle_stroke` expanding + fading on minimap. |
| Build complete | Authoritative build item transitions into `Complete` on a new live sequence | 400ms | Ease-out | Burst only on first live observation of that terminal transition. Catch-up/replay updates the queue silently. `BuildCompleteAnim { build_id, sequence_id, t }`. |
| Panel slide | Authoritative selection/build-presence toggles, or local-only chrome expands/collapses | 150ms | Ease-out-cubic | Gameplay-driven slides wait for confirmed projection change; local chrome may animate immediately. Animate panel `min_size` or `egui::Area` `Pos2` position. |
| Idle flicker | Always | 2s period | Sine | Purely decorative and client-local. Safe during replay/rehydration because it is not keyed to external events. `border_alpha = 0.20 + 0.003 * sin(time * TAU / 2.0)`. |

Sequence and replay rules:
- Every snapshot and delta consumed by the HUD carries `mission_id`, `sequence_id`, and a stable domain key (`build_id`, `log_id`, `unit_id`, etc.).
- The client tracks `ProjectionCursor { last_applied_sequence, last_visual_sequence, connection_state }`, where `connection_state` is `Connected`, `Disconnected`, or `Rehydrating`.
- Messages at or below `last_applied_sequence` are duplicates and are discarded. Sequence gaps move the client into `Rehydrating` until a snapshot/delta stream closes the gap.
- While `Rehydrating`, authoritative data still updates projections, but transient one-shot visuals (`ThreatPing`, `BuildCompleteAnim`, `DataPulse`, event-feed unread bursts) are suppressed.
- Animation state lives as Bevy-side `Component`s/resources keyed by projection sequence or `intent_id`. egui remains a stateless renderer.

---

## 8. Projection-Driven State Server Model

The HUD is storage-native: object storage remains the only canonical truth, the State Server exposes an ABAC-filtered hot cache, and the Bevy app holds read-only projections for rendering. The client never treats local ECS resources as authoritative gameplay state.

Core rules:
- Startup and reconnect path: request a mission-scoped snapshot from the State Server, record its `sequence_id`, then subscribe to deltas strictly after that cursor.
- The transport layer may be State Server WebSocket/SSE backed by NATS JetStream, but the UI contract is the same: ordered snapshot + delta projection messages carrying stable IDs and sequence numbers.
- ECS resources in the HUD are projections only. They may be reduced, indexed, and cached locally for rendering, but they are never the source of truth and must be replaceable by replay.
- Local-only UI state is allowed in separate chrome resources (`UiChromeState`, hover state, active tab, local camera framing, animation caches). Those resources must never masquerade as canonical mission state.

| Zone | State Server projection feed | Local Bevy projection |
|---|---|---|
| Resource Bar — resources / alerts / fleet summary | `resource_bar.snapshot` + `resource_bar.delta` | `Res<ResourceBarProjection>` with `resources`, `active_alert`, `fleet_summary`, `last_sequence_id` |
| Event Feed | `event_feed.snapshot` + `event_feed.delta` | `Res<EventFeedProjection>` storing sequence-keyed `VecDeque<LogEntryProjection>` and latest unread cursor |
| Minimap — terrain / units / fog | `minimap.snapshot` + `minimap.delta` | `Res<MinimapProjection>` with preprojected terrain, unit dots, fog polygons, and stable `unit_id` keys |
| Minimap — camera frustum overlay | authoritative world/camera markers from `minimap.delta`, plus local camera framing | `Res<MinimapProjection>` + `Res<UiChromeState>` for local viewport-only overlays |
| Fleet Tree | `fleet_tree.snapshot` + `fleet_tree.delta` | `Res<FleetTreeProjection>` grouped Fleet → Wing → Ship by authoritative IDs |
| Selection Panel | `selection.snapshot` + `selection.delta` | `Res<SelectionProjection>` for acknowledged command focus, plus `Res<PendingIntentLedger>` for temporary overlays |
| Command Console | `command_console.snapshot` + `command_console.delta` | `Res<CommandConsoleProjection>` defining available tabs, button states, and hotkeys |
| Build Queue | `build_queue.snapshot` + `build_queue.delta` | `Res<BuildQueueProjection>` for authoritative queue/research state, with pending rows sourced separately from `PendingIntentLedger` |

Shared client resources:

```rust
pub struct ProjectionCursor {
    pub mission_id: MissionId,
    pub last_snapshot_sequence: u64,
    pub last_applied_sequence: u64,
    pub last_visual_sequence: u64,
    pub connection_state: ConnectionState,
}

pub enum ConnectionState {
    Connected,
    Disconnected,
    Rehydrating,
}

pub struct PendingIntentLedger {
    pub entries: HashMap<IntentId, PendingIntentVisual>,
}

pub struct UiChromeState {
    pub collapsed_panels: UiCollapsedPanels,
    pub active_command_tab: CommandTab,
    pub hovered_row: Option<RowId>,
    pub local_camera_rect: Option<Rect>,
}
```

Reducer rule: all projection reducers are idempotent and sequence-aware. Applying the same message twice must produce the same local state. Any domain list shown in the HUD (`EventFeed`, `BuildQueue`, `FleetTree`) must use stable IDs plus `sequence_id` so duplicate replay traffic cannot create duplicate rows.

---

## 9. Intent Boundary and Command Lifecycle

All gameplay-affecting writes from the HUD cross a strict intent boundary. The UI may express intent immediately, but only the State Server may validate, sequence, and reflect the authoritative result back into projections.

```rust
pub struct PlayerIntent {
    pub intent_id: IntentId,
    pub mission_id: MissionId,
    pub operator_id: OperatorId,
    pub kind: IntentKind,
    pub payload: IntentPayload,
    pub expected_base_sequence: u64,
    pub client_sent_at_ms: u64,
}
```

| Lifecycle | HUD treatment | Exit condition |
|---|---|---|
| Pending | Render a dashed/ghost affordance keyed by `intent_id` (ghost waypoint, dashed build slot, subtle spinner, disabled repeat click). Do not mutate authoritative projections locally. | `CommandConfirmed`, a matching authoritative projection delta, or `CommandRejected` |
| Confirmed | Remove pending overlay and let the authoritative projection fully own the display. If the authoritative result differs from the optimistic hint, snap/animate toward the server result. | Matching `intent_id` or stable domain key appears in the projection stream |
| Rejected | Remove the pending overlay, restore the prior projection view, and emit an amber/red alert row describing the rejection reason or superseding conflict. | `CommandRejected { intent_id, reason }` |

Conflict rule: if two operators submit conflicting intents, the HUD follows the deterministic sequence emitted by the State Server. Any locally pending affordance that loses the race rolls back cleanly and surfaces the rejection as UI feedback rather than trying to resolve the conflict on the client.

---

## 10. egui Panel Approach

```
TopBottomPanel::top("resource_bar")
    └── Resource slots + Alert banner + Fleet summary

SidePanel::left("event_feed")          [resizable: false, width: 16%]
    └── ScrollArea::vertical() with VecDeque<LogEntry>

CentralPanel ("world_viewport")
    └── Pass-through — only world-space overlays (health bars, selection rings)
        rendered as egui Areas with world→screen projected positions

SidePanel::right("right_panel")        [resizable: false, width: 15%]
    ├── Fixed Area: Minimap (top ~30% of panel height)
    └── ScrollArea: Fleet Tree (remaining height)

TopBottomPanel::bottom("bottom_bar")   [height: 20% of window]
    └── ui.columns([0.20, 0.45, 0.25], |cols| {
            cols[0] → SelectionPanel
            cols[1] → CommandConsole (tab bar + content)
            cols[2] → BuildQueue
        })
```

Egui panel order (determines overlap/clip): top → left → right → bottom → central.
Right panel must be added before bottom panel so right panel extends full height.
All panes render from projection resources plus `UiChromeState`; no pane mutates authoritative mission state directly.

---

## 11. Open Questions — All Resolved

| Question | Resolution |
|---|---|
| Minimap shape: rect or octagon? | Octagonal clip mask over rectangular data. Mask applied as painter `PathShape` clip. |
| Tooltip cascade depth? | Three layers: immediate name (hover) → full stats (600ms) → comparison (hold Alt). |
| Rarity system? | 5 tiers: Common grey / Uncommon green / Rare blue / Epic purple / Legendary amber. Applied to ship class labels and tech tree nodes. |
| Command grid size? | 5×3 (15 slots), consistent with SC2 muscle memory. Bottom-right slot reserved for Cancel at all times. |
| Collapsible panels: default state? | Event feed: open. Fleet tree: open. Both collapse to 32px icon rail with unread badge. Preference persisted in local `Res<UiPreferences>` / `UiChromeState`, not in authoritative mission state. |
| Blur implementation in Bevy? | Render HUD panels to a separate `RenderTarget`; apply separable Gaussian blur (σ=4px, 12px visual) as a post-process pass before compositing. Initial version: skip blur, use lower alpha (0.65) as fallback. |
| Font loading? | Load JetBrains Mono + IBM Plex Sans via `egui::FontDefinitions` in startup system. Embed as `include_bytes!` assets. |
| Health ring implementation? | `ui.painter().arc` does not exist in egui — use `painter.add(PathShape::closed_line(...))` with manual arc point generation. Helper fn: `fn arc_points(center, radius, start_angle, end_angle, n: usize) -> Vec<Pos2>`. |
| World-space health bars? | Bevy system projects `GlobalTransform` positions through `Camera::world_to_viewport`; results stored in `Res<WorldSpaceOverlays>`; egui `Area`s positioned each frame at those screen coords. |
| Event log max entries? | 200 entries (`VecDeque` cap). Drain from front when full. Full log exportable to file via `[Export Log]` button in event feed header. |
