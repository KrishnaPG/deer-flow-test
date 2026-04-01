## World-Primary Shell Support

### `battle_command`

Purpose:

- world-primary HOTL/HITL shell for tactical supervision, rapid command issuing,
  spatial awareness, and queue-backed operations

World projection requirement:

- `required`

Required views:

- `TacticalWorldView`
- `MinimapView`
- `GlobalVitalsView`
- `EventFeedView`
- `FleetTreeView`
- `InspectorTabsView`
- `SelectionSummaryView`
- `ActionDeckView`
- `QueueStatusView`

Optional views:

- `AttentionQueueView`
- `PinnedContextStripView`
- `ArtifactAccessView`
- `ReplayActivityView`

Required panels:

- `WorldViewportPanel`
- `MinimapPanel`
- `GlobalVitalsPanel`
- `EventRailPanel`
- `FleetRailPanel`
- `InspectorPanel`
- `BottomDeckPanel`

Required layout traits:

- `world_anchored_shell`
- `persistent_operational_frame`
- `persistent_edges`
- `selection_to_inspection_loop`
- `attention_without_takeover`
- `tiered_shell_visibility`
- `rail_compaction`
- `compound_bottom_deck`

### World / Camera / Minimap Rules

Add required linked-interaction primitives:

- `viewport_navigation`
- `camera_sync`

Rules:

- `TacticalWorldView` is the primary world interaction surface
- `MinimapView` is the authoritative overview and viewport locator
- `camera_sync` must propagate camera position, zoom, and frustum bidirectionally
  between world and minimap surfaces
- `viewport_navigation` may reposition camera only; it must not implicitly select
  targets or arm commands
- world picks may emit `selection` and optional `focus`
- minimap picks may emit `focus` only when explicitly used as locate or follow
- command targeting must remain distinct from navigation and camera movement

### Tiered Shell Visibility

- Tier 1
  - always-visible shell chrome: vitals, world viewport, minimap, compact rails,
    minimal inspector presence
- Tier 2
  - context-revealed operational deck: bottom deck opens on selection, command
    focus, or queue activity
- Tier 3
  - opaque overlays for deep inspection, forensics, settings, or blocking
    intervention; world interaction pauses, but canonical selection and draft
    context persist

### Collapsible Rails

- `EventRailPanel` and `FleetRailPanel` are independently collapsible persistent
  rails
- collapsed state becomes icon or badge strip, but linked participation survives
- collapse changes density, not semantic participation

### Compound Bottom Deck

- `BottomDeckPanel` contains three required sections:
  - `SelectionSummaryView`
  - `ActionDeckView`
  - `QueueStatusView`
- all three share brokered shell context but preserve different interaction
  semantics
- `ActionDeckView` is a fast contextual command surface with stable hotkey slots,
  not a free-form console replacement

### Coexistence With Existing Modes

- `battle_command` is a peer to `control_center` and `live_meeting`
- `control_center` remains broad orchestration-first monitoring
- `live_meeting` remains transcript-first and non-world-primary
- mode switches must preserve canonical selection, pins, and draft intent where
  the target mode supports the same references

