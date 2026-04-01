### `pinning`

Purpose:

- preserve important context across mode switches and selection churn

Required basis:

- stable object ID
- representation kind where relevant
- level and plane where relevant
- pin source
- freshness and supersession semantics
- reopen target

Allowed sources:

- transcript
- artifact access
- replay
- search results
- inspector
- world picks

Required receivers where declared:

- `PinnedContextStripView`
- `InspectorPanel`
- active center mode
- compare-capable surfaces

Degradation rule:

- may degrade from live-updating pin to frozen snapshot only when staleness is
  shown explicitly

### `drill_down`

Purpose:

- move from summary or aggregate state into deeper detail without losing context

Required basis:

- explicit drill-down target
- canonical target IDs
- breadcrumb or backlink chain
- level and plane transform where relevant
- provenance anchors

Allowed sources:

- any summary, replay, shelf, search, world, or inspector surface

Required receivers where declared:

- `InspectorPanel`
- `CenterCanvasPanel`
- `ModalDetailPanel`

Degradation rule:

- may reroute to a generic supported detail host only if breadcrumbs and
  provenance remain intact

### `compare`

Purpose:

- hold two or more entities, checkpoints, runs, or representations for side-by-
  side analysis

Required basis:

- stable comparable IDs
- compare axis such as `version`, `run`, `time`, or `representation`
- normalization dimensions
- ordering or alignment metadata
- lineage or temporal anchors

Allowed sources:

- shelves
- search results
- replay checkpoints
- inspector picks
- pinned context

Required receivers where declared:

- compare-capable inspector tabs
- center compare modes
- modal detail surfaces

Degradation rule:

- may degrade to pairwise compare only when the shell mode does not require
  larger compare sets

