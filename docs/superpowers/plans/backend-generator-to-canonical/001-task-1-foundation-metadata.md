## Task 1: Harden Foundation Metadata For Immutable Joins, Lineage, ABAC, And C:L2 View Refresh

**Files:**
- Modify: `crates/foundation/contracts/src/meta.rs`
- Modify: `crates/foundation/contracts/src/records.rs`
- Modify: `crates/foundation/domain/src/common.rs`
- Test: `crates/foundation/contracts/tests/source_metadata_contracts.rs`
- Test: `crates/foundation/domain/tests/source_aware_record_builders.rs`

**Milestone unlock:** every stored A/B record carries the immutable join keys, lineage anchors, correlation metadata, ABAC/exclusion metadata, and view-refresh metadata required to author and refresh `C:L2` SQL views without app-local reconstruction.

**Forbidden shortcuts:** do not use mutable display ids as join keys; do not infer lineage from payload text; do not leave ABAC/exclusion decisions in app code; do not let `C:L2` refresh depend on client caches; do not add metadata that is DeerFlow-specific unless it is nested behind a generator-binding field.

### Metadata that must exist after this task

| Metadata block | Required fields | Why `C:L2` needs it |
| --- | --- | --- |
| `JoinKeysMeta` | `generator_key`, `storage_object_key`, `session_key`, `thread_key`, `run_key`, `agent_key`, `task_key`, `artifact_key`, `intent_key`, `exclusion_key` | stable SQL joins across A and B |
| `LineageMeta` | `source_record_id`, `source_hash`, `derived_from`, `supersedes`, `transform_key` | traceable provenance and replay-safe rebuilds |
| `CorrelationMeta` | `trace_id`, `mission_id`, `thread_id`, `run_id`, `task_id`, `agent_id`, `actor_id` | cross-family stitching without payload heuristics |
| `AccessControlMeta` | `abac_scope`, `policy_tags`, `exclusion_targets`, `redaction_mode`, `authorized_projection` | enforce view-level visibility and exclusions |
| `ViewRefreshMeta` | `eligible_c_views`, `refresh_mode`, `refresh_watermark`, `refresh_group`, `invalidated_by`, `last_source_level` | deterministic `C:L2` refresh and invalidation |

- [ ] **Step 1: Write the failing foundation metadata tests**

Create contract tests that prove all of the following:

- record headers expose immutable join keys separately from user-facing labels
- lineage metadata can identify both the immediate source row and any canonical transforms applied later
- correlation metadata preserves agent, actor, run, thread, and task anchors needed for joins across A/B families
- access-control metadata can express ABAC scope plus exclusion/redaction targets used by `C:L2` views
- refresh metadata can declare which `C:L2` views a row can invalidate or refresh and which watermark field they use

Create domain-builder tests that prove all of the following:

- default builders keep existing behavior for simple records
- source-aware builders can preserve join keys and lineage when moving from A-level capture into B-level records
- builders can attach refresh metadata without requiring app-local code to patch headers later

- [ ] **Step 2: Run the targeted foundation tests and confirm they fail**

Run the contracts test target and the domain-builder test target.

Expected: failure because the header does not yet expose dedicated join-key, access-control, or view-refresh metadata and the builders cannot preserve them.

- [ ] **Step 3: Extend foundation contracts with explicit metadata blocks**

Update `meta.rs` and `records.rs` so that every stored A/B record header can carry:

- `JoinKeysMeta` for immutable relational keys
- expanded `LineageMeta` that distinguishes source anchors from later transforms
- expanded `CorrelationMeta` for cross-family stitching
- `AccessControlMeta` for ABAC and exclusion/redaction state
- `ViewRefreshMeta` for `C:L2` refresh dependencies and watermarks

Make the new fields first-class header members rather than opaque JSON blobs so SQL view definitions can rely on stable field names.

- [ ] **Step 4: Extend domain builders to preserve the new metadata end to end**

Update `common.rs` so record builders support both:

- existing simple constructors for callers that do not need explicit metadata overrides
- source-aware constructors that accept join keys, lineage, correlation, access-control, and refresh metadata in one place

Ensure the builders can preserve an A-level source anchor while emitting a B-level record that later participates in `C:L2` SQL joins.

- [ ] **Step 5: Re-run the targeted foundation tests**

Re-run the contracts and domain-builder tests.

Expected: pass with headers and builders exposing the metadata needed by `C:L2` SQL view definitions and refresh logic.

- [ ] **Step 6: Verify metadata coverage against the C:L2 contract**

Review the Task 0 view requirements and confirm each required contract dimension now has a foundation-level source of truth:

- join keys for SQL joins
- lineage for provenance and replay
- correlation for cross-family grouping
- ABAC/exclusion for visibility
- refresh metadata for view invalidation and materialization
