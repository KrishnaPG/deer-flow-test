# Design: Modular Toolkit - Shell Mode Support Matrix

**Date:** 2026-04-01
**Status:** Draft revision

## Why This File Exists

`36-view-contract-support-model.md` pins `view contract` as the primary support
unit.

That still leaves one important gap: the cinematic shell is experienced as an
operating mode, not as an isolated view.

This file defines the missing shell-level layer so support can be judged as:

`backend -> canonical mappings -> view support -> panel support -> layout support -> shell mode support`

## Pinned Rule

- `shell mode` is the primary unit of operator experience
- `view contract` remains the primary atomic unit of backend support
- `panel` support is derived from hosted view support
- `layout` support is derived from required panel coexistence and capabilities
- `shell mode` support is derived from required views, required panels, required
  layout traits, linked dashboard interactions, mediated reads, and required
  intents

## Shell Mode Contract

Every `ShellModeContract` must define enough detail, directly or by normative
inheritance, to make shell-support judgment possible:

- `purpose`
- `HOTL/HITL affinity`
- `required views`
- `optional views`
- `required panels`
- `required layout traits`
- `required linked dashboard interactions`
- `required canonical record families`
- `required metadata`
- `required mediated reads`
- `required intents`, where actionable
- `required broker ownership map` for each required linked interaction type
- `required temporal semantics`, where the shell mode is replay-capable or
  live-streamed
- `required policy/access overlay behavior`, where exclusions, masking, or
  tombstones may affect linked state
- `required shell-state durability and rehydration rules`
- `required command-target behavior`, where the shell mode is actionable
- `world projection requirement`: `required`, `optional`, or `not_applicable`
- `degradation behavior`
- `unsupported conditions`

Shared inherited requirements:

- `Required Metadata Baseline` in this file applies to every shell mode unless a
  mode tightens it
- `39-linked-interaction-contract.md` provides the default linked-interaction,
  broker, metadata, degradation, and support-judgment rules for any shell mode
  that declares those interaction paths
- `41-final-integration-tranche.md` provides the default temporal, failover,
  policy-overlay, and intent-boundary rules for any shell mode that exposes
  those capabilities
- mode sections only need to restate inherited requirements when they narrow,
  extend, or override them

## Canonical Panel Contracts

Panels are role-based hosts, not pixel positions.

| Panel | Purpose | Required hosted views | Optional hosted views |
| --- | --- | --- | --- |
| `GlobalVitalsPanel` | persistent global health and burn framing | `GlobalVitalsView` | none |
| `MissionRailPanel` | mission/workflow/swarm navigation | `MissionRailView` | `ActivityRailView` |
| `AttentionPanel` | approvals, blockers, actionable interrupts | `AttentionQueueView` | `InterventionGateView` |
| `CommandDeckPanel` | command entry and intervention issuing | `CommandConsoleView` | `IntentComposerView` |
| `InspectorPanel` | current selection detail and actions | `InspectorTabsView` | `ArtifactAccessView` |
| `CenterCanvasPanel` | central transcluded working surface | at least one center-bearing hosted view appropriate to the shell mode; `CenterCanvasModeSwitcherView` is additionally required where `mode_switchable_center` is declared | `PinnedContextStripView`, `SearchRetrievalResultsView` |
| `ArtifactShelfPanel` | artifact browsing and attachment surfacing | `ArtifactShelfView` | `ArtifactAccessView` |
| `ReplayPanel` | replay and event-sequence inspection | `ReplayActivityView` | `ForensicsLogView` |
| `ModalDetailPanel` | fullscreen deep inspection | one of `ForensicsLogView`, `ArtifactAccessView`, `ReplayActivityView` | `SearchRetrievalResultsView` |
| `WorldCanvasPanel` | cinematic background or world-bearing projection host | no required hosted view for first-wave non-blocking shell support | future world-bearing views |
| `WorldViewportPanel` | primary world-bearing operational surface | `TacticalWorldView` | `ReplayActivityView`, `PinnedContextStripView` |
| `MinimapPanel` | authoritative overview and viewport locator | `MinimapView` | none |
| `EventRailPanel` | persistent temporal/event rail for live tail and backlog awareness | `EventFeedView` | `AttentionQueueView` |
| `FleetRailPanel` | persistent hierarchy/asset/mission rail | `FleetTreeView` | none |
| `BottomDeckPanel` | contextual command, selection summary, and queue state deck | `SelectionSummaryView`, `ActionDeckView`, `QueueStatusView` | `ArtifactAccessView` |

