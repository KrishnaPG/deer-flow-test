# Design: Modular Toolkit - New Generator Onboarding Contract

**Date:** 2026-04-04
**Status:** Approved

## Why This File Exists

The toolkit must accept new backend generators without letting each backend invent
its own consumer contract.

Every new generator must land into the same shared A/B storage families and must
become consumable through the same hierarchy-C presentation model, where `C:L0`
is the admissible A/B source pool for presentation composition, `C:L1` is any
optional presentation-side format/selection transform, and `C:L2` is the
canonical shell-defined consumer contract/query layer, implemented per
generator by SQL/materialized views or equivalent query projections.

This file is the checklist-heavy runbook for doing that safely.

## Core Rule

A new backend generator is considered onboarded only when all of the following
are true:

- its source objects land into shared A/B storage families
- its normalizer path produces shared A/B `L2+` rows
- every declared target shell mode can query it through registered `C:L2` views
- no app/tool/game needs direct `L0/L1` or generator-native reads

## Non-Negotiable Architecture

- Hierarchy A and Hierarchy B are the physical storage hierarchies in this slice.
- `L0` and `L1` are storage evidence and sanitation layers only.
- Consumers never query `L0` or `L1` directly.
- Hierarchy C is the presentation/query hierarchy only.
- `C:L0` is the presentation hierarchy's admissible source pool: all
  admissible `A:L2/L3/L4/L5/L6` and `B:L2/L3/L4/L5/L6` rows selected for
  presentation composition.
- `C:L1` is optional presentation-side format, selection, or reduction logic
  performed before shell-facing query surfaces are declared.
- `C:L2` is the minimum consumer query surface.
- `C:L2` is a catalog of SQL views/materialized views over admissible
  `A:L2/L3/L4/L5/L6` and `B:L2/L3/L4/L5/L6` inputs, optionally reduced through
  `C:L1` first.
- `B:L3/L4/L5/L6` rows are not a special exception in this contract; they are
  one important subset of admissible `B:L2+` inputs that some backends, such as
  Rowboat, may emphasize.
- A generator does not define app-facing canon.
- A shell/app does not bypass `C:L2` with app-local read glue.

## Presentation Hierarchy Rule

Hierarchy C must be read in three steps:

- `C:L0` - eligible A/B source rows for presentation composition
- `C:L1` - optional presentation-side transforms such as formatting, selection,
  deduplication, or shell-specific reduction
- `C:L2` - shell-defined canonical query contracts used by Game UI, tools, and
  apps, implemented by generator-supplied SQL/materialized views or equivalent
  query projections

The general rule is simple: if an `A:L2+` or `B:L2+` row is admissible for the
shell contract, it may enter `C:L0`.

`B:L3/L4/L5/L6` data is therefore not a special presentation path.

It is one example of admissible `B:L2+` input, and backends such as Rowboat may
simply contribute more value from that subset than from `B:L2` alone.

All such admissible rows are treated as `C:L0` inputs and reprojected into
generator-supplied `C:L2` implementations for the shell-defined canonical read
models of the target shell.

### Compact Rowboat Projection Example

When a Rowboat slice is rich in `B:L3/L4/L5/L6`, different shell modes may
still consume it through ordinary `C:L2` contracts:

- mission-summary shell: `B:L3` insight/aggregation rows plus supporting
  `B:L2` session/task rows enter `C:L0`, optionally reduce in `C:L1`, then
  surface as `C:L2` mission-summary and operator-brief views
- artifact-analysis shell: `B:L4` prediction/synthetic-artifact rows enter
  `C:L0` and project into `C:L2` comparison, provenance, and artifact-access
  views
- intervention shell: `B:L5` prescription/intervention rows enter `C:L0` and
  project into `C:L2` command-support and objective-support views
- replay/forensics shell: `B:L6` evaluation/deviation rows enter `C:L0` and
  project into `C:L2` replay-marker, outcome-comparison, and audit views

## Shared A/B Storage Families

Every new generator must map into these shared family ids instead of inventing
generator-specific consumer families.

### A families

