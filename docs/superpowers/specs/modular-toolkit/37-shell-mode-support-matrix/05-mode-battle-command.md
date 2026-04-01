### `battle_command`

Purpose:

- world-primary HOTL/HITL shell for tactical supervision, rapid command issuing,
  spatial awareness, and queue-backed operations

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

Required linked dashboard interactions:

- `selection`
- `focus`
- `pinning`
- `drill_down`
- `action_intent_emission`
- `command_target`
- `viewport_navigation`
- `camera_sync`
- `replay_cursor`
- `temporal_range`

Required canonical record families:

- `RunRecord`
- `SessionRecord`
- `TaskRecord`
- `RuntimeStatusRecord`
- `MessageRecord`
- `IntentRecord`
- `ReplayCheckpointRecord`

Required metadata:

- `IdentityMeta`
- `CorrelationMeta`
- `LineageMeta`
- stable spatial anchors and world-object identity
- stable timestamps or sequence IDs
- freshness timestamp or last applied `sequence_id`
- staleness budget and explicit stale reason
- `policy_epoch` and `policy_reason`
- exclusion/tombstone visibility state
- drill-down targets and reopen safety markers

Required mediated reads:

- live-activity stream payload
- historical snapshot payload
- replay window payload, where temporal inspection is exposed
- artifact pointer payload, where artifact access is exposed

Required intents:

- `operator intent`
- `intervention intent`, where interrupt-handling actions are exposed
- `approval/denial intent`, only where approval surfaces are present

Required broker ownership map:

- `selection`: `WorldViewportPanel`
- `focus`: `InspectorPanel`
- `pinning`: `InspectorPanel`
- `drill_down`: `InspectorPanel`
- `action_intent_emission`: `BottomDeckPanel`
- `command_target`: `BottomDeckPanel`
- `viewport_navigation`: `WorldViewportPanel`
- `camera_sync`: `MinimapPanel`
- `replay_cursor`: `EventRailPanel`
- `temporal_range`: `EventRailPanel`

Required `prefill_seed` paths:

- all declared `action_intent_emission` sources
- all declared `command_target` sources
- world picks
- replay-anchor gestures from replay-capable surfaces
- attention interrupts that surface a candidate intervention context

Not seed-capable by default:

- ordinary navigation-only camera movement remains intent-neutral by default,
  including ordinary camera pan, ordinary camera zoom, and plain viewport
  repositioning without explicit command-context intent

Required temporal semantics:

- singular `replay_cursor` shared across replay-capable surfaces
- bounded `temporal_range` where temporal brushing or replay scope is shown
- operator-visible live-tail versus historical state
- late events inserted by canonical temporal anchors, not append order

Required policy/access overlay behavior:

- excluded references drop or tombstone immediately
- masked references remain joinable only while safe drill-down survives
- large-object access uses mediated handoff only

Required shell-state durability and rehydration rules:

- local state classified as `discardable`, `rehydratable`, `recoverable_draft`,
  or `non_rehydratable`
- rehydration resolves through canonical references, never cached payloads
- broker recovery resumes only under a new epoch

Required command-target behavior:

- navigation and camera movement may not implicitly select or arm commands
- declared `prefill_seed` paths may create or refresh shell-local `prefill`
  only
- `prefill -> draft -> validated -> submitted` requires explicit operator
  promotion

World projection requirement:

- `required`

Degradation rule:

- world rendering richness, optional replay overlays, and optional artifact
  enrichments may degrade, but world selection, minimap-linked camera state,
  command-target safety, and queue-backed operator action may not

Unsupported conditions:

- world/minimap camera linkage is absent or one-way when bidirectional sync is
  required
- viewport navigation implicitly selects targets or arms commands
- collapsed rails lose linked participation rather than density only
- excluded or tombstoned references remain actionable or drill-down-safe
- late events are applied by append order rather than canonical temporal anchors

