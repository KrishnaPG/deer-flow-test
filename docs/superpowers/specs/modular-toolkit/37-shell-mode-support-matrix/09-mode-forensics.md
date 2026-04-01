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