## Canonical Layout Traits

Layouts are judged by capability, not fixed geometry.

- `persistent_operational_frame`
  - global status and command entry remain available without entering a modal
- `persistent_edges`
  - mission/navigation and inspection can coexist while center work continues
- `dominant_center_canvas`
  - the center can host one primary work mode without collapsing the shell
- `selection_to_inspection_loop`
  - mission, artifact, replay, or world selections can open immediate detail
- `cross_view_brushing_and_filtering`
  - side-by-side views can participate in the shell mode's declared shared
    selection, brushing, filtering, and focus paths without becoming semantic
    silos
- `attention_without_takeover`
  - approvals and failures surface persistently without forcing every event into
    a modal
- `mode_switchable_center`
  - the center can switch among meeting, artifact, replay, and forensics modes
- `deep_inspection_escalation`
  - logs, lineage, and raw detail can move into fullscreen inspection when needed
- `world_anchored_shell`
  - the shell can coexist with a cinematic background or world projection layer
- `tiered_shell_visibility`
  - the shell supports always-visible world chrome, context-revealed
    operational decks, and blocking overlays without losing canonical selection
    or draft context
- `rail_compaction`
  - persistent rails may collapse to dense icon or badge strips without dropping
    linked participation
- `compound_bottom_deck`
  - a single panel can host distinct summary, command, and queue sections with
    shared brokered context but separate interaction semantics

## Required Metadata Baseline

All supported shell modes must preserve enough metadata to render, inspect,
filter, correlate, and drill down safely.

Required baseline metadata:

- `IdentityMeta`
- `CorrelationMeta`
- `LineageMeta`
- level and plane
- discovery object kind
- stable cross-view join keys and filter dimensions where dashboard linkage is
  required
- drill-down targets where relevant
- lifecycle and supersession semantics where relevant

## View Dependency Matrix

This matrix pins the first-wave and next-wave views discussed so far to their
minimum canonical dependencies.

