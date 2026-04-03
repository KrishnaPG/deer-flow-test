# Shared Pipeline And DeerFlow Onboarding Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend the existing reusable pipeline so DeerFlow can flow from generator capture through storage-aware raw envelopes, state-server-mediated reads, lineage-safe canonical records, and L2 game-facing projections that later generators can follow.

**Architecture:** Reuse the current foundation, raw-source, normalizer, derivation, and world-projection crates instead of inventing app-local glue. Start by hardening foundation metadata for source level/plane and correlation needs, then add a DeerFlow-specific raw/mediated adapter, normalize that into L0/L1/L2 canonical records, and finish with reusable game-facing derivations plus world objects backed by lineage and backlinks.

**Tech Stack:** Rust workspace crates, `serde`, `serde_json`, `chrono`, `thiserror`, `insta`, existing DeerFlow fixture/bridge shapes as reference only

---

## Scope Boundary

- This plan covers the shared reusable spine plus DeerFlow onboarding only.
- It modifies existing crates: `foundation/contracts`, `foundation/domain`, `pipeline/raw_sources`, `pipeline/normalizers`, `pipeline/derivations`, and `runtime/world_projection`.
- It does **not** modify `apps/deer_gui` or any proof app.
- It does **not** onboard Hermes, Rowboat, or PocketFlow yet; the resulting pipeline must make those follow-on adapters straightforward.
- It does **not** implement a production state server. It introduces state-server-aligned mediated read DTOs and fixture-backed DeerFlow adapters that preserve the boundary.
- It does **not** allow direct backend payloads, direct artifact paths, or app-local mutable truth to bypass storage/lineage semantics.
- It does **not** implement production ABAC policy; it introduces intent envelope shapes aligned to the spec 11 intent boundary so that later adapters can route through policy validation.
- It **does** capture the contract surfaces this slice needs for exclusions, authorized artifact previews, ordered intents, replay checkpoints, and storage backpressure so later infrastructure work has a canonical target instead of app-local glue.

## Spec Alignment Notes

- **Spec 11 (State Server Alignment):** Pipeline follows `generator → storage truth → mediated source → normalizer → canonical → derivation → world projection`. Lineage is append-only (`source_events`, `derived_from`, `supersedes`). Intent shapes are introduced as raw envelope families so that human/UI-originated actions can be modeled as commands that pass through policy/ABAC before becoming append-only records.
- **Spec 12 (Levels & Planes):** `RecordHeader` carries `level`, `plane`, `source_level`, `source_plane`, `is_persisted_truth`, `is_index_projection`, `is_client_transient`. Planes `AsIs`, `Chunks`, `Embeddings` are defined in envelope refs. This slice implements `AsIs` end to end and explicitly preserves the hash anchors and metadata that future `Chunks` and `Embeddings` work must build on.
- **Spec 24 (Level/Plane/Lineage Matrix):** This plan exercises the matrix cells needed for DeerFlow onboarding now: `L0_SourceRecord(L0/AsIs)`, `L1_SanitizedRecord(L1/AsIs)`, `TaskRecord(L2)`, `ArtifactRecord(L2/AsIs)`, `RuntimeStatusRecord(L0..L2)`, `IntentRecord(L2)`, `ExclusionRecord(L2)`, `ConflictRecord(L2)`, `ReplayCheckpointRecord(L2)`, `AsIsRepresentationRecord(L0/AsIs)`, and `TransformRecord(L2)`. The remaining matrix families stay out of scope for this slice rather than being redefined incompletely.

## File Structure

### Foundation metadata hardening

- Modify: `crates/foundation/contracts/src/meta.rs` - extend correlation metadata with `agent_id` and source object anchors needed by generator onboarding.
- Modify: `crates/foundation/contracts/src/records.rs` - add source level/plane and storage-disposition convenience flags to `RecordHeader`.
- Modify: `crates/foundation/domain/src/common.rs` - add header builders that preserve correlations and explicit source level/plane metadata.
- Modify: `crates/foundation/domain/src/common.rs` - extend the `define_record!` macro with `new_with_meta(...)` while keeping existing `new(...)` intact.
- Create: `crates/foundation/contracts/tests/source_metadata_contracts.rs` - verify required source/plane fields and correlation metadata.
- Create: `crates/foundation/domain/tests/source_aware_record_builders.rs` - verify records can preserve correlation and source metadata without breaking existing constructors.

### DeerFlow raw source and mediated-read layer

- Create: `crates/pipeline/raw_sources/src/envelopes.rs` - raw envelope families aligned to the spec catalog.
- Create: `crates/pipeline/raw_sources/src/deerflow.rs` - DeerFlow-specific fixture/parser adapter that emits raw envelope batches.
- Create: `crates/pipeline/raw_sources/src/mediated_reads.rs` - state-server-aligned live stream, snapshot, replay window, and artifact preview DTOs.
- Modify: `crates/pipeline/raw_sources/src/lib.rs` - export the new envelope, DeerFlow, and mediated-read APIs.
- Modify: `crates/pipeline/raw_sources/Cargo.toml` - add any crate dependencies needed by the new modules.
- Create: `crates/pipeline/raw_sources/tests/deerflow_adapter.rs` - end-to-end fixture test for DeerFlow raw capture and mediated read shaping.
- Create: `crates/pipeline/raw_sources/tests/fixtures/deerflow_thread_snapshot.json` - DeerFlow thread/run snapshot fixture.
- Create: `crates/pipeline/raw_sources/tests/fixtures/deerflow_live_activity.json` - DeerFlow multi-agent live activity fixture.
- Create: `crates/pipeline/raw_sources/tests/fixtures/deerflow_replay_window.json` - DeerFlow replay/history fixture.
- Create: `crates/pipeline/raw_sources/tests/fixtures/deerflow_artifact_preview.json` - ABAC-authorized artifact preview fixture with signed access metadata.
- Create: `crates/pipeline/raw_sources/tests/fixtures/deerflow_exclusion_intent.json` - compliance exclusion intent fixture.
- Create: `crates/pipeline/raw_sources/tests/fixtures/deerflow_conflicting_intents.json` - concurrent intent fixture with deterministic sequence ids.
- Create: `crates/pipeline/raw_sources/tests/fixtures/deerflow_storage_backpressure.json` - storage backpressure fixture for rate-limit stress coverage.
- Create: `crates/pipeline/raw_sources/tests/fixtures/deerflow_replay_checkpoint.json` - replay checkpoint fixture for cache rehydration alignment.

