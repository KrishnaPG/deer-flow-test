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

## Linked Interaction Contract

Every linked interaction contract must define:

- participating views and panels
- required interaction paths
- canonical anchor record families for each path
- required stable join keys and filter dimensions
- required metadata at the point of use
- broker responsibilities
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

- `InspectorTabsView`
- `AttentionQueueView`
- `ArtifactAccessView`
- `TranscriptView`
- `ReplayActivityView`
- `SearchRetrievalResultsView`
- `MissionRailView`

Required receivers where declared:

- `CommandConsoleView`
- `IntentComposerView`
- intervention or approval surfaces

Degradation rule:

- may degrade to a prefilled draft only when the mediated intent path remains
  available

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

## Shell Mode Minimums

Every shell mode that declares linked behavior must define a broker for:

- `selection`
- `focus`
- `filtering`
- `replay cursor`, where temporal reasoning matters
- `command target`, where the shell mode is actionable

Every required panel in a shell mode must participate as at least a `source` or
`sink`.

Purely decorative required panels are not allowed.

Minimal required loops:

- every inspectable shell mode must link current selection into `InspectorPanel`
- every actionable shell mode must link current selection and/or pinned context
  into `CommandDeckPanel`
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

## Acceptable Degradation

Examples of acceptable degradation:

- an optional participant does not join the linked state
- an optional brush visualization degrades to explicit selection or filter chips
- an optional continuous range brush degrades to discrete bucket filtering
- optional ranking, embedding, or world enrichments are absent while canonical
  linked filtering still works

Examples of unacceptable degradation:

- required linkage becomes manual navigation instead of shared interaction state
- required bidirectional linkage becomes one-way only
- filters propagate visually but lose stable identity or provenance
- temporal or lineage correlation is approximated from display order alone

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
