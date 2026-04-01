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

