### `selection`

Purpose:

- nominate the current canonical entity or entity set for shared inspection

Required basis:

- stable canonical identity
- canonical record family
- selection scope: `single` or `set`
- provenance backlink or declared correlation target

Allowed sources:

- rails
- queues
- transcript rows
- artifact shelves
- replay items
- search results
- world picks

Required receivers where declared:

- `InspectorPanel`
- `CenterCanvasPanel`
- `ModalDetailPanel`
- `PinnedContextStripView`

Degradation rule:

- may degrade to local-only selection only when cross-view selection is optional

### `brushing`

Purpose:

- preview a correlated subset before commitment

Required basis:

- declared brush dimension or range
- typed value semantics
- temporal or quantitative bucket semantics where relevant
- ephemeral lifetime semantics

Allowed sources:

- timelines
- replay tracks
- maps/world surfaces
- tables
- result lists

Required receivers where declared:

- sibling lists
- inspector summaries
- replay/detail views
- world overlays

Degradation rule:

- may degrade to origin-only highlight
- must not silently become `filtering` without explicit promotion

### `filtering`

Purpose:

- persist inclusion and exclusion state across multiple hosted views

Required basis:

- declared filter dimensions
- typed operators and values
- null handling semantics
- temporal granularity where relevant

Allowed sources:

- filter bars
- facet lists
- brush promotion actions
- inspector constraints
- command/context surfaces

Required receivers where declared:

- all views that declare the filtered dimensions

Degradation rule:

- degradation is allowed only when unsupported dimensions are surfaced explicitly

### `focus`

Purpose:

- elevate one entity, checkpoint, region, or slice as the primary inspection
  target

Required basis:

- stable focus target ID
- target kind
- temporal position or checkpoint where relevant
- drill-down target
- lineage backlink

Allowed sources:

- selection-capable views
- replay scrubbers
- world surfaces
- inspector tabs
- search results

Required receivers where declared:

- `InspectorPanel`
- active center view
- `ModalDetailPanel`
- world camera or projection surfaces

Degradation rule:

- may degrade to selection plus local centering only when shell-wide focus is not
  required

