# Design: Modular Toolkit - New Generator Onboarding Contract

**Date:** 2026-04-04
**Status:** Approved

## Why This File Exists

The toolkit must accept new backend generators without letting each backend invent
its own consumer contract.

Every new generator must land into the same shared A/B storage families and must
become consumable through the same `C:L2` SQL/materialized-view presentation
layer.

This file is the checklist-heavy runbook for doing that safely.

## Core Rule

A new backend generator is considered onboarded only when all of the following
are true:

- its source objects land into shared A/B storage families
- its normalizer path produces shared A/B `L2+` rows
- at least one consumer shell can query it through registered `C:L2` views
- no app/tool/game needs direct `L0/L1` or generator-native reads

## Non-Negotiable Architecture

- Hierarchy A and Hierarchy B are the physical storage hierarchies in this slice.
- `L0` and `L1` are storage evidence and sanitation layers only.
- Consumers never query `L0` or `L1` directly.
- Hierarchy C is the presentation/query hierarchy only.
- `C:L2` is the minimum consumer query surface.
- `C:L2` is a catalog of SQL views/materialized views over `A:L2/L3/L4/L5/L6`
  and `B:L2/L3/L4/L5/L6`.
- A generator does not define app-facing canon.
- A shell/app does not bypass `C:L2` with app-local read glue.

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

## Required C:L2 Contract Fields

Every shell-defined `C:L2` view registered for a generator-backed slice must
declare:

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

## Phase 4 - Presentation Contract Gate

The generator becomes consumable only when a target shell/app has registered
`C:L2` query views over the shared A/B rows it now produces.

At this stage, the shell declares what it needs. The generator does not invent
the shell contract.

### Phase 4 Checklist

- [ ] at least one shell declares required `C:L2` view ids for this generator
      slice
- [ ] every declared `C:L2` view reads only from allowed A/B `L2+` levels
- [ ] every declared `C:L2` view specifies joins, row grain, projected columns,
      ABAC scope, exclusion behavior, and refresh metadata
- [ ] no shell view reads raw `L0/L1`
- [ ] battle/world shells remain downstream consumers of `C:L2`, not separate
      truth layers

## Phase 5 - End-To-End Proof Gate

No generator onboarding is complete without fixtures and proof queries.

The proof must demonstrate the full path:

`generator-native source -> raw landing -> normalized A/B L2+ rows -> registered C:L2 query results`

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

## Readiness Gate

A generator is ready for product consumption only when every item below is true.

- [ ] all source objects map to shared A/B families
- [ ] no generator-specific family ids leak above raw-source handling
- [ ] raw landing preserves immutable keys, lineage, level, and plane metadata
- [ ] normalization produces shared A/B `L2+` rows rather than app-facing canon
- [ ] at least one shell-specific `C:L2` view set is registered
- [ ] ABAC and exclusion behavior are enforced in the registered `C:L2` views
- [ ] world projection, if present, is downstream of `C:L2`
- [ ] end-to-end fixtures prove the generator is consumable without raw reads

If any checklist item above is incomplete, the generator is not onboarded.

## DeerFlow Reference Binding

DeerFlow is the first reference onboarding path for this contract.

Use these plan docs as the canonical execution sequence:

- `docs/superpowers/plans/backend-generator-to-canonical/000-overview.md`
- `docs/superpowers/plans/backend-generator-to-canonical/000a-task-0-storage-contract-and-promotion-policy.md`
- `docs/superpowers/plans/backend-generator-to-canonical/002-task-2-deerflow-raw-envelopes.md`
- `docs/superpowers/plans/backend-generator-to-canonical/003-task-3-deerflow-normalization.md`
- `docs/superpowers/plans/backend-generator-to-canonical/004-task-4-game-facing-derivations.md`
- `docs/superpowers/plans/backend-generator-to-canonical/005-task-5-world-projection.md`

Any future generator should follow the same runbook:

- bind into shared A/B families first
- normalize into shared A/B `L2+` rows second
- register shell-defined `C:L2` views third
- prove end-to-end consumption fourth

## Failure Rule

Do not mark a backend generator as supported for Game UI, apps, tools, or shell
consumers unless the full onboarding runbook has been satisfied.
