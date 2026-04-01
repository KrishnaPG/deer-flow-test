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

Every `ShellModeContract` must define:

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
- `world projection requirement`: `required`, `optional`, or `not_applicable`
- `degradation behavior`
- `unsupported conditions`

## Canonical Panel Contracts

Panels are role-based hosts, not pixel positions.

| Panel | Purpose | Required hosted views | Optional hosted views |
| --- | --- | --- | --- |
| `GlobalVitalsPanel` | persistent global health and burn framing | `GlobalVitalsView` | none |
| `MissionRailPanel` | mission/workflow/swarm navigation | `MissionRailView` | `ActivityRailView` |
| `AttentionPanel` | approvals, blockers, actionable interrupts | `AttentionQueueView` | `InterventionGateView` |
| `CommandDeckPanel` | command entry and intervention issuing | `CommandConsoleView` | `IntentComposerView` |
| `InspectorPanel` | current selection detail and actions | `InspectorTabsView` | `ArtifactAccessView` |
| `CenterCanvasPanel` | central transcluded working surface | `CenterCanvasModeSwitcherView` plus at least one of `SessionRunHeaderView`, `TranscriptView`, `ArtifactAccessView`, `ReplayActivityView`, or `ForensicsLogView` depending on shell mode | `PinnedContextStripView`, `SearchRetrievalResultsView` |
| `ArtifactShelfPanel` | artifact browsing and attachment surfacing | `ArtifactShelfView` | `ArtifactAccessView` |
| `ReplayPanel` | replay and event-sequence inspection | `ReplayActivityView` | `ForensicsLogView` |
| `ModalDetailPanel` | fullscreen deep inspection | one of `ForensicsLogView`, `ArtifactAccessView`, `ReplayActivityView` | `SearchRetrievalResultsView` |
| `WorldCanvasPanel` | cinematic background or world-bearing projection host | no required hosted view for first-wave non-blocking shell support | future world-bearing views |

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
  - side-by-side views can participate in shared selection, brushing, filtering,
    and focus without becoming semantic silos
- `attention_without_takeover`
  - approvals and failures surface persistently without forcing every event into
    a modal
- `mode_switchable_center`
  - the center can switch among meeting, artifact, replay, and forensics modes
- `deep_inspection_escalation`
  - logs, lineage, and raw detail can move into fullscreen inspection when needed
- `world_anchored_shell`
  - the shell can coexist with a cinematic background or world projection layer

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
| `ArtifactAccessView` | `ArtifactRecord`, `AsIsRepresentationRecord`; optional `ChunkRecord`, `EmbeddingRecord` | retrieval mode, access policy, representation lineage | `InspectorPanel`, `ModalDetailPanel`, `ArtifactShelfPanel` |
| `ReplayActivityView` | `ReplayCheckpointRecord`, `RuntimeStatusRecord`, `MessageRecord`; optional `ToolCallRecord`, `ArtifactRecord` | stable timestamps or sequence IDs, replay boundaries, backlinks | `ReplayPanel`, `ModalDetailPanel` |
| `GlobalVitalsView` | `RuntimeStatusRecord`; optional `RunRecord` | global health, burn/cost, alert severity | `GlobalVitalsPanel` |
| `MissionRailView` | `RunRecord`, `SessionRecord`, `TaskRecord`, `RuntimeStatusRecord` | mission identity, progress state, focus target | `MissionRailPanel` |
| `AttentionQueueView` | `ClarificationRecord`, `RuntimeStatusRecord`; optional `ConflictRecord`, `ResolutionRecord` | severity, deadline, dedup lineage, actionable state | `AttentionPanel` |
| `CommandConsoleView` | `SessionRecord`, `ArtifactRecord`, `IntentRecord` | target context, accepted modes, submission status | `CommandDeckPanel` |
| `InspectorTabsView` | selected carrier records plus relevant semantic/representation records | selected entity identity, panel drill-down targets, policy visibility | `InspectorPanel` |
| `ForensicsLogView` | `ReplayCheckpointRecord`, `RuntimeStatusRecord`, `ToolCallRecord`, `TransformRecord` | sequence boundaries, filters, source backlinks | `ReplayPanel`, `ModalDetailPanel` |
| `CenterCanvasModeSwitcherView` | focused `RunRecord` and `SessionRecord` context; derived shell state | current center mode, allowed transitions, active selection | `CenterCanvasPanel` |
| `PinnedContextStripView` | `ArtifactRecord`, `MessageRecord`; optional `L0_SourceRecord` through `L6_OutcomeRecord` | pinned object identity, semantic level, retrieval backlinks | `CenterCanvasPanel` |
| `SearchRetrievalResultsView` | `ArtifactRecord`, `ChunkRecord`, `EmbeddingRecord`; optional `L1_SanitizedRecord` through `L4_PredictionRecord` | retrieval mode, rank/explanation, source lineage | `CenterCanvasPanel`, `ModalDetailPanel` |

## Canonical Shell Modes

The shell should be judged by operating modes before it is judged by exact
screen geometry.

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
- required world projections, if any, preserve drill-down backlinks to supported
  detail views

Required linked dashboard interactions may include:

- shared selection propagation
- brushing from one view into filters on sibling views
- shell-level filter state applied across multiple hosted views
- focus propagation from rails, queues, replay, or search into inspector and
  center views

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