| View | Required canonical record families | Required metadata extras | Canonical host panel |
| --- | --- | --- | --- |
| `SessionRunHeaderView` | `RunRecord`, `SessionRecord`, `RuntimeStatusRecord` | execution status, timing, correlation to active mission/session | `CenterCanvasPanel` |
| `TranscriptView` | `MessageRecord`, `ClarificationRecord`; optional `ToolCallRecord`, `ArtifactRecord` | speaker/source identity, ordering, provenance backlinks | `CenterCanvasPanel` |
| `IntentComposerView` | `SessionRecord`, `ArtifactRecord`, `IntentRecord` | current target scope, attachment identity, policy visibility | `CommandDeckPanel` |
| `InterventionGateView` | `ClarificationRecord`, `TaskRecord`, `RuntimeStatusRecord`, `IntentRecord` | blocking reason, deadline/urgency, consequence path | `AttentionPanel` |
| `ActivityRailView` | `TaskRecord`, `RuntimeStatusRecord`, `ToolCallRecord`, `DeliveryRecord` | ordering, status transitions, correlation to run/session | `MissionRailPanel` |
| `ArtifactShelfView` | `ArtifactRecord`; optional `AsIsRepresentationRecord` | artifact kind, availability, lineage anchors | `ArtifactShelfPanel` |
| `ArtifactAccessView` | `ArtifactRecord`, `AsIsRepresentationRecord`; optional `ChunkRecord`, `EmbeddingRecord` | retrieval mode, access policy, representation lineage | `InspectorPanel`, `ModalDetailPanel`, `ArtifactShelfPanel`, `BottomDeckPanel` |
| `ReplayActivityView` | `ReplayCheckpointRecord`, `RuntimeStatusRecord`, `MessageRecord`; optional `ToolCallRecord`, `ArtifactRecord` | stable timestamps or sequence IDs, replay boundaries, backlinks | `ReplayPanel`, `ModalDetailPanel`, `WorldViewportPanel` |
| `GlobalVitalsView` | `RuntimeStatusRecord`; optional `RunRecord` | global health, burn/cost, alert severity | `GlobalVitalsPanel` |
| `MissionRailView` | `RunRecord`, `SessionRecord`, `TaskRecord`, `RuntimeStatusRecord` | mission identity, progress state, focus target | `MissionRailPanel` |
| `AttentionQueueView` | `ClarificationRecord`, `RuntimeStatusRecord`; optional `ConflictRecord`, `ResolutionRecord` | severity, deadline, dedup lineage, actionable state | `AttentionPanel` |
| `CommandConsoleView` | `SessionRecord`, `ArtifactRecord`, `IntentRecord` | target context, accepted modes, submission status | `CommandDeckPanel` |
| `InspectorTabsView` | selected carrier records plus relevant semantic/representation records | selected entity identity, panel drill-down targets, policy visibility | `InspectorPanel` |
| `ForensicsLogView` | `ReplayCheckpointRecord`, `RuntimeStatusRecord`, `ToolCallRecord`, `TransformRecord` | sequence boundaries, filters, source backlinks | `ReplayPanel`, `ModalDetailPanel` |
| `CenterCanvasModeSwitcherView` | focused `RunRecord` and `SessionRecord` context; derived shell state | current center mode, allowed transitions, active selection | `CenterCanvasPanel` |
| `PinnedContextStripView` | `ArtifactRecord`, `MessageRecord`; optional `L0_SourceRecord` through `L6_OutcomeRecord` | pinned object identity, semantic level, retrieval backlinks | `CenterCanvasPanel`, `WorldViewportPanel` |
| `SearchRetrievalResultsView` | `ArtifactRecord`, `ChunkRecord`, `EmbeddingRecord`; optional `L1_SanitizedRecord` through `L4_PredictionRecord` | retrieval mode, rank/explanation, source lineage | `CenterCanvasPanel`, `ModalDetailPanel` |
| `TacticalWorldView` | `TaskRecord`, `RuntimeStatusRecord`; optional `ArtifactRecord`, `ReplayCheckpointRecord` | stable world object identity, spatial anchors, camera target IDs, policy visibility, drill-down backlinks | `WorldViewportPanel` |
| `MinimapView` | `TaskRecord`, `RuntimeStatusRecord` | viewport bounds, camera/frustum state, focus target IDs | `MinimapPanel` |
| `EventFeedView` | `RuntimeStatusRecord`, `MessageRecord`, `DeliveryRecord`; optional `ToolCallRecord`, `ReplayCheckpointRecord` | stable timestamps or sequence IDs, live-tail status, backlog/freshness markers | `EventRailPanel` |
| `FleetTreeView` | `RunRecord`, `TaskRecord`, `RuntimeStatusRecord` | hierarchy identity, focus target, rollout/health state | `FleetRailPanel` |
| `SelectionSummaryView` | selected carrier records plus relevant governance/representation records | selected entity identity, policy visibility, supersession/tombstone state | `BottomDeckPanel` |
| `ActionDeckView` | `SessionRecord`, `IntentRecord`; optional `ArtifactRecord`, `TaskRecord` | command target scope, policy visibility, validation state, hotkey slot identity | `BottomDeckPanel` |
| `QueueStatusView` | `TaskRecord`, `RuntimeStatusRecord`, `IntentRecord`; optional `ClarificationRecord`, `DeliveryRecord` | queue ordering, actionable blockage, freshness/staleness markers | `BottomDeckPanel` |

## Canonical Shell Modes

The shell should be judged by operating modes before it is judged by exact
screen geometry.

Mode interpretation rule:

- each shell mode below may list only the mode-specific requirements and deltas
  that are necessary to distinguish it
