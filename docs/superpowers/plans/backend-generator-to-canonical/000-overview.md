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

## Spec Alignment Notes

- **Spec 11 (State Server Alignment):** Pipeline follows `generator → storage truth → mediated source → normalizer → canonical → derivation → world projection`. Lineage is append-only (`source_events`, `derived_from`, `supersedes`). Intent shapes are introduced as raw envelope families so that human/UI-originated actions can be modeled as commands that pass through policy/ABAC before becoming append-only records.
- **Spec 12 (Levels & Planes):** `RecordHeader` carries `level`, `plane`, `source_level`, `source_plane`, `is_persisted_truth`, `is_index_projection`, `is_client_transient`. Planes `AsIs`, `Chunks`, `Embeddings` are defined in envelope refs. This slice exercises `AsIs` end-to-end; `Chunks` and `Embeddings` normalization are deferred (spec 12: representation policy is selective, not uniform).
- **Spec 24 (Level/Plane/Lineage Matrix):** This plan exercises the following matrix cells: `L0_SourceRecord(L0/AsIs)`, `L1_SanitizedRecord(L1/AsIs)`, `TaskRecord(L2)`, `ArtifactRecord(L2/AsIs)`, `RuntimeStatusRecord(L0..L2)`, `IntentRecord(L2)`, `AsIsRepresentationRecord(L0/AsIs)`, `TransformRecord(L2)`. All other matrix cells (L3–L6 families, `ChunkRecord`, `EmbeddingRecord`, governance families like `DedupRecord`, `BranchRecord`, `VersionRecord`, etc.) are intentionally deferred to follow-on onboarding slices.

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

### Canonical normalization

- Modify: `crates/pipeline/normalizers/src/lib.rs` - export the new DeerFlow normalization entrypoint.
- Modify: `crates/pipeline/normalizers/src/carrier.rs` - normalize DeerFlow raw envelopes into L0/L1/L2 carrier records with correlations and lineage.
- Modify: `crates/pipeline/normalizers/src/representation.rs` - emit representation records for DeerFlow `as_is` and selective `chunks` payloads.
- Modify: `crates/pipeline/normalizers/src/governance.rs` - emit transform records for DeerFlow promotion steps.
- Create: `crates/pipeline/normalizers/tests/deerflow_levels_lineage.rs` - verify L0/L1/L2 occupancy, lineage backlinks, and correlation preservation.

### L2 game-facing derivations

- Create: `crates/pipeline/derivations/src/game_face.rs` - derive agent units, task-force links, queue rows, history rows, and telemetry summaries from canonical records.
- Modify: `crates/pipeline/derivations/src/lib.rs` - export the new game-facing VM entrypoints.
- Create: `crates/pipeline/derivations/tests/deerflow_game_face.rs` - verify DeerFlow canonical records derive into stable L2 game-facing views with backlinks.

### World projection

- Modify: `crates/runtime/world_projection/src/world_object.rs` - add world object constructors for unit actors, task-force links, queue beacons, and history anchors with level/plane/source metadata.
- Modify: `crates/runtime/world_projection/src/projection_rules.rs` - project DeerFlow-backed canonical records into world objects without inventing app-local truth.
- Create: `crates/runtime/world_projection/tests/deerflow_pipeline_projection.rs` - end-to-end pipeline proof from DeerFlow raw fixture to world projection.

## Stress-Test Alignment (docs/architecture/stess-tests.md)

| Scenario | Coverage in this plan | Deferred |
| --- | --- | --- |
| 1: 10,000 parallel agents | Raw envelope batches support arbitrary cardinality; `agent_id` correlation tracks per-agent records; queue pressure telemetry detects load | Bloom-filter dedup, Parquet micro-batching, NATS JetStream backpressure |
| 2: HOTL media pipelines | Mediated reads include live activity streams; `RuntimeStatus` envelopes carry pipeline state; operator intent shapes exist | Pre-signed URL gateway, large blob proxy avoidance |
| 3: Right to be forgotten | `IntentRecord` with exclusion payload shapes the compliance path; `ExclusionRecord` is in the spec 24 matrix | Full exclusion ledger, anti-join DB view construction |
| 4: Massive video proxying | `ArtifactRecord` carries `as_is_hash` identity anchor | Pre-signed URL generation, lakeFS/S3 version commit routing |
| 5: HITL low-latency chat | `LiveActivityStream` DTOs and `StreamDelta` envelopes support sub-second token delivery | WebRTC offload, LiveKit egress integration |
| 6: S3 rate limiting | Intent and envelope queues are append-only and NATS-aligned | NATS JetStream ingress queue, exponential backoff workflows |
| 7: State Server replica crash | Lineage `source_events` and `derived_from` enable cache rehydration from NATS tail | Redis leadership, NATS JetStream replay from sequence |
| 8: Concurrent conflicting intents | Both intents append as `IntentRecord` with deterministic `event_id`; lineage preserves both | NATS JetStream sequence ID, `argMax` materialized views |

## Gamification Alignment (docs/superpowers/research/data-gamification/)

| Gamification concept | Coverage in this plan | Deferred |
| --- | --- | --- |
| Agent health / fatigue / performance | `AgentHealthVm`, `AgentPerformanceVm`, `AgentTrackRecordVm` added to `UnitActorVm` | Dynamic fatigue calculation, XP/leveling, reputation tiers beyond veteran/rookie |
| Knowledge graph | `KnowledgeGraphVm` with `KnowledgeNodeVm` nodes and lineage-based links | Full graph traversal, topic clusters, reference bundles |
| Queue pressure | `QueuePressureVm` with `queued_count`, `running_count`, `pressure_level` | Supply congestion visualization, encumbrance mechanics |
| Event stories | `TelemetryVm.event_stories` chains related artifact/task events into narrative strings | Full event-story collapse, severity-based notification grouping |
| Artifact families | `ArtifactRecord` carries `label` and `as_is_hash`; type classification via `media_type` in `AsIsRepresentationBody` | Explicit `ArtifactFamily` enum (text, code, dataset, image, audio, video, model) |
| Session state / branching | `CorrelationMeta` carries `thread_id`, `run_id`, `session_id` equivalents | Explicit session state machine, branch/fork VM types |
| Operator override / HITL | `IntentRecord` shapes for both action and exclusion intents | QTE-style intervention surfaces, timed dialogue choice VMs |
| HOTL review surfaces | `LiveActivityStream` with `RuntimeStatus` for pipeline checkpointing | Refinery queue UI, checkpoint approval/rejection flows |
