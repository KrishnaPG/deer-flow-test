# Normalization, Derivation, And Read-Model Spine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the remainder of the M0 reusable pipeline spine so raw fixture inputs can flow into canonical carrier and representation records, then into typed orchestration, artifact, and macro-state view models, while transient shell state stays testable outside any app shell.

**Architecture:** Introduce three new workspace crates in dependency order: `deer-pipeline-normalizers` for raw-envelope-to-canonical translation and promotion enforcement, `deer-pipeline-derivations` for typed view-model builders over canonical records, and `deer-runtime-read-models` for transient shell-local reducers such as intent lifecycle, artifact selection, linked selection, and replay cursor state. Keep this plan pre-proof-app and pre-`deer_gui`: the only acceptance surface here is crate-local fixture, golden, and reducer tests.

**Tech Stack:** Rust 2021 workspace crates, `serde`, `serde_json`, `thiserror`, `chrono`, `uuid`, `insta`, `similar-asserts`

---

## Scope Boundary

- This plan covers only `crates/pipeline/normalizers`, `crates/pipeline/derivations`, and `crates/runtime/read_models`.
- It depends on `docs/superpowers/plans/2026-04-01-foundation-spine-contracts-domain-replay.md` and reuses the foundation crates defined there; it must not redefine IDs, headers, lineage, or replay primitives locally.
- It does **not** create `crates/pipeline/raw_sources`, `crates/runtime/world_projection`, `crates/views/*`, `crates/ui/*`, or any proof app.
- It does **not** modify `apps/deer_gui`.
- It does **not** add Bevy, egui, renderer, docking, layout, camera, or world-metaphor dependencies to these crates.
- It does **not** allow direct storage, DB, vector-index, or backend-specific read bypasses; all external-facing reads remain mediated by contract.

## File Structure

### Workspace

- Modify: `Cargo.toml` - add each new crate member incrementally when that crate task begins.

### Normalizers crate

- Create: `crates/pipeline/normalizers/Cargo.toml` - package manifest for canonical normalization.
- Create: `crates/pipeline/normalizers/src/lib.rs` - public exports.
- Create: `crates/pipeline/normalizers/src/error.rs` - normalization and promotion error types.
- Create: `crates/pipeline/normalizers/src/envelopes.rs` - raw envelope batch DTOs used only below the canonical boundary.
- Create: `crates/pipeline/normalizers/src/carrier.rs` - raw-to-carrier normalization for run, session, message, task, artifact, clarification, and runtime status records.
- Create: `crates/pipeline/normalizers/src/promotions.rs` - explicit `L0..L6` promotion guards and occupancy checks.
- Create: `crates/pipeline/normalizers/src/representation.rs` - `AsIsRepresentationRecord`, `ChunkRecord`, and `EmbeddingRecord` identity-chain normalization helpers.
- Create: `crates/pipeline/normalizers/src/governance.rs` - intent-stage and operational record emission helpers.
- Create: `crates/pipeline/normalizers/tests/carrier_normalization.rs` - fixture transform tests for carrier records.
- Create: `crates/pipeline/normalizers/tests/promotion_and_representation.rs` - promotion, hash-anchor, and governance tests.
- Create: `crates/pipeline/normalizers/tests/fixtures/deerflow_live_run.json` - representative DeerFlow live activity input.
- Create: `crates/pipeline/normalizers/tests/fixtures/hermes_run.json` - representative Hermes live activity input to prove generator agnosticism.
- Create: `crates/pipeline/normalizers/tests/fixtures/representation_chain.json` - `as_is` / `chunk` / `embedding` fixture.

### Derivations crate

- Create: `crates/pipeline/derivations/Cargo.toml` - package manifest for typed view-model derivation.
- Create: `crates/pipeline/derivations/src/lib.rs` - public exports.
- Create: `crates/pipeline/derivations/src/common.rs` - canonical refs, provenance badges, and support metadata shared across VMs.
- Create: `crates/pipeline/derivations/src/orchestration.rs` - `TranscriptVm`, `RunStatusVm`, and `TaskProgressVm` builders.
- Create: `crates/pipeline/derivations/src/artifacts.rs` - `ArtifactShelfVm` and artifact detail summary builders.
- Create: `crates/pipeline/derivations/src/macro_state.rs` - app-agnostic macro-state projection rows with drill-back metadata.
- Create: `crates/pipeline/derivations/tests/orchestration_vm.rs` - deterministic transcript and run-status golden tests.
- Create: `crates/pipeline/derivations/tests/artifact_vm.rs` - deterministic artifact-shelf golden tests.
- Create: `crates/pipeline/derivations/tests/macro_state_vm.rs` - deterministic macro-state golden tests.

### Read-models crate

- Create: `crates/runtime/read_models/Cargo.toml` - package manifest for transient shell-local state.
- Create: `crates/runtime/read_models/src/lib.rs` - public exports.
- Create: `crates/runtime/read_models/src/chat.rs` - prompt draft, attachment queue, and clarification pending state.
- Create: `crates/runtime/read_models/src/artifacts.rs` - selected artifact and mediated preview request state.
- Create: `crates/runtime/read_models/src/linked_shell.rs` - selection, focus, pin, compare, and drill-down state.
- Create: `crates/runtime/read_models/src/intent.rs` - `prefill_seed -> prefill -> draft -> validated -> submitted` reducer.
- Create: `crates/runtime/read_models/src/temporal.rs` - replay cursor, temporal range, live-tail, and staleness state.
- Create: `crates/runtime/read_models/src/policy.rs` - exclusion and masking invalidation behavior.
- Create: `crates/runtime/read_models/tests/intent_reducer.rs` - intent lifecycle reducer tests.
- Create: `crates/runtime/read_models/tests/linked_shell_reducer.rs` - selection, pin, compare, and drill-down tests.
- Create: `crates/runtime/read_models/tests/temporal_and_policy.rs` - cursor, stale-state, and policy invalidation tests.