- `a_session_snapshot`
- `a_message_event`
- `a_task_event`
- `a_artifact_event`
- `a_runtime_event`
- `a_intent_event`
- `a_exclusion_event`
- `a_replay_checkpoint`
- `a_backpressure_event`

### B families

- `b_session`
- `b_message`
- `b_task`
- `b_artifact`
- `b_artifact_access`
- `b_runtime_status`
- `b_intent`
- `b_exclusion`
- `b_conflict`
- `b_replay_window`
- `b_replay_checkpoint`
- `b_backpressure`
- `b_transform`

## High-Order B-Level Mapping Rule

All admissible `A:L2+` and `B:L2+` rows may feed `C:L0`.

Some generators, especially provenance-heavy or knowledge-heavy generators, may
still be strongest not at `B:L2` but at `B:L3/L4/L5/L6`.

Examples:

- `B:L3` insight or aggregation rows
- `B:L4` prediction, synthetic artifact, or forward projection rows
- `B:L5` prescription or intervention-plan rows
- `B:L6` evaluation, deviation, or outcome-comparison rows

Those rows remain B-level truth in storage, just like admissible `A:L2+` and
`B:L2` rows do.

For presentation, they become ordinary `C:L0` inputs and may be reprojected
into shell-specific `C:L2` canonical models through generator-supplied query
implementations.

### Required consequence

- a generator is allowed to be weak at `B:L2` but strong at `B:L3+`
- that generator still becomes GameUI-consumable only if its admissible
  `A:L2+`/`B:L2+` rows, including any emphasized `B:L3+` subset, can be
  projected into the shell-defined `C:L2` contracts through generator-supplied
  query implementations
- shell-facing canon is always judged at `C:L2`, not at the generator's natural
  B level

## Required C:L2 Contract Fields

Every shell-defined `C:L2` view contract registered for a generator-backed
slice must declare:

- `consumer_shell`
- `view_id`
- `sql_name`
- `view_kind`
- `row_grain`
- `required_join_keys`
- `source_families`
- `allowed_source_levels`
- `projected_columns`
- `abac_scope`
- `exclusion_behavior`
- `refresh_mode`
- `refresh_watermark`
- `refresh_dependencies`

Every shell-defined `C:L2` package should also state, where relevant:

- whether it consumes `C:L0` from `A`, `B`, or both
- whether it requires `C:L1` transforms before the final `C:L2` view contract
- whether it depends only on baseline `A:L2`/`B:L2` inputs or also on higher
  `A:L3+`/`B:L3+` inputs

For ownership, the rule is:

- the shell defines the `C:L2` contract because it owns the layouts, panels,
  hosted views, and canonical consumer-facing domain shape it expects
- the generator supplies the concrete SQL/materialized-view or equivalent query
  implementation because only the generator knows how to map its admissible A/B
  rows into that shell-defined contract

## Generator Onboarding Runbook

### Phase 1 - Source Mapping Gate

Before writing adapter code, the generator must have a source-mapping document
that answers:

- what source objects/events exist
- which shared A/B family each source binds to
- which immutable keys are available from source vs must be derived
- which lineage anchors exist
- which levels/planes the generator naturally populates
- which target shells/apps are expected to consume the data later

### Phase 1 Checklist

- [ ] every source object/event is mapped to one or more shared A/B family ids
- [ ] no proposed generator-specific family ids leak above raw-source handling
- [ ] immutable keys are identified for every mapped family
- [ ] correlation fields are identified (`mission_id`, `run_id`, `agent_id`,
      `artifact_id`, `trace_id` where applicable)
- [ ] level/plane occupancy is identified for each mapped source shape
- [ ] unsupported source shapes are called out explicitly rather than ignored
- [ ] a generator-specific source-mapping table exists for each major source
      shape, naming target A/B families, immutable keys, lineage anchors,
      level/plane occupancy, and unsupported-shape disposition
- [ ] target shell modes are named explicitly, including whether they consume
      `C:L0` from A, B, or both and which admissible `A:L2+`/`B:L2+` levels they
      project into `C:L2`

## Phase 2 - Raw Landing Gate

The raw-source adapter must land append-only storage rows, not app DTOs and not
consumer-facing view rows.

Every landed row must preserve:

