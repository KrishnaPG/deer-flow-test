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