- any shell mode that declares a linked interaction path inherits the default
  broker, metadata, degradation, and support rules from
  `39-linked-interaction-contract.md` unless this file tightens them
- any shell mode that exposes temporal, failover, policy-overlay, or intent-
  boundary behavior inherits the default rules from
  `41-final-integration-tranche.md` unless this file tightens them
- required canonical record families, metadata, mediated reads, intents, and
  broker ownership may therefore be satisfied by explicit mode-local detail or
  by normative inheritance from the required views, panels, and referenced
  shell-level contracts
- when a shell mode introduces materially different or stricter behavior, it
  must state that behavior inline

### `control_center`

Purpose:

- broad HOTL/HITL operational awareness with bounded command, attention, and
  inspection loops

Required views:

- `GlobalVitalsView`
- `MissionRailView`
- `AttentionQueueView`
- `CommandConsoleView`
- `InspectorTabsView`
- `CenterCanvasModeSwitcherView`

Optional views:

- `ActivityRailView`
- `PinnedContextStripView`
- `ArtifactShelfView`

Required panels:

- `GlobalVitalsPanel`
- `MissionRailPanel`
- `AttentionPanel`
- `CommandDeckPanel`
- `InspectorPanel`
- `CenterCanvasPanel`

Required layout traits:

- `persistent_operational_frame`
- `persistent_edges`
- `dominant_center_canvas`
- `attention_without_takeover`
- `selection_to_inspection_loop`
- `cross_view_brushing_and_filtering`
- `mode_switchable_center`

World projection requirement:

- `optional`

Degradation rule:

- may degrade from cinematic world-bearing shell to atmospheric background shell,
  but may not lose command, attention, or inspect loops

Required linked dashboard interactions:

- `selection`
- `focus`
- `filtering`
- `pinning`
- `drill_down`
- `action_intent_emission`
- `command_target`

Required canonical record families:

- `RunRecord`
- `SessionRecord`
- `TaskRecord`
- `RuntimeStatusRecord`
- `ClarificationRecord`
- `IntentRecord`

Required metadata:

- `Required Metadata Baseline`
- target scope and policy visibility
- drill-down targets and lifecycle state

Required mediated reads:

- live-activity stream payload
- historical snapshot payload

Required intents:

- `operator intent`
- `intervention intent`
- `approval/denial intent`, where approval surfaces are present

Required broker ownership map:

- `selection`: `MissionRailPanel`
- `focus`: `InspectorPanel`
- `filtering`: `CenterCanvasPanel`
- `pinning`: `CenterCanvasPanel`
- `drill_down`: `InspectorPanel`
- `action_intent_emission`: `CommandDeckPanel`
- `command_target`: `CommandDeckPanel`

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
- linked gestures may prefill command context only
- `prefill -> draft -> validated -> submitted` requires explicit operator action

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

### `live_meeting`

Purpose:

- live HITL collaboration with transcript, pinned context, intervention, and
  artifact access in one shell

Required views:

- `SessionRunHeaderView`
- `TranscriptView`
- `IntentComposerView`
- `InspectorTabsView`

Optional views:

- `PinnedContextStripView`
- `ArtifactShelfView`
- `ArtifactAccessView`
- `InterventionGateView`

Required panels:

- `CenterCanvasPanel`
- `CommandDeckPanel`
- `InspectorPanel`

Required layout traits:

- `dominant_center_canvas`
- `selection_to_inspection_loop`
- `cross_view_brushing_and_filtering`
- `deep_inspection_escalation`

World projection requirement:

- `not_applicable`

Degradation rule:

- transcript-only fallback is allowed only if intent submission and detail
  inspection remain available

Required linked dashboard interactions:

- `selection`
- `focus`
- `pinning`
- `drill_down`
- `action_intent_emission`
- `command_target`

Required canonical record families:

- `RunRecord`
- `SessionRecord`
- `MessageRecord`
- `ClarificationRecord`
- `ArtifactRecord`
- `IntentRecord`

Required metadata:

- `Required Metadata Baseline`
- speaker/source identity and ordering
- target scope, attachment identity, and policy visibility