- generator binding identity
- source object identity
- immutable keys
- lineage anchors
- level/plane metadata
- deterministic ordering where event sequencing matters

### Phase 2 Checklist

- [ ] the adapter lands A-family events into the shared A registry
- [ ] the adapter lands B-family seed rows where the source already provides
      enough information
- [ ] no consumer-facing `C:L2` rows are emitted from raw-source code
- [ ] no DeerFlow-only, Hermes-only, or generator-local family ids become shared
      consumer contract ids
- [ ] replay, exclusion, artifact-access, conflict, and backpressure shapes are
      landed when the source supports them
- [ ] landed rows include lineage, hashes or stable identities, and join keys
- [ ] when source semantics exist, the adapter lands or explicitly defers with
      rationale the A/B families needed by downstream shell packages

## Phase 3 - Normalization Gate

The normalizer path may sanitize and shape data, but it must stop at shared A/B
`L2+` rows.

Normalizers do not emit app-facing canon.

### Phase 3 Checklist

- [ ] raw `L0` and sanitized `L1` remain storage evidence only
- [ ] normalizers produce shared A/B family rows with explicit target-level
      metadata at `L2+`
- [ ] lineage is preserved across normalization (`derived_from`, source event
      anchors, supersession where applicable)
- [ ] hashes, joins, exclusions, and replay anchors survive normalization
- [ ] representation metadata needed for later `as_is`, `chunks`, and
      `embeddings` support is preserved
- [ ] normalizers do not emit `C:L2` SQL views or app-facing DTOs
- [ ] admissible `A:L2+`/`B:L2+` rows are preserved as presentation-eligible
      `C:L0` inputs rather than flattened into generator-local consumer shapes
- [ ] if the generator is strongest at `B:L3/L4/L5/L6`, the normalizer declares
      which emphasized `B:L3+` rows are intended to feed which shell-owned
      `C:L2` packages

## Phase 4 - Presentation Contract Gate

The generator becomes consumable only when every declared target shell/app has
registered `C:L2` query contracts and generator-supplied query implementations
over the shared A/B rows it now produces.

At this stage, the shell declares what it needs. The generator does not invent
the shell contract.

This is where admissible `A:L2+`/`B:L2+ -> C:L2` mapping becomes explicit,
including any Rowboat-like emphasis on `B:L3/L4/L5/L6`.

Examples:

- artifact-analysis shells may project `B:L4` artifact or prediction rows into
  `C:L2` comparison, provenance, and artifact-access views
- battle/world shells may project `B:L5` prescription rows into command,
  intervention, or objective-support `C:L2` views
- replay/forensics shells may project `B:L6` evaluation or deviation rows into
  outcome-comparison, replay-marker, or audit `C:L2` views
- baseline operational shells may project `A:L2` or `B:L2` rows into session,
  task, artifact, or runtime `C:L2` views without needing any special `B:L3+`
  exception path

### Phase 4 Checklist

- [ ] every declared target shell mode names its required `C:L2` view ids for
      this generator slice
- [ ] every declared `C:L2` view reads only from allowed A/B `L2+` levels
- [ ] every declared `C:L2` view specifies joins, row grain, projected columns,
      ABAC scope, exclusion behavior, and refresh metadata
- [ ] no shell view reads raw `L0/L1`
- [ ] battle/world shells remain downstream consumers of `C:L2`, not separate
      truth layers
- [ ] if world-primary/Game UI is in scope, the battle/world shell package is
      declared up front, including tactical summary, units, history, artifact,
      and minimap/camera-linked requirements where relevant
- [ ] if replay or forensics is in scope, the shell package declares temporal
      anchors such as `sequence_id`, `event_time`, `checkpoint_id`, replay
      cursor expectations, and outcome/deviation projections where needed
- [ ] the shell package declares the canonical `C:L2` read model shape it
      requires from the admissible `A:L2+`/`B:L2+` source pool, including any
      Rowboat-heavy `B:L3/L4/L5/L6` emphasis where relevant
- [ ] the generator supplies the SQL/materialized-view or equivalent query
      implementation that maps its admissible A/B rows into each declared
      shell-defined `C:L2` contract
- [ ] if multiple shell modes are claimed, the generator proves a complete
      `C:L2` package for each claimed shell mode, not just one narrow slice