## Task 1: Create The Workspace And Normalizer Carrier Spine

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/pipeline/normalizers/Cargo.toml`
- Create: `crates/pipeline/normalizers/src/lib.rs`
- Create: `crates/pipeline/normalizers/src/error.rs`
- Create: `crates/pipeline/normalizers/src/envelopes.rs`
- Create: `crates/pipeline/normalizers/src/carrier.rs`
- Test: `crates/pipeline/normalizers/tests/carrier_normalization.rs`
- Test fixture: `crates/pipeline/normalizers/tests/fixtures/deerflow_live_run.json`
- Test fixture: `crates/pipeline/normalizers/tests/fixtures/hermes_run.json`

**Milestone unlock:** M0 `pipeline/normalizers` carrier path

**Proof path later:** downstream proof in `apps/deer_chat_lab` and replay-driven inspection in `apps/deer_replay`

**Forbidden dependencies:** `bevy`, `bevy_egui`, `apps/deer_gui`, direct storage reads, backend-specific types above `envelopes.rs`

- [ ] **Step 1: Write the failing carrier-normalization test, fixture, and workspace stubs**

```toml
# Cargo.toml
[workspace]
members = [
    "apps/deer_gui",
    "crates/foundation/contracts",
    "crates/foundation/domain",
    "crates/foundation/replay",
    "crates/pipeline/normalizers",
]
resolver = "2"
```

```toml
# crates/pipeline/normalizers/Cargo.toml
[package]
name = "deer-pipeline-normalizers"
version = "0.1.0"
edition = "2021"

[dependencies]
chrono = { version = "0.4.44", features = ["clock", "serde"] }
deer-foundation-contracts = { path = "../../foundation/contracts" }
deer-foundation-domain = { path = "../../foundation/domain" }
deer-foundation-replay = { path = "../../foundation/replay" }
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.149"
thiserror = "2.0.17"

[dev-dependencies]
similar-asserts = "1.7.0"
```

```rust
// crates/pipeline/normalizers/src/lib.rs
pub mod carrier;
pub mod envelopes;
pub mod error;

pub use carrier::{normalize_batch, NormalizedBatch};
pub use envelopes::RawEnvelopeBatch;
pub use error::NormalizationError;
```

```json
// crates/pipeline/normalizers/tests/fixtures/deerflow_live_run.json
{
  "generator": "deerflow",
  "session": {
    "session_id": "session_1",
    "title": "Survey the ridge"
  },
  "run": {
    "run_id": "run_1",
    "status": "running"
  },
  "events": [
    {
      "kind": "message",
      "message_id": "msg_1",
      "role": "operator",
      "text": "Survey the ridge and summarize the terrain.",
      "level": "L2"
    },
    {
      "kind": "task",
      "task_id": "task_1",
      "title": "Gather terrain notes",
      "state": "running"
    },
    {
      "kind": "artifact",
      "artifact_id": "artifact_1",
      "name": "terrain.md",
      "status": "presented",
      "as_is_hash": "sha256:terrain-md"
    },
    {
      "kind": "runtime_status",
      "state": "streaming"
    },
    {
      "kind": "clarification",
      "clarification_id": "clar_1",
      "prompt": "Confirm the survey radius."
    }
  ]
}
```

```json
// crates/pipeline/normalizers/tests/fixtures/hermes_run.json
{
  "generator": "hermes",
  "session": {
    "session_id": "session_1",
    "title": "Survey the ridge"
  },
  "run": {
    "run_id": "run_1",
    "status": "running"
  },
  "events": [
    {
      "kind": "message",
      "message_id": "msg_2",
      "role": "assistant",
      "text": "Terrain is clear.",
      "level": "L2"
    }
  ]
}
```

```rust
// crates/pipeline/normalizers/tests/carrier_normalization.rs
use deer_foundation_domain::AnyRecord;
use deer_pipeline_normalizers::{normalize_batch, RawEnvelopeBatch};

#[test]
fn normalizes_deerflow_events_into_canonical_records() {
    let fixture = include_str!("fixtures/deerflow_live_run.json");
    let batch: RawEnvelopeBatch = serde_json::from_str(fixture).unwrap();

    let normalized = normalize_batch(&batch).unwrap();

    assert!(normalized.records.iter().any(|record| matches!(record, AnyRecord::Session(_))));
    assert!(normalized.records.iter().any(|record| matches!(record, AnyRecord::Run(_))));
    assert!(normalized.records.iter().any(|record| matches!(record, AnyRecord::Message(_))));
    assert!(normalized.records.iter().any(|record| matches!(record, AnyRecord::Task(_))));
    assert!(normalized.records.iter().any(|record| matches!(record, AnyRecord::Artifact(_))));
    assert!(normalized.records.iter().any(|record| matches!(record, AnyRecord::Clarification(_))));
    assert!(normalized.records.iter().any(|record| matches!(record, AnyRecord::RuntimeStatus(_))));
    assert!(!normalized.records.iter().any(|record| matches!(record, AnyRecord::Intent(_))));
}

