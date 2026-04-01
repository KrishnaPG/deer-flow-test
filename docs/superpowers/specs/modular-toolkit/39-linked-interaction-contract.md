# Design: Modular Toolkit - Linked Interaction Contract

**Date:** 2026-04-01
**Status:** Draft revision

## Why This File Exists

The current spec line now explicitly rejects view silos.

That creates one more missing layer between isolated view support and actual shell
support: a canonical linked-interaction contract.

This file defines the shared interaction layer that lets co-present views,
panels, and shell modes behave as one dashboard.

## Pinned Rule

- linked interaction is shell-level behavior, not a private feature of one view
- linked interaction state is local, shared, and non-canonical
- canonical records remain immutable storage-backed truth
- linked interaction must use declared canonical references, join keys, and
  mediated reads
- side-by-side hosted views do not count as linked unless they participate in the
  same declared interaction contract
- navigational world gestures and camera movement are not command-target changes
- linked state must remain deterministic across broker failover and policy
  invalidation
- `filtering` is required only for shell modes that declare shared filter
  semantics
- `prefill_seed` is broader than `action_intent_emission`; `command_target` and
  explicitly allowed linked-gesture paths may seed or refresh shell-local
  `prefill` without becoming editable or submittable intent state

## Linked Interaction Contract

Every linked interaction contract must define:

- participating views and panels
- required interaction paths
- canonical anchor record families for each path
- required stable join keys and filter dimensions
- required metadata at the point of use
- broker responsibilities
- broker owner by interaction type
- state durability and rehydration classification
- temporal/failover semantics, where relevant
- policy invalidation behavior, where relevant
- command-surface lifecycle entry boundaries, where the shell mode is
  actionable
- degradation behavior
- unsupported conditions

## Canonical Linked-Interaction Primitives

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

### `action_intent_emission`

Purpose:

- emit a canonical action candidate from any qualifying hosted view into mediated
  command flow

Required basis:

- actor and session context
- target IDs
- intent kind
- parameter references
- policy visibility
- consequence or validation state where relevant

Allowed sources:

- `ActionDeckView`
- `InspectorTabsView`
- `AttentionQueueView`
- `ArtifactAccessView`
- `TranscriptView`
- `ReplayActivityView`
- `SearchRetrievalResultsView`
- `MissionRailView`

Required receivers where declared:

- `ActionDeckView`
- `CommandConsoleView`
- `IntentComposerView`
- intervention or approval surfaces

Degradation rule:

- may degrade to shell-local `prefill` only when the mediated intent path
  remains available

### `prefill_seed`

Purpose:

- create or refresh shell-local `prefill` intent context without granting
  editable ownership or submit authority

Allowed sources:

- any declared `action_intent_emission` source
- any declared `command_target` source
- any explicitly allowed linked-gesture seed source declared by the shell mode

Allowed effect:

- create `prefill`
- refresh `prefill`
- update suggested target, scope, parameter, or provenance context

Forbidden effect:

- create `draft` automatically
- enter `validated`
- enter `submitted`
- append any canonical `IntentRecord`

Boundary rule:

- `prefill_seed` remains shell-local only; explicit operator promotion is
  required for `prefill -> draft`

### `command_target`

Purpose:

- normalize the current actionable target bundle for mediated command flow

Required basis:

- actor and session context
- canonical target refs
- scope
- provenance
- policy visibility
- broker sequence or epoch

Allowed sources:

- selection-capable views
- `InspectorTabsView`
- `ActionDeckView`
- `CommandConsoleView`
- `IntentComposerView`
- `ReplayActivityView`
- world picks

Required receivers where declared:

- `ActionDeckView`
- `CommandConsoleView`
- `IntentComposerView`
- intervention or approval surfaces

Degradation rule:

- may degrade to shell-local `prefill` context only when explicit operator
  promotion into editable `draft` remains required
- declared `command_target` sources may seed or refresh shell-local `prefill`,
  but may not advance beyond `prefill` without explicit operator promotion

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

## Shared Shell Interaction State

Linked dashboard behavior requires shell-local shared state.

That state is non-canonical and must never mutate storage truth.

Required shared state slices:

- `selection_state`
  - current selected canonical references