## Phase 5 - End-To-End Proof Gate

No generator onboarding is complete without fixtures and proof queries.

The proof must demonstrate the full path:

`generator-native source -> raw landing -> normalized A/B L2+ rows -> registered C:L2 query results`

For generators with strong `B:L3+` semantics, the proof must also demonstrate:

`B:L3/L4/L5/L6 storage truth -> C:L0 presentation eligibility -> optional C:L1 transform -> generator-supplied implementation of shell-defined C:L2 query result`

### Phase 5 Checklist

- [ ] fixture coverage exists for representative thread/session/task/artifact
      shapes
- [ ] fixture coverage exists for replay/governance/pressure shapes when the
      generator supports them
- [ ] raw-source tests prove source objects bind to the correct A/B families
- [ ] normalizer tests prove shared `L2+` rows are emitted with lineage intact
- [ ] `C:L2` catalog tests prove the generator-backed shell views are registered
- [ ] query-level tests prove consumer-facing rows are available through `C:L2`
- [ ] no test requires direct app reads from raw generator payloads
- [ ] end-to-end proof executes registered `C:L2` SQL or equivalent query logic
      and asserts actual consumer-facing rows; catalog-registration-only tests
      are insufficient
- [ ] proof tests show how the claimed admissible `A:L2+`/`B:L2+` inputs project
      into shell-facing `C:L2` models; if `B:L3/L4/L5/L6` data is a generator
      strength, that emphasized subset is shown explicitly rather than treated as
      a special-case exception
- [ ] when multiple shell modes are claimed, proof tests cover each declared
      shell package rather than a single happy-path shell

## Readiness Gate

A generator is ready for product consumption only when every item below is true.

- [ ] all source objects map to shared A/B families
- [ ] no generator-specific family ids leak above raw-source handling
- [ ] raw landing preserves immutable keys, lineage, level, and plane metadata
- [ ] normalization produces shared A/B `L2+` rows rather than app-facing canon
- [ ] every declared target shell mode has a complete shell-defined `C:L2`
      contract package plus generator-supplied query implementation registered
- [ ] ABAC and exclusion behavior are enforced in the registered `C:L2` views
- [ ] world projection, if present, is downstream of `C:L2`
- [ ] end-to-end fixtures prove the generator is consumable without raw reads
- [ ] the claimed shell modes prove canonical `C:L2` projections from the
      admissible `A:L2+`/`B:L2+` source pool, with any generator-specific
      `B:L3/L4/L5/L6` strength shown as an illustrative subset rather than a
      separate rule
- [ ] if world-primary consumption is claimed, world/minimap/history/objective
      views are provided through `C:L2`, not a separate truth layer
- [ ] if replay or forensics consumption is claimed, temporal anchors and
      checkpoint/outcome semantics are proven in `C:L2`

If any checklist item above is incomplete, the generator is not onboarded.

## DeerFlow Reference Binding

DeerFlow is the first reference onboarding path for this contract.

Use these plan docs as the canonical execution sequence:

- `docs/superpowers/plans/backend-generator-to-canonical/000-overview.md`
- `docs/superpowers/plans/backend-generator-to-canonical/000a-task-0-storage-contract-and-promotion-policy.md`
- `docs/superpowers/plans/backend-generator-to-canonical/001-task-1-foundation-metadata.md`
- `docs/superpowers/plans/backend-generator-to-canonical/002-task-2-deerflow-raw-envelopes.md`
- `docs/superpowers/plans/backend-generator-to-canonical/003-task-3-deerflow-normalization.md`
- `docs/superpowers/plans/backend-generator-to-canonical/004-task-4-game-facing-derivations.md`
- `docs/superpowers/plans/backend-generator-to-canonical/005-task-5-world-projection.md`

Any future generator should follow the same runbook:

- bind into shared A/B families first
- normalize into shared A/B `L2+` rows second
- register shell-defined `C:L2` contracts plus generator-supplied query
  implementations third
- prove end-to-end consumption fourth

## Failure Rule

Do not mark a backend generator as supported for Game UI, apps, tools, or shell
consumers unless the full onboarding runbook has been satisfied.