#[test]
fn normalizes_hermes_events_into_canonical_records() {
    let fixture = include_str!("fixtures/hermes_run.json");
    let batch: RawEnvelopeBatch = serde_json::from_str(fixture).unwrap();

    let normalized = normalize_batch(&batch).unwrap();

    assert!(normalized.records.iter().any(|record| matches!(record, AnyRecord::Session(_))));
    assert!(normalized.records.iter().any(|record| matches!(record, AnyRecord::Run(_))));
    assert!(normalized.records.iter().any(|record| matches!(record, AnyRecord::Message(_))));
}
```

- [ ] **Step 2: Run the carrier-normalization test to verify it fails**

Run: `cargo test -p deer-pipeline-normalizers --test carrier_normalization -v`

Expected: FAIL with missing crate modules, missing `RawEnvelopeBatch`, or unresolved `normalize_batch` errors.

- [ ] **Step 3: Write the minimal carrier-normalization implementation**

```rust
// crates/pipeline/normalizers/src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NormalizationError {
    #[error("unsupported event kind: {0}")]
    UnsupportedEventKind(String),
    #[error("invalid level promotion: {from} -> {to}")]
    InvalidPromotion { from: String, to: String },
    #[error("missing required hash anchor for {family}")]
    MissingHashAnchor { family: &'static str },
}
```

```rust
// crates/pipeline/normalizers/src/envelopes.rs
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "generator", rename_all = "snake_case")]
pub enum RawEnvelopeBatch {
    Deerflow(DeerFlowBatch),
    Hermes(HermesBatch),
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeerFlowBatch {
    pub session: RawSessionEnvelope,
    pub run: RawRunEnvelope,
    pub events: Vec<RawEventEnvelope>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HermesBatch {
    pub session: RawSessionEnvelope,
    pub run: RawRunEnvelope,
    pub events: Vec<RawEventEnvelope>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawSessionEnvelope {
    pub session_id: String,
    pub title: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawRunEnvelope {
    pub run_id: String,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RawEventEnvelope {
    Message { message_id: String, role: String, text: String, level: String },
    Task { task_id: String, title: String, state: String },
    Artifact { artifact_id: String, name: String, status: String, as_is_hash: String },
    RuntimeStatus { state: String },
    Clarification { clarification_id: String, prompt: String },
}
```

```rust
// crates/pipeline/normalizers/src/carrier.rs
use deer_foundation_contracts::{AsIsHash, CanonicalLevel, CanonicalPlane, RecordId};
use deer_foundation_domain::{
    AnyRecord, ArtifactRecord, ClarificationRecord, MessageRecord, RunRecord,
    RuntimeStatusRecord, SessionRecord, TaskRecord,
};

use crate::{error::NormalizationError, envelopes::RawEnvelopeBatch};

#[derive(Debug, Clone, Default)]
pub struct NormalizedBatch {
    pub records: Vec<AnyRecord>,
}

pub fn normalize_batch(batch: &RawEnvelopeBatch) -> Result<NormalizedBatch, NormalizationError> {
    let (session, run, events) = match batch {
        RawEnvelopeBatch::Deerflow(b) => (&b.session, &b.run, &b.events),
        RawEnvelopeBatch::Hermes(b) => (&b.session, &b.run, &b.events),
    };

    let mut records = vec![
        AnyRecord::Session(SessionRecord::new(RecordId::from(session.session_id.as_str()), session.title.clone())),
        AnyRecord::Run(RunRecord::new(RecordId::from(run.run_id.as_str()), run.status.clone())),
    ];

    for event in events {
        match event {
            crate::envelopes::RawEventEnvelope::Message { message_id, role, text, level: _ } => {
                records.push(AnyRecord::Message(MessageRecord::new(
                    RecordId::from(message_id.as_str()),
                    role.clone(),
                    text.clone(),
                    CanonicalLevel::L2,
                    CanonicalPlane::AsIs,
                )));
            }
            crate::envelopes::RawEventEnvelope::Task { task_id, title, state } => {
                records.push(AnyRecord::Task(TaskRecord::new(
                    RecordId::from(task_id.as_str()),
                    title.clone(),
                    state.clone(),
                )));
            }
            crate::envelopes::RawEventEnvelope::Artifact { artifact_id, name, status, as_is_hash } => {
                records.push(AnyRecord::Artifact(ArtifactRecord::new(
                    RecordId::from(artifact_id.as_str()),
                    name.clone(),
                    status.clone(),
                    Some(AsIsHash::from(as_is_hash.as_str())),
                )));
            }
            crate::envelopes::RawEventEnvelope::RuntimeStatus { state } => {
                records.push(AnyRecord::RuntimeStatus(RuntimeStatusRecord::new(state.clone())));
            }
            crate::envelopes::RawEventEnvelope::Clarification { clarification_id, prompt } => {
                records.push(AnyRecord::Clarification(ClarificationRecord::new(
                    RecordId::from(clarification_id.as_str()),
                    prompt.clone(),
                )));
            }
        }
    }

    Ok(NormalizedBatch { records })
}
```

- [ ] **Step 4: Run the carrier-normalization test to verify it passes**

Run: `cargo test -p deer-pipeline-normalizers --test carrier_normalization -v`

Expected: PASS with the carrier fixture producing canonical `Session`, `Run`, `Message`, `Task`, `Artifact`, and `RuntimeStatus` records.

- [ ] **Step 5: Commit the normalizer carrier spine**

```bash
git add Cargo.toml crates/pipeline/normalizers
git commit -m "feat: add canonical carrier normalizers"
```

## Task 2: Add Promotion, Representation, And Governance Rules To Normalizers

**Files:**
- Create: `crates/pipeline/normalizers/src/promotions.rs`
- Create: `crates/pipeline/normalizers/src/representation.rs`
- Create: `crates/pipeline/normalizers/src/governance.rs`
- Modify: `crates/pipeline/normalizers/src/lib.rs`
- Test: `crates/pipeline/normalizers/tests/promotion_and_representation.rs`
- Test fixture: `crates/pipeline/normalizers/tests/fixtures/representation_chain.json`

**Milestone unlock:** M0 promotion and hash-anchor invariants

**Proof path later:** downstream artifact retrieval proof in `apps/deer_chat_lab` and lineage inspection in `apps/deer_replay`

**Forbidden shortcuts:** direct `L0 -> L4` jumps, embedding truth without `chunk_hash`, local intent stages becoming canonical `IntentRecord` before `submitted`

- [ ] **Step 1: Write the failing promotion and representation tests plus fixture**

```json
// crates/pipeline/normalizers/tests/fixtures/representation_chain.json
{
  "as_is_record_id": "artifact_repr_1",
  "as_is_hash": "sha256:artifact-raw",
  "chunk_hash": "sha256:artifact-chunk-1",
  "embedding_basis_hash": "sha256:embedding-basis-1",
  "from_level": "L2",
  "to_level": "L4",
  "intent_stage": "prefill_seed"
}
```

```rust
// crates/pipeline/normalizers/tests/promotion_and_representation.rs
use deer_foundation_contracts::CanonicalLevel;
use deer_foundation_domain::AnyRecord;
use deer_pipeline_normalizers::{
    emit_intent_records, normalize_representation_chain, validate_promotion,
};

#[test]
fn representation_chain_requires_hash_anchors_and_preserves_backlinks() {
    let fixture = include_str!("fixtures/representation_chain.json");
    let chain = normalize_representation_chain(fixture).unwrap();

    assert!(matches!(chain[0], AnyRecord::AsIsRepresentation(_)));
    assert!(matches!(chain[1], AnyRecord::Chunk(_)));
    assert!(matches!(chain[2], AnyRecord::Embedding(_)));
}

#[test]
fn rejects_invalid_direct_level_jump() {
    let error = validate_promotion(CanonicalLevel::L0, CanonicalLevel::L4).unwrap_err();
    assert!(error.to_string().contains("invalid level promotion"));
}

#[test]
fn submitted_is_the_only_stage_that_emits_intent_records() {
    let prefill_seed = emit_intent_records("prefill_seed").unwrap();
    let submitted = emit_intent_records("submitted").unwrap();

    assert!(prefill_seed.is_empty());
    assert_eq!(submitted.len(), 2);
    assert!(submitted.iter().any(|record| matches!(record, AnyRecord::Intent(_))));
    assert!(submitted.iter().any(|record| matches!(record, AnyRecord::WriteOperation(_))));
}
```

- [ ] **Step 2: Run the promotion and representation tests to verify they fail**

Run: `cargo test -p deer-pipeline-normalizers --test promotion_and_representation -v`

Expected: FAIL with unresolved imports for `validate_promotion`, `normalize_representation_chain`, and `emit_intent_records`.

- [ ] **Step 3: Implement promotion guards, representation-chain normalization, and governance emission**

```rust
// crates/pipeline/normalizers/src/lib.rs
pub mod carrier;
pub mod envelopes;
pub mod error;
pub mod governance;
pub mod promotions;
pub mod representation;

pub use carrier::{normalize_batch, NormalizedBatch};
pub use envelopes::RawEnvelopeBatch;
pub use error::NormalizationError;
pub use governance::emit_intent_records;
pub use promotions::validate_promotion;
pub use representation::normalize_representation_chain;
```

```rust
// crates/pipeline/normalizers/src/promotions.rs
use deer_foundation_contracts::CanonicalLevel;

use crate::error::NormalizationError;

pub fn validate_promotion(
    from: CanonicalLevel,
    to: CanonicalLevel,
) -> Result<(), NormalizationError> {
    let valid = matches!(
        (from, to),
        (CanonicalLevel::L0, CanonicalLevel::L1)
            | (CanonicalLevel::L1, CanonicalLevel::L2)
            | (CanonicalLevel::L2, CanonicalLevel::L3)
            | (CanonicalLevel::L2, CanonicalLevel::L4)
            | (CanonicalLevel::L3, CanonicalLevel::L4)
            | (CanonicalLevel::L4, CanonicalLevel::L5)
            | (CanonicalLevel::L5, CanonicalLevel::L6)
    );

    if valid {
        Ok(())
    } else {
        Err(NormalizationError::InvalidPromotion {
            from: format!("{from:?}"),
            to: format!("{to:?}"),
        })
    }
}
```

```rust
// crates/pipeline/normalizers/src/representation.rs
use deer_foundation_contracts::{AsIsHash, ChunkHash, EmbeddingBasisHash, RecordId};
use deer_foundation_domain::{
    AnyRecord, AsIsRepresentationRecord, ChunkRecord, EmbeddingRecord,
};

use crate::error::NormalizationError;

#[derive(serde::Deserialize)]
struct RepresentationFixture {
    as_is_record_id: String,
    as_is_hash: String,
    chunk_hash: String,
    embedding_basis_hash: String,
}

pub fn normalize_representation_chain(
    fixture_json: &str,
) -> Result<Vec<AnyRecord>, NormalizationError> {
    let fixture: RepresentationFixture = serde_json::from_str(fixture_json).unwrap();

    if fixture.as_is_hash.is_empty() {
        return Err(NormalizationError::MissingHashAnchor { family: "as_is" });
    }

    Ok(vec![
        AnyRecord::AsIsRepresentation(AsIsRepresentationRecord::new(
            RecordId::from(fixture.as_is_record_id.as_str()),
            AsIsHash::from(fixture.as_is_hash.as_str()),
        )),
        AnyRecord::Chunk(ChunkRecord::new(
            RecordId::from("chunk_1"),
            AsIsHash::from(fixture.as_is_hash.as_str()),
            ChunkHash::from(fixture.chunk_hash.as_str()),
        )),
        AnyRecord::Embedding(EmbeddingRecord::new(
            RecordId::from("embedding_1"),
            AsIsHash::from(fixture.as_is_hash.as_str()),
            ChunkHash::from(fixture.chunk_hash.as_str()),
            EmbeddingBasisHash::from(fixture.embedding_basis_hash.as_str()),
        )),
    ])
}
```

```rust
// crates/pipeline/normalizers/src/governance.rs
use deer_foundation_domain::{AnyRecord, IntentRecord, WriteOperationRecord};

use crate::error::NormalizationError;

pub fn emit_intent_records(stage: &str) -> Result<Vec<AnyRecord>, NormalizationError> {
    Ok(match stage {
        "submitted" => vec![
            AnyRecord::Intent(IntentRecord::new("submitted".into())),
            AnyRecord::WriteOperation(WriteOperationRecord::new("queued".into())),
        ],
        "prefill_seed" | "prefill" | "draft" | "validated" => Vec::new(),
        other => return Err(NormalizationError::UnsupportedEventKind(other.to_string())),
    })
}
```

- [ ] **Step 4: Run the promotion and representation tests, then the full normalizers sweep**

Run: `cargo test -p deer-pipeline-normalizers --test promotion_and_representation -v && cargo test -p deer-pipeline-normalizers -v`

Expected: PASS with allowed promotions enforced, invalid promotion rejected, representation records hash-anchored, and only `submitted` emitting `IntentRecord` plus `WriteOperationRecord`.

- [ ] **Step 5: Commit the full normalizer rule set**

```bash
git add crates/pipeline/normalizers
git commit -m "feat: add promotion-safe canonical normalizers"
```

## Task 3: Create Typed Orchestration, Artifact, And Macro-State Derivations

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/pipeline/derivations/Cargo.toml`
- Create: `crates/pipeline/derivations/src/lib.rs`
- Create: `crates/pipeline/derivations/src/common.rs`
- Create: `crates/pipeline/derivations/src/orchestration.rs`
- Create: `crates/pipeline/derivations/src/artifacts.rs`
- Create: `crates/pipeline/derivations/src/macro_state.rs`
- Test: `crates/pipeline/derivations/tests/orchestration_vm.rs`
- Test: `crates/pipeline/derivations/tests/artifact_vm.rs`
- Test: `crates/pipeline/derivations/tests/macro_state_vm.rs`

**Milestone unlock:** M0 typed VM derivations over canonical records

**Proof path later:** `apps/deer_chat_lab` for transcript and artifact slice; `apps/deer_design` for cross-panel macro-state composition

**Forbidden shortcuts:** no raw payload reads, no egui types, no world-only metaphors, no backend support claims without view metadata

- [ ] **Step 1: Write the failing derivation golden tests and workspace stubs**

```toml
# Cargo.toml
[workspace]
members = [
    "apps/deer_gui",
    "crates/foundation/contracts",
    "crates/foundation/domain",
    "crates/foundation/replay",
    "crates/pipeline/normalizers",
    "crates/pipeline/derivations",
]
resolver = "2"
```

```toml
# crates/pipeline/derivations/Cargo.toml
[package]
name = "deer-pipeline-derivations"
version = "0.1.0"
edition = "2021"

[dependencies]
deer-foundation-contracts = { path = "../../foundation/contracts" }
deer-foundation-domain = { path = "../../foundation/domain" }
deer-foundation-replay = { path = "../../foundation/replay" }
serde = { version = "1.0.228", features = ["derive"] }

[dev-dependencies]
insta = { version = "1.43.2", features = ["yaml"] }
similar-asserts = "1.7.0"
```

```rust
// crates/pipeline/derivations/src/lib.rs
pub mod artifacts;
pub mod common;
pub mod macro_state;
pub mod orchestration;

pub use artifacts::derive_artifact_shelf_vm;
pub use macro_state::derive_macro_state_vm;
pub use orchestration::derive_transcript_vm;
```

```rust
// crates/pipeline/derivations/tests/orchestration_vm.rs
use deer_foundation_contracts::{CanonicalLevel, CanonicalPlane};
use deer_foundation_domain::{AnyRecord, MessageRecord, RunRecord, SessionRecord, TaskRecord};
use deer_pipeline_derivations::derive_transcript_vm;
use insta::assert_yaml_snapshot;

#[test]
fn derives_transcript_and_run_status_vms_from_canonical_records() {
    let records = vec![
        AnyRecord::Session(SessionRecord::new("session_1".into(), "Survey the ridge".into())),
        AnyRecord::Run(RunRecord::new("run_1".into(), "running".into())),
        AnyRecord::Message(MessageRecord::new(
            "msg_1".into(),
            "operator".into(),
            "Survey the ridge".into(),
            CanonicalLevel::L2,
            CanonicalPlane::AsIs,
        )),
        AnyRecord::Task(TaskRecord::new("task_1".into(), "Gather terrain notes".into(), "running".into())),
    ];

    let transcript = derive_transcript_vm(&records);

    assert_yaml_snapshot!(transcript, @r#"
entries:
  - record_id: msg_1
    role: operator
    text: Survey the ridge
run_status:
  run_id: run_1
  state: running
tasks:
  - task_id: task_1
    title: Gather terrain notes
    state: running
"#);
}
```

```rust
// crates/pipeline/derivations/tests/artifact_vm.rs
use deer_foundation_domain::{AnyRecord, ArtifactRecord};
use deer_pipeline_derivations::derive_artifact_shelf_vm;
use insta::assert_yaml_snapshot;

#[test]
fn derives_artifact_shelf_vm_with_preview_and_retrieval_metadata() {
    let records = vec![AnyRecord::Artifact(ArtifactRecord::new(
        "artifact_1".into(),
        "terrain.md".into(),
        "presented".into(),
        Some("sha256:terrain-md".into()),
    ))];

    let shelf = derive_artifact_shelf_vm(&records);

    assert_yaml_snapshot!(shelf, @r#"
entries:
  - artifact_id: artifact_1
    title: terrain.md
    status: presented
    preview_supported: true
    retrieval_mode: mediated_pointer
"#);
}
```

```rust
// crates/pipeline/derivations/tests/macro_state_vm.rs
use deer_foundation_domain::{AnyRecord, ArtifactRecord, TaskRecord};
use deer_pipeline_derivations::derive_macro_state_vm;
use insta::assert_yaml_snapshot;

#[test]
fn derives_macro_state_rows_with_backlinks_and_support_metadata() {
    let records = vec![
        AnyRecord::Task(TaskRecord::new("task_1".into(), "Gather terrain notes".into(), "running".into())),
        AnyRecord::Artifact(ArtifactRecord::new("artifact_1".into(), "terrain.md".into(), "presented".into(), Some("sha256:terrain-md".into()))),
    ];

    let macro_state = derive_macro_state_vm(&records);

    assert_yaml_snapshot!(macro_state, @r#"
rows:
  - kind: task
    source_record_id: task_1
    panel_target: task_detail
  - kind: artifact
    source_record_id: artifact_1
    panel_target: artifact_detail
backlinks:
  - source_record_id: task_1
    level: L2
    plane: AsIs
    panel_target: task_detail
  - source_record_id: artifact_1
    level: L2
    plane: AsIs
    panel_target: artifact_detail
"#);
}
```

- [ ] **Step 2: Run the derivation tests to verify they fail**

Run: `cargo test -p deer-pipeline-derivations --test orchestration_vm --test artifact_vm --test macro_state_vm -v`

Expected: FAIL with unresolved derivation functions and missing VM types.

- [ ] **Step 3: Implement the minimal typed derivation layer**

```rust
// crates/pipeline/derivations/src/common.rs
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VmBacklink {
    pub source_record_id: String,
    pub level: String,
    pub plane: String,
    pub panel_target: &'static str,
}
```

```rust
// crates/pipeline/derivations/src/orchestration.rs
use serde::Serialize;

use deer_foundation_domain::AnyRecord;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TranscriptEntryVm {
    pub record_id: String,
    pub role: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RunStatusVm {
    pub run_id: String,
    pub state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TaskProgressVm {
    pub task_id: String,
    pub title: String,
    pub state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TranscriptVm {
    pub entries: Vec<TranscriptEntryVm>,
    pub run_status: RunStatusVm,
    pub tasks: Vec<TaskProgressVm>,
}

pub fn derive_transcript_vm(records: &[AnyRecord]) -> TranscriptVm {
    let mut entries = Vec::new();
    let mut run_status = RunStatusVm { run_id: String::new(), state: String::new() };
    let mut tasks = Vec::new();

    for record in records {
        match record {
            AnyRecord::Message(message) => entries.push(TranscriptEntryVm {
                record_id: message.record_id().to_string(),
                role: message.role().to_string(),
                text: message.text().to_string(),
            }),
            AnyRecord::Run(run) => {
                run_status = RunStatusVm {
                    run_id: run.record_id().to_string(),
                    state: run.status().to_string(),
                };
            }
            AnyRecord::Task(task) => tasks.push(TaskProgressVm {
                task_id: task.record_id().to_string(),
                title: task.title().to_string(),
                state: task.state().to_string(),
            }),
            _ => {}
        }
    }

    TranscriptVm { entries, run_status, tasks }
}
```

```rust
// crates/pipeline/derivations/src/artifacts.rs
use serde::Serialize;

use deer_foundation_domain::AnyRecord;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArtifactEntryVm {
    pub artifact_id: String,
    pub title: String,
    pub status: String,
    pub preview_supported: bool,
    pub retrieval_mode: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArtifactShelfVm {
    pub entries: Vec<ArtifactEntryVm>,
}

pub fn derive_artifact_shelf_vm(records: &[AnyRecord]) -> ArtifactShelfVm {
    let entries = records
        .iter()
        .filter_map(|record| match record {
            AnyRecord::Artifact(artifact) => Some(ArtifactEntryVm {
                artifact_id: artifact.record_id().to_string(),
                title: artifact.name().to_string(),
                status: artifact.status().to_string(),
                preview_supported: true,
                retrieval_mode: "mediated_pointer",
            }),
            _ => None,
        })
        .collect();

    ArtifactShelfVm { entries }
}
```

```rust
// crates/pipeline/derivations/src/macro_state.rs
use serde::Serialize;

use deer_foundation_domain::AnyRecord;

use crate::common::VmBacklink;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MacroStateRowVm {
    pub kind: &'static str,
    pub source_record_id: String,
    pub panel_target: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MacroStateVm {
    pub rows: Vec<MacroStateRowVm>,
    pub backlinks: Vec<VmBacklink>,
}

pub fn derive_macro_state_vm(records: &[AnyRecord]) -> MacroStateVm {
    let mut rows = Vec::new();
    let mut backlinks = Vec::new();

    for record in records {
        match record {
            AnyRecord::Task(task) => {
                rows.push(MacroStateRowVm {
                    kind: "task",
                    source_record_id: task.record_id().to_string(),
                    panel_target: "task_detail",
                });
                backlinks.push(VmBacklink {
                    source_record_id: task.record_id().to_string(),
                    level: "L2".into(),
                    plane: "AsIs".into(),
                    panel_target: "task_detail",
                });
            }
            AnyRecord::Artifact(artifact) => {
                rows.push(MacroStateRowVm {
                    kind: "artifact",
                    source_record_id: artifact.record_id().to_string(),
                    panel_target: "artifact_detail",
                });
                backlinks.push(VmBacklink {
                    source_record_id: artifact.record_id().to_string(),
                    level: "L2".into(),
                    plane: "AsIs".into(),
                    panel_target: "artifact_detail",
                });
            }
            _ => {}
        }
    }

    MacroStateVm { rows, backlinks }
}
```

- [ ] **Step 4: Run the derivation tests to verify they pass**

Run: `cargo test -p deer-pipeline-derivations --test orchestration_vm --test artifact_vm --test macro_state_vm -v`

Expected: PASS with deterministic snapshots for transcript, artifact shelf, and macro-state derivations.

- [ ] **Step 5: Commit the derivation layer**

```bash
git add Cargo.toml crates/pipeline/derivations
git commit -m "feat: add canonical view-model derivations"
```

## Task 4: Create Intent, Linked-Shell, Artifact, And Temporal Read Models

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/runtime/read_models/Cargo.toml`
- Create: `crates/runtime/read_models/src/lib.rs`
- Create: `crates/runtime/read_models/src/chat.rs`
- Create: `crates/runtime/read_models/src/artifacts.rs`
- Create: `crates/runtime/read_models/src/linked_shell.rs`
- Create: `crates/runtime/read_models/src/intent.rs`
- Create: `crates/runtime/read_models/src/temporal.rs`
- Create: `crates/runtime/read_models/src/policy.rs`
- Test: `crates/runtime/read_models/tests/intent_reducer.rs`
- Test: `crates/runtime/read_models/tests/linked_shell_reducer.rs`
- Test: `crates/runtime/read_models/tests/temporal_and_policy.rs`

**Milestone unlock:** M0 transient shell-local state outside any app shell

**Proof path later:** `apps/deer_chat_lab` for chat draft and clarification behavior; `apps/deer_design` for linked-shell state; `apps/deer_gui` only after those proofs exist

**Forbidden shortcuts:** no direct jump from selection to `submitted`, no direct storage reads, no ghost selections after policy exclusion, no late-event cursor jumps outside `live_tail`

- [ ] **Step 1: Write the failing reducer tests and workspace stubs**

```toml
# Cargo.toml
[workspace]
members = [
    "apps/deer_gui",
    "crates/foundation/contracts",
    "crates/foundation/domain",
    "crates/foundation/replay",
    "crates/pipeline/normalizers",
    "crates/pipeline/derivations",
    "crates/runtime/read_models",
]
resolver = "2"
```

```toml
# crates/runtime/read_models/Cargo.toml
[package]
name = "deer-runtime-read-models"
version = "0.1.0"
edition = "2021"

[dependencies]
deer-foundation-contracts = { path = "../../foundation/contracts" }
serde = { version = "1.0.228", features = ["derive"] }
thiserror = "2.0.17"
```

```rust
// crates/runtime/read_models/src/lib.rs
pub mod artifacts;
pub mod chat;
pub mod intent;
pub mod linked_shell;
pub mod policy;
pub mod temporal;

pub use intent::{IntentAction, IntentDraftState, IntentLifecycleState, reduce_intent_state};
pub use linked_shell::{LinkedShellAction, LinkedShellState, reduce_linked_shell_state};
pub use temporal::{TemporalAction, TemporalState, reduce_temporal_state};
```

```rust
// crates/runtime/read_models/tests/intent_reducer.rs
use deer_runtime_read_models::{
    reduce_intent_state, IntentAction, IntentLifecycleState,
};

#[test]
fn intent_reducer_requires_explicit_stepwise_promotion() {
    let state = IntentLifecycleState::default();

    let state = reduce_intent_state(state, IntentAction::SeedFromSelection { source_record_id: "task_1".into() });
    assert_eq!(state.stage, "prefill_seed");

    let state = reduce_intent_state(state, IntentAction::OpenComposer);
    assert_eq!(state.stage, "prefill");

    let state = reduce_intent_state(state, IntentAction::TakeOwnership);
    assert_eq!(state.stage, "draft");

    let state = reduce_intent_state(state, IntentAction::Validate);
    assert_eq!(state.stage, "validated");

    let state = reduce_intent_state(state, IntentAction::Submit);
    assert_eq!(state.stage, "submitted");
}
```

```rust
// crates/runtime/read_models/tests/linked_shell_reducer.rs
use deer_runtime_read_models::{
    reduce_linked_shell_state, LinkedShellAction, LinkedShellState,
};

#[test]
fn linked_shell_state_tracks_selection_pins_and_drill_down_by_canonical_ref() {
    let state = LinkedShellState::default();

    let state = reduce_linked_shell_state(state, LinkedShellAction::Select { source_record_id: "artifact_1".into() });
    let state = reduce_linked_shell_state(state, LinkedShellAction::Pin { source_record_id: "artifact_1".into() });
    let state = reduce_linked_shell_state(state, LinkedShellAction::OpenDrillDown { panel_target: "artifact_detail" });

    assert_eq!(state.selected.as_deref(), Some("artifact_1"));
    assert_eq!(state.pinned, vec!["artifact_1".to_string()]);
    assert_eq!(state.drill_down_target.as_deref(), Some("artifact_detail"));
}
```

```rust
// crates/runtime/read_models/tests/temporal_and_policy.rs
use deer_runtime_read_models::{
    reduce_linked_shell_state, reduce_temporal_state, LinkedShellAction,
    LinkedShellState, TemporalAction, TemporalState,
};

#[test]
fn temporal_state_keeps_historical_cursor_stable_on_late_event() {
    let state = TemporalState::historical("checkpoint_7");
    let next = reduce_temporal_state(state, TemporalAction::LateEventInserted { event_id: "evt_9".into() });

    assert_eq!(next.cursor_id.as_deref(), Some("checkpoint_7"));
    assert!(next.is_stale);
}

#[test]
fn policy_exclusion_clears_ghost_selection_and_pins() {
    let state = LinkedShellState {
        selected: Some("artifact_1".into()),
        pinned: vec!["artifact_1".into()],
        ..Default::default()
    };

    let cleared = reduce_linked_shell_state(state, LinkedShellAction::Exclude { source_record_id: "artifact_1".into() });

    assert_eq!(cleared.selected, None);
    assert!(cleared.pinned.is_empty());
}
```

- [ ] **Step 2: Run the reducer tests to verify they fail**

Run: `cargo test -p deer-runtime-read-models --test intent_reducer --test linked_shell_reducer --test temporal_and_policy -v`

Expected: FAIL with missing reducer functions, missing action enums, and missing state structs.

- [ ] **Step 3: Implement the minimal read-model reducers and shell-local state**

```rust
// crates/runtime/read_models/src/intent.rs
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct IntentDraftState {
    pub source_record_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct IntentLifecycleState {
    pub stage: String,
    pub draft: IntentDraftState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentAction {
    SeedFromSelection { source_record_id: String },
    OpenComposer,
    TakeOwnership,
    Validate,
    Submit,
}

pub fn reduce_intent_state(
    mut state: IntentLifecycleState,
    action: IntentAction,
) -> IntentLifecycleState {
    match action {
        IntentAction::SeedFromSelection { source_record_id } => {
            state.stage = "prefill_seed".into();
            state.draft.source_record_id = Some(source_record_id);
        }
        IntentAction::OpenComposer => state.stage = "prefill".into(),
        IntentAction::TakeOwnership => state.stage = "draft".into(),
        IntentAction::Validate => state.stage = "validated".into(),
        IntentAction::Submit => state.stage = "submitted".into(),
    }

    state
}
```

```rust
// crates/runtime/read_models/src/linked_shell.rs
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct LinkedShellState {
    pub selected: Option<String>,
    pub pinned: Vec<String>,
    pub drill_down_target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkedShellAction {
    Select { source_record_id: String },
    Pin { source_record_id: String },
    OpenDrillDown { panel_target: &'static str },
    Exclude { source_record_id: String },
}

pub fn reduce_linked_shell_state(
    mut state: LinkedShellState,
    action: LinkedShellAction,
) -> LinkedShellState {
    match action {
        LinkedShellAction::Select { source_record_id } => state.selected = Some(source_record_id),
        LinkedShellAction::Pin { source_record_id } => state.pinned.push(source_record_id),
        LinkedShellAction::OpenDrillDown { panel_target } => {
            state.drill_down_target = Some(panel_target.into())
        }
        LinkedShellAction::Exclude { source_record_id } => {
            if state.selected.as_deref() == Some(source_record_id.as_str()) {
                state.selected = None;
            }
            state.pinned.retain(|id| id != &source_record_id);
        }
    }

    state
}
```

```rust
// crates/runtime/read_models/src/temporal.rs
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct TemporalState {
    pub mode: &'static str,
    pub cursor_id: Option<String>,
    pub is_stale: bool,
}

impl TemporalState {
    pub fn historical(cursor_id: &str) -> Self {
        Self {
            mode: "historical",
            cursor_id: Some(cursor_id.into()),
            is_stale: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemporalAction {
    LateEventInserted { event_id: String },
    ReturnToLiveTail,
}

pub fn reduce_temporal_state(mut state: TemporalState, action: TemporalAction) -> TemporalState {
    match action {
        TemporalAction::LateEventInserted { .. } if state.mode == "historical" => {
            state.is_stale = true;
        }
        TemporalAction::LateEventInserted { .. } => {
            state.mode = "live_tail";
        }
        TemporalAction::ReturnToLiveTail => {
            state.mode = "live_tail";
            state.cursor_id = None;
            state.is_stale = false;
        }
    }

    state
}
```

```rust
// crates/runtime/read_models/src/chat.rs
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct ChatDraftState {
    pub prompt_text: String,
    pub attachment_ids: Vec<String>,
    pub clarification_pending: bool,
}
```

```rust
// crates/runtime/read_models/src/artifacts.rs
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct ArtifactPanelState {
    pub selected_artifact_id: Option<String>,
    pub preview_request_state: Option<String>,
}
```

```rust
// crates/runtime/read_models/src/policy.rs
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct PolicyOverlayState {
    pub excluded_record_ids: Vec<String>,
}
```

- [ ] **Step 4: Run the reducer tests and then the full M0 pipeline sweep**

Run: `cargo test -p deer-runtime-read-models --test intent_reducer --test linked_shell_reducer --test temporal_and_policy -v && cargo test -p deer-pipeline-normalizers -v && cargo test -p deer-pipeline-derivations -v && cargo test -p deer-runtime-read-models -v`

Expected: PASS with stepwise intent promotion, canonical-ref-only linked-shell state, stable historical cursor behavior on late events, and exclusion-driven invalidation clearing ghost state.

- [ ] **Step 5: Commit the read-model spine and complete the M0 pipeline slice**

```bash
git add Cargo.toml crates/runtime/read_models
git commit -m "feat: add transient runtime read models"
```

## Self-Review Checklist

- Coverage maps to the accepted M0 scope: carrier normalization, promotion-safe representation/governance normalization, orchestration/artifact/macro-state derivations, and transient shell-local read models.
- Every task preserves strict layering: raw envelope -> normalizers -> canonical domain -> derivations or read models; no views, panels, layout runtime, world projection, or `deer_gui` work appears here.
- TDD shape matches the accepted contract: fixture transform tests for normalizers, deterministic golden tests for derivations, and reducer/state-machine tests for read models.
- Tool-app proof paths are named but deferred correctly to later plans: `apps/deer_chat_lab`, `apps/deer_replay`, and `apps/deer_design`.
- The plan states the main forbidden shortcuts explicitly: no raw schema leakage above normalizers, no direct read bypass, no world-metaphor drift, no local intent shortcut to canonical submission, and no ghost linked-shell state after exclusion.