Required mediated reads:

- live-activity stream payload
- artifact pointer payload, where artifact access is exposed

Required intents:

- `operator intent`
- `clarification response`
- `intervention intent`, where intervention surfaces are present

Required broker ownership map:

- `selection`: `CenterCanvasPanel`
- `focus`: `InspectorPanel`
- `pinning`: `CenterCanvasPanel`
- `drill_down`: `InspectorPanel`
- `action_intent_emission`: `CommandDeckPanel`
- `command_target`: `CommandDeckPanel`

### `artifact_analysis`

Purpose:

- inspect, compare, retrieve, and pin artifacts and semantic derivatives across
  levels and planes

Required views:

- `ArtifactShelfView`
- `ArtifactAccessView`
- `InspectorTabsView`

Optional views:

- `PinnedContextStripView`
- `SearchRetrievalResultsView`
- `SessionRunHeaderView`

Required panels:

- `ArtifactShelfPanel`
- `InspectorPanel`
- `CenterCanvasPanel`

Required layout traits:

- `dominant_center_canvas`
- `selection_to_inspection_loop`
- `cross_view_brushing_and_filtering`
- `deep_inspection_escalation`

World projection requirement:

- `not_applicable`

Degradation rule:

- retrieval ranking and embedding-backed search may degrade, but lineage-safe
  artifact access may not

Required linked dashboard interactions:

- `selection`
- `focus`
- `filtering`
- `pinning`
- `drill_down`
- `compare`

Required canonical record families:

- `ArtifactRecord`
- `AsIsRepresentationRecord`
- `ChunkRecord`
- `EmbeddingRecord`

Required metadata:

- `Required Metadata Baseline`
- retrieval mode, rank/explanation, and representation lineage

Required mediated reads:

- artifact preview payload
- artifact pointer payload
- retrieval/search result payload
- historical snapshot payload

Required intents:

- none

Required broker ownership map:

- `selection`: `ArtifactShelfPanel`
- `focus`: `InspectorPanel`
- `filtering`: `CenterCanvasPanel`
- `pinning`: `ArtifactShelfPanel`
- `drill_down`: `InspectorPanel`
- `compare`: `CenterCanvasPanel`

### `replay`

Purpose:

- reconstruct sequence, causality, and operator-visible activity over time

Required views:

- `ReplayActivityView`
- `InspectorTabsView`

Optional views:

- `SessionRunHeaderView`
- `ActivityRailView`
- `ArtifactAccessView`

Required panels:

- `ReplayPanel`
- `InspectorPanel`
- `CenterCanvasPanel`

Required layout traits:

- `dominant_center_canvas`
- `selection_to_inspection_loop`
- `cross_view_brushing_and_filtering`
- `deep_inspection_escalation`

World projection requirement:

- `optional`

Degradation rule:

- event lists may replace richer replay visuals, but stable replay ordering and
  drill-down targets may not be lost

Required linked dashboard interactions:

- `selection`
- `focus`
- `drill_down`
- `pinning`
- `replay_cursor`
- `temporal_range`

Required canonical record families:

- `ReplayCheckpointRecord`
- `RuntimeStatusRecord`
- `MessageRecord`
- `ToolCallRecord`
- `ArtifactRecord`

Required metadata:

- `Required Metadata Baseline`
- stable timestamps or sequence IDs
- replay boundaries and freshness state

Required mediated reads:

- replay window payload
- historical snapshot payload
- live-activity stream payload, where live tail is exposed

Required intents:

- none

Required broker ownership map:

- `selection`: `ReplayPanel`
- `focus`: `InspectorPanel`
- `drill_down`: `InspectorPanel`
- `pinning`: `CenterCanvasPanel`
- `replay_cursor`: `ReplayPanel`
- `temporal_range`: `ReplayPanel`

### `forensics`

Purpose:

- deep investigation of failures, retries, ambiguities, and suspicious outcomes

Required views:

- `ForensicsLogView`
- `InspectorTabsView`

Optional views:

- `ReplayActivityView`
- `SearchRetrievalResultsView`
- `ArtifactAccessView`
- `InterventionGateView`

