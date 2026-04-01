### `viewport_navigation`

Purpose:

- reposition the active world camera without changing canonical target state

Required basis:

- viewport or camera target
- navigation origin
- zoom or extent semantics
- broker epoch

Allowed sources:

- `TacticalWorldView`
- `MinimapView`

Required receivers where declared:

- world camera surfaces
- minimap viewport locators

Degradation rule:

- may degrade to local-only navigation only when the shell mode does not require
  linked world/minimap navigation
- ordinary navigation-only camera movement remains intent-neutral by default
  unless the shell mode explicitly declares a navigation gesture as
  `prefill_seed`-capable

### `camera_sync`

Purpose:

- keep world and minimap camera state aligned bidirectionally

Required basis:

- camera position
- zoom
- frustum or bounds
- broker epoch
- origin panel and emission sequence

Allowed sources:

- `TacticalWorldView`
- `MinimapView`

Required receivers where declared:

- `WorldViewportPanel`
- `MinimapPanel`

Degradation rule:

- no acceptable degradation when a shell mode declares bidirectional sync as
  required

### `replay_cursor`

Purpose:

- identify the singular active temporal inspection point shared across
  replay-capable surfaces

Required basis:

- `sequence_id`, `event_time`, or `checkpoint_id`
- source stream or canonical record family
- broker epoch
- origin panel and emission sequence
- resolution mode: `live_tail`, `checkpoint`, or `historical_event`

Allowed sources:

- replay scrubbers
- `EventFeedView`
- `ReplayActivityView`
- world timelines

Required receivers where declared:

- replay/detail views
- inspector summaries
- queue/event surfaces
- world overlays

Degradation rule:

- may degrade to explicit checkpoint stepping only when temporal ordering,
  freshness visibility, and drill-down safety remain intact

### `temporal_range`

Purpose:

- express a bounded time window for brushing, filtering, compare alignment, or
  replay scope

Required basis:

- lower and upper temporal anchors
- anchor kind
- inclusivity semantics
- granularity
- open-endedness where relevant

Allowed sources:

- timelines
- replay tracks
- `EventFeedView`
- range controls

Required receivers where declared:

- replay/detail views
- sibling lists
- compare-capable surfaces
- world overlays

Degradation rule:

- may degrade from continuous range to explicit bounded buckets only when
  canonical temporal anchors remain stable

