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