Required panels:

- `ModalDetailPanel`
- `InspectorPanel`

Required layout traits:

- `deep_inspection_escalation`
- `selection_to_inspection_loop`
- `cross_view_brushing_and_filtering`

World projection requirement:

- `not_applicable`

Degradation rule:

- if replay overlays are absent, time-sorted logs are acceptable; if log-source
  backlinks are absent, the mode is unsupported

Required linked dashboard interactions:

- `selection`
- `focus`
- `filtering`
- `drill_down`

Required canonical record families:

- `ReplayCheckpointRecord`
- `RuntimeStatusRecord`
- `ToolCallRecord`
- `TransformRecord`
- `ArtifactRecord`

Required metadata:

- `Required Metadata Baseline`
- sequence boundaries, source backlinks, and escalation targets

Required mediated reads:

- historical snapshot payload
- replay window payload, where replay overlays are exposed
- artifact pointer payload, where artifact access is exposed
- retrieval/search result payload, where retrieval overlays are exposed

Required intents:

- `intervention intent`, where intervention surfaces are present

Required broker ownership map:

- `selection`: `ModalDetailPanel`
- `focus`: `InspectorPanel`
- `filtering`: `ModalDetailPanel`
- `drill_down`: `InspectorPanel`

## Support Judgment Rules

### Shell-Supported

A backend supports a shell mode only when all of the following are true:

- all required views are `view-supported`
- all required panels derived from those views are supported
- all required layout traits are satisfiable by the supported panel set
- all required linked dashboard interactions are satisfiable across the hosted
  views and panels in that shell mode
- all required canonical record families are available through stable canonical
  mappings
- all required metadata is present at the point of use
- all required mediated reads are available through declared read contracts
- all required intents are available where the shell mode is actionable
- all required broker ownership maps are declared and satisfiable for each
  required linked interaction type
- shells with temporal behavior preserve broker epoch semantics, freshness
  visibility, and late-event safety
- shells with policy overlays invalidate or tombstone linked state on relevant
  `policy_epoch` changes
- required world projections, if any, preserve drill-down backlinks to supported
  detail views

Required linked dashboard interactions may include:

- shared selection propagation
- brushing from one view into filters on sibling views
- shell-level filter state applied across multiple hosted views
- focus propagation from rails, queues, replay, or search into inspector and
  center views

Required linked dashboard interactions are mode-specific.

- `filtering` is required only where the shell mode explicitly declares shared
  filter behavior
- world-primary shells are not `full` unless navigation, camera sync, and
  command-target separation are all preserved

### Coverage Labels

- `full`
  - all required and optional shell contracts are supported
- `partial`
  - all required shell contracts are supported, but one or more optional views,
    enrichments, or world-bearing extensions degrade according to the declared
    shell mode contract
- `unsupported`
  - one or more required shell contracts cannot be satisfied

### Partial Support Constraint

Do not use `partial` unless the operator can still:

- enter the shell mode
- inspect required detail
- traverse required drill-downs
- preserve stable identity and provenance
- perform required cross-view brushing and filtering when the shell mode depends
  on it
- issue required intents, if the shell mode is actionable

If any of those fail, the shell mode is `unsupported`.

## Anti-Drift Rules

- Do not claim shell support from backend nouns alone.
- Do not treat raw payloads as canonical support.
- Do not treat direct database or storage access as mediated read support.
- Do not treat direct mutation endpoints as intent support.
- Do not treat side-by-side hosted views as independent silos when the shell mode
  requires linked dashboard behavior.
- Do not let world projection invent semantics that cannot be traced to
  supported canonical records and supported detail views.
- Do not label a shell mode `full` when required metadata is missing and the UI
  would need to guess identity, lineage, level, plane, or lifecycle.

## Next Step This Enables

The next backend evaluation pass should now judge DeerFlow, Hermes, PocketFlow,
and Rowboat against:

- required canonical mappings per shell mode
- required view support
- derived panel support
- derived layout trait support
- final `full` / `partial` / `unsupported` shell mode support
