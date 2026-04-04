# Shared Storage Service Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first shared storage service slice: immutable append contracts, Redpanda-backed durable intake, canonical relative path construction, `FileAccepted` / `FileSaved` semantics, post-commit downstream handoff, and a DeerFlow backend-driver stress test.

**Architecture:** Extend `deer-foundation-contracts` with a shared storage contract module that reuses existing `CanonicalLevel` and `CanonicalPlane`, add a new `deer-storage-core` crate for validation/path/topic/admission/commit logic behind ports, and add a DeerFlow driver facade in `deer-pipeline-raw-sources` that emits only dimensions, metadata, and append requests. The storage service stops at immutable save plus `FileSaved`; Iceberg, LakeFS, ClickHouse, Milvus, SQL/search exposure, and downstream publication remain outside this implementation boundary.

**Tech Stack:** Rust 2021 workspace crates, `serde`, `thiserror`, `bytes`, existing `deer-foundation-contracts` types, Redpanda topic-class abstractions behind traits, object-storage commit traits, crate-local tests with fake ports

---

## Source Of Truth

- Spec: `docs/superpowers/specs/2026-04-04-shared-storage-service-design.md`
- Shadow WIP: `.opencode/wip/shared-storage-service-design.md`

## Key Corrections From The Superseded Large Plan

- Keep existing public re-exports in `crates/foundation/contracts/src/lib.rs` and `crates/pipeline/raw_sources/src/lib.rs`; only add to them.
- Reuse `CanonicalLevel` and `CanonicalPlane` from `crates/foundation/contracts/src/records.rs` instead of duplicating level/plane enums.
- Model `StorageHierarchy`, `PayloadKind`, `PayloadFormat`, and partition tags as storage-facing types without adding any driver path-hint surface.
- Define both `append_data` and `append_control` explicitly.
- Define both `FileAccepted` and full `FileSaved` explicitly.
- Use a composable QoS policy object plus convenience templates, while keeping blocking waits and caller-tunable backpressure out of the public API.
- Make multi-file visibility depend on the final manifest/marker, not on member-file presence.
- Add the missing dependency change in `crates/pipeline/raw_sources/Cargo.toml` for the storage contract module.
- Keep replay-safe downstream tests at the event-contract boundary; do not implement downstream consumers in the storage crate.

## File Map

### Shared contracts

- Modify: `Cargo.toml` - add `crates/storage/core` workspace member.
- Modify: `crates/foundation/contracts/src/lib.rs` - preserve existing `pub use` lines and export the new `storage` module.
- Modify: `crates/foundation/contracts/src/ids.rs` - add newtypes for `LogicalWriteId`, `IdempotencyKey`, `WriterId`, and `CommitManifestId`.
- Create: `crates/foundation/contracts/src/storage.rs` - append request/control request contracts, storage-specific ids, layout types, payload descriptors, QoS policy + templates, `FileAccepted`, typed `FileSavedTarget`, `FileSaved`, `DerivationTrigger`, and rejection reasons.
- Test: `crates/foundation/contracts/tests/storage_contracts.rs`.

### Storage core

- Create: `crates/storage/core/Cargo.toml`
- Create: `crates/storage/core/src/lib.rs`
- Create: `crates/storage/core/src/service.rs`
- Create: `crates/storage/core/src/validation.rs`
- Create: `crates/storage/core/src/layout.rs`
- Create: `crates/storage/core/src/path_builder.rs`
- Create: `crates/storage/core/src/topics.rs`
- Create: `crates/storage/core/src/ports.rs`
- Create: `crates/storage/core/src/commit.rs`
- Create: `crates/storage/core/src/manifest.rs`
- Create: `crates/storage/core/src/admission.rs`
- Create: `crates/storage/core/src/downstream_handoff.rs`
- Create: `crates/storage/core/src/diagnostics.rs`
- Create: `crates/storage/core/src/boundary.rs`
- Tests: `crates/storage/core/tests/append_requests.rs`, `crates/storage/core/tests/path_builder.rs`, `crates/storage/core/tests/commit_semantics.rs`, `crates/storage/core/tests/manifest_semantics.rs`, `crates/storage/core/tests/admission.rs`, `crates/storage/core/tests/file_saved_contract.rs`, `crates/storage/core/tests/post_commit_trigger.rs`, `crates/storage/core/tests/stress_scenarios.rs`.

### DeerFlow driver stress test

- Modify: `crates/pipeline/raw_sources/Cargo.toml` - add dependency on `deer-foundation-contracts`.
- Modify: `crates/pipeline/raw_sources/src/lib.rs` - preserve existing re-exports and add the new `deerflow_driver` module.
- Create: `crates/pipeline/raw_sources/src/deerflow_driver/mod.rs`
- Create: `crates/pipeline/raw_sources/src/deerflow_driver/facade.rs`
- Create: `crates/pipeline/raw_sources/src/deerflow_driver/mapping.rs`
- Create: `crates/pipeline/raw_sources/src/deerflow_driver/policy.rs`
- Create: `crates/pipeline/raw_sources/src/deerflow_driver/transcript_emitter.rs`
- Create: `crates/pipeline/raw_sources/src/deerflow_driver/artifact_emitter.rs`
- Create: `crates/pipeline/raw_sources/src/deerflow_driver/metric_emitter.rs`
- Create: `crates/pipeline/raw_sources/src/deerflow_driver/control_emitter.rs`
- Tests: `crates/pipeline/raw_sources/tests/deerflow_driver_contract.rs`, `crates/pipeline/raw_sources/tests/deerflow_driver_stress.rs`.

## Task Index

1. `001-task-1-shared-storage-contracts.md` - shared storage contracts, `FileAccepted`, `FileSaved`, QoS policy, append/control separation
2. `002-task-2-storage-core-crate.md` - storage-core validation, paths, topics, admission, commit, manifest semantics
3. `003-task-3-handoff-and-diagnostics.md` - `FileSaved`/trigger emission, replay-safe handoff, internal diagnostics, LiveKit boundary
4. `004-task-4-deerflow-driver.md` - DeerFlow facade + specialized emitters over the generic storage contract
5. `005-task-5-integrated-proof.md` - targeted package sweep and full verification

## Non-Goals For This Plan

- implementing Iceberg/LakeFS/ClickHouse/Milvus publishers
- implementing SQL views or search indexing
- implementing CAS/hash-collapse dedupe
- exposing internal queue/worker states as public API
- adding driver path hints or backend-specific canonical path logic