### Canonical normalization

- Modify: `crates/pipeline/normalizers/src/lib.rs` - export the new DeerFlow normalization entrypoint.
- Modify: `crates/pipeline/normalizers/src/carrier.rs` - normalize DeerFlow raw envelopes into L0/L1/L2 carrier records with correlations and lineage.
- Modify: `crates/pipeline/normalizers/src/representation.rs` - emit representation records for DeerFlow `as_is` and selective `chunks` payloads.
- Modify: `crates/pipeline/normalizers/src/governance.rs` - emit transform records for DeerFlow promotion steps.
- Create: `crates/pipeline/normalizers/tests/deerflow_levels_lineage.rs` - verify L0/L1/L2 occupancy, lineage backlinks, correlation preservation, exclusions, ordered intents, replay checkpoints, and backpressure records.

### L2 game-facing derivations

- Create: `crates/pipeline/derivations/src/game_face.rs` - derive agent units, task-force links, queue rows, history rows, knowledge nodes, intervention surfaces, session state, artifact families, and telemetry summaries from canonical records.
- Modify: `crates/pipeline/derivations/src/lib.rs` - export the new game-facing VM entrypoints.
- Create: `crates/pipeline/derivations/tests/deerflow_game_face.rs` - verify DeerFlow canonical records derive into stable L2 game-facing views with backlinks.

### World projection

- Modify: `crates/runtime/world_projection/src/world_object.rs` - add world object constructors for unit actors, task-force links, queue beacons, and history anchors with level/plane/source metadata.
- Modify: `crates/runtime/world_projection/src/projection_rules.rs` - project DeerFlow-backed canonical records into world objects without inventing app-local truth.
- Create: `crates/runtime/world_projection/tests/deerflow_pipeline_projection.rs` - end-to-end pipeline proof from DeerFlow raw fixture to world projection.

## Stress-Test Alignment (docs/architecture/stess-tests.md)

| Scenario | Concrete plan capture |
| --- | --- |
| 1: 10,000 parallel agents | Raw envelope batches remain cardinality-safe, intent/event ordering is explicit via `sequence_id`, and game-facing queue pressure surfaces expose load before collapse. |
| 2: HOTL media pipelines | Mediated reads include runtime/checkpoint surfaces, authorized artifact preview DTOs, and intervention records so supervisory review appears as observed state rather than app-local mutation. |
| 3: Right to be forgotten | Exclusion intents normalize into canonical exclusion/governance records carrying `target_hash`, `reason`, and lineage back to the affected record family. |
| 4: Massive video proxying | Artifact preview DTOs carry authorized access metadata (`authorization_kind`, `access_url`, `expires_at`) so clients never depend on raw artifact paths. |
| 5: HITL low-latency chat | `LiveActivityStream` and `StreamDelta` records remain the thin read boundary for fast UI feedback while preserving storage-native source identity. |
| 6: S3 rate limiting | Backpressure fixtures normalize into canonical operational records and game-facing queue pressure so storage throttling is visible without blocking generators. |
| 7: State Server replica crash | Replay window and replay checkpoint DTOs preserve the last acknowledged sequence boundary needed for cache rehydration semantics. |
| 8: Concurrent conflicting intents | Conflicting intents are represented as separate append-only source objects with deterministic ordering metadata and canonical conflict/resolution records. |

## Gamification Alignment (docs/superpowers/research/data-gamification/)

| Gamification concept | Concrete plan capture |
| --- | --- |
| Agent health / fatigue / performance | `UnitActorVm` includes `AgentHealthVm`, `AgentPerformanceVm`, and `AgentTrackRecordVm` so the same canonical record supports RTS telemetry and RPG companion sheets. |
| Knowledge graph | `KnowledgeGraphVm` and `KnowledgeNodeVm` expose artifact/reference lineage as navigable graph nodes rather than flat history. |
| Queue pressure | `QueuePressureVm` and queue beacon projection make congestion visible as gameplay state instead of buried operational metrics. |
| Event stories | `HistoryRowVm.event_story` and telemetry event chains group related task/artifact events into readable narrative beats. |
| Artifact families | `ArtifactFamilyVm` classifies text/code/dataset/image/audio/video/model artifacts so the same object can render as relic, blueprint, dossier, replay, or codex entry. |
| Session state / branching | `SessionStateVm` makes run/thread state and branch context first-class so RTS theaters and RPG chapters stay backed by the same truth. |
| Operator override / HITL | `InterventionVm` and HOTL checkpoint rows expose human approval/override as deliberate gameplay actions backed by intents and observed outcomes. |
| HOTL review surfaces | Checkpoint and authorized preview records keep live review grounded in lineage, timestamps, and source attribution. |