- `filter_state`
  - normalized active filter predicates with `global`, `panel`, or `view` scope
- `brush_state`
  - transient exploratory range, cohort, region, or timeline state
- `focus_state`
  - singular active target driving inspector, center, or modal routing
- `pin_set`
  - intentionally retained canonical references across shell navigation
- `compare_set`
  - bounded ordered set of comparable canonical references
- `drill_down_targets`
  - resolvable shell-local targets that map references to supported detail hosts
- `intent_draft_context`
  - unsent operator composition context for command and intervention flow
- `command_target_state`
  - normalized actionable target bundle for command surfaces
- `viewport_state`
  - current world viewport/camera target state
- `camera_sync_state`
  - brokered world/minimap camera alignment state
- `replay_cursor_state`
  - singular active temporal inspection point
- `temporal_range_state`
  - bounded active temporal window
- `stream_lifecycle_state`
  - per-surface `connecting` / `live` / `catching_up` / `degraded` /
    `stalled` / `recovered` / `closed` state
- `policy_overlay_state`
  - current exclusion, masking, tombstone, `policy_epoch`, and `policy_reason`
    overlays affecting linked interaction
- `broker_epoch_map`
  - current broker epoch by interaction type for failover-safe propagation

## Canonical Reference Discipline

Shell state must reference canonical truth through typed references rather than
embedded mutable payloads.

Every durable linked reference should preserve enough information to resolve the
target safely, including:

- canonical record family
- primary identity
- required correlation keys
- representation-specific identity where relevant
- lineage or source identity where required

Do not use render position, array position, or backend-native opaque handles as
the durable basis of linked interaction state.

## Panel Participation Model

Panels participate in linked interaction through explicit roles.

### Roles

- `source`
  - originates linked state changes from operator action
- `sink`
  - receives linked state and re-renders, highlights, filters, or scopes detail
- `broker`
  - owns shell-level coordination for a given interaction type and republishes it
- `mirror`
  - reflects brokered state for orientation or continuity without becoming the
    authority

Each interaction type must have one clear broker.

### Canonical Panel Participation

| Panel | Default roles | Minimum participation |
| --- | --- | --- |
| `CenterCanvasPanel` | `source`, `sink`, optional `broker` | must originate and receive selection, focus, replay cursor, and pinned context where the shell mode uses them |
| `MissionRailPanel` | `source`, `sink` | must originate mission/session/run focus and reflect active shell filters and focus |
| `InspectorPanel` | `sink`, selective `source` | must resolve current shell selection into detail and actions, and may originate secondary drill-downs |
| `AttentionPanel` | `source`, `sink` | must push actionable interrupts into command, center, and inspector flows |
| `CommandDeckPanel` | `source`, `sink`, optional `broker` | must consume target scope and pinned context, and originate mediated intents |
| `ReplayPanel` | `source`, `sink` | must originate replay cursor or range and reflect externally selected checkpoints or ranges |
| `ArtifactShelfPanel` | `source`, `sink`, `mirror` | must originate artifact selection and pinning while preserving lineage and representation identity |
| `WorldViewportPanel` | `source`, `sink` | must originate world selection, focus, and viewport navigation and must consume brokered camera, temporal, and command-target context without silently arming commands |
| `MinimapPanel` | `source`, `sink`, `mirror` | must reflect current viewport and may originate viewport navigation and explicit locate or follow focus without taking command authority |
| `EventRailPanel` | `source`, `sink` | must originate and receive `replay_cursor`, `temporal_range`, and event-linked selection while preserving live-tail versus historical state |
| `FleetRailPanel` | `source`, `sink` | must originate hierarchy-driven selection/focus and reflect current command-target and policy-invalidated state |
| `BottomDeckPanel` | `sink`, selective `source` | must consume brokered selection, command-target, queue, and policy state, and may originate explicit command-surface actions through declared command views only |

## Shell Mode Minimums

Every shell mode that declares linked behavior must define a broker for:

- `selection`
- `focus`

Additionally, a shell mode must define a broker for:

- `filtering`, only where the shell mode declares shared filter behavior
- `replay_cursor` and `temporal_range`, where temporal reasoning matters
- `command_target`, where the shell mode is actionable
- `camera_sync` and `viewport_navigation`, where the shell mode is world-primary

Every required panel in a shell mode must participate as at least a `source` or
`sink`.

Purely decorative required panels are not allowed.

Minimal required loops:

- every inspectable shell mode must link current selection into `InspectorPanel`
- every actionable shell mode must link current selection and/or pinned context
  into its declared command surface
- every artifact-bearing shell mode must support artifact selection, pinning, and
  drill-down propagation across participating panels
- every temporally reasoning shell mode must support replay-style cursor or range
  propagation, even if the visual host is an ordered event list rather than a
  rich replay timeline

## Joinability And Metadata Requirements

Linked interaction support requires all of the following where the shell mode
declares them:

- stable canonical join keys across participating views
- normalized filter dimensions with compatible value semantics
- `IdentityMeta`
- `CorrelationMeta`
- `LineageMeta`
- level and plane where semantic cross-view filtering depends on them
- stable timestamps or sequence IDs where temporal brushing or replay alignment
  is required
- broker epoch or equivalent recovery sequence where linked recovery is
  supported
- freshness timestamp or last applied `sequence_id`
- explicit stale reason and staleness budget where live streams or queue state
  are shown
- `policy_epoch` and `policy_reason` where policy overlays may invalidate
  linked state
- exclusion or tombstone markers where objects may disappear from direct access
- spatial anchors and viewport target IDs where world/minimap linkage is
  required
- drill-down targets and provenance backlinks for escalated detail
- lifecycle and supersession semantics where linked context may outlive the
  source surface

Labels, display order, screen position, and styling are never valid join keys.

## Support Judgment

### `full`

- all required and optional linked interaction paths work as declared
- every required participant can publish and consume linked state as declared
- propagation uses stable canonical join keys and required metadata without
  semantic guessing
- linked state preserves identity, provenance, and drill-down safety end to end

### `partial`

- all required linked interaction paths still work
- one or more optional participants, optional dimensions, or optional
  enrichments degrade only as declared
- degraded behavior still preserves stable identity, provenance, and drill-down
  safety

### `unsupported`

- any required linked interaction path cannot be satisfied
- any required participant cannot publish or consume the required linked state
- required linkage would depend on guessed joins, lossy identity, missing
  provenance, or unsafe drill-downs
- navigational gestures can implicitly arm or retarget commands
- broker recovery can resume under the same epoch after failover
- excluded references remain in selection, compare, focus, or drill-down paths
  without tombstone or removal

## Acceptable Degradation

Examples of acceptable degradation:

- an optional participant does not join the linked state
- an optional brush visualization degrades to explicit selection or filter chips
- an optional continuous range brush degrades to discrete bucket filtering
- optional ranking, embedding, or world enrichments are absent while canonical
  linked filtering still works
- explicit checkpoint stepping may replace continuous temporal scrubbing when
  canonical temporal anchors, freshness visibility, and drill-down safety remain
  intact

Examples of unacceptable degradation:

- required linkage becomes manual navigation instead of shared interaction state
- required bidirectional linkage becomes one-way only
- filters propagate visually but lose stable identity or provenance
- temporal or lineage correlation is approximated from display order alone
- world and minimap silently diverge when bidirectional sync is required

## Anti-Drift Rules

- Do not claim linkage from side-by-side placement alone.
- Do not claim linkage from matching labels, colors, or simultaneous updates
  alone.
- Do not claim linkage because views read from the same backend unless they share
  declared canonical join keys and linked-state semantics.
- Do not claim filtering support when each view applies independent local filters
  without shared canonical state.
- Do not claim brushing support when highlight coincidence cannot be traced to
  stable canonical records.
- Do not claim `action_intent_emission` unless the emitted action passes through
  the mediated intent model.
- Do not claim linked dashboard support if selection, filtering, brushing,
  focus, or drill-down require direct storage access or non-mediated reads.

## Design Consequence

Backend adapters alone should not be expected to prove linked shell behavior.

The reusable toolkit layer must own the composition logic that turns isolated
view support into:

- shared selection
- brush and filter propagation
- focus routing
- pinning and compare state
- drill-down-safe escalation
- mediated action intent emission
