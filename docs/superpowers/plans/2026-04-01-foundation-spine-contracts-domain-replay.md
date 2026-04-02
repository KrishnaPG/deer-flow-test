# Foundation Spine Contracts, Domain, And Replay Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first reusable Rust foundation spine: typed contracts, canonical domain records, and replay-safe event history primitives that every later normalizer, derivation, and proof app can share.

**Architecture:** Introduce three new workspace crates in dependency order: `deer-foundation-contracts` for shared IDs/envelopes/enums/command-event DTOs, `deer-foundation-domain` for canonical record families plus immutable lineage-safe headers, and `deer-foundation-replay` for append-only history, fixture loading, and replay cursors. Keep this plan strictly pre-app: no Bevy, no egui, no transport adapters, no `deer_gui` wiring, and no proof-app code.

**Tech Stack:** Rust 2021 workspace crates, `serde`, `serde_json`, `chrono`, `uuid`, `thiserror`

---

## Scope Boundary

- This plan covers only `foundation/contracts`, `foundation/domain`, and `foundation/replay`.
- It does **not** implement `pipeline/normalizers`, `pipeline/derivations`, or `runtime/read_models`; those belong to `docs/superpowers/plans/2026-04-01-normalization-derivation-read-model-spine.md`.
- It does **not** create proof apps. The first proof-app path for this foundation spine is downstream consumption by `apps/deer_chat_lab`, `apps/deer_replay`, `apps/deer_graph_lab`, and `apps/deer_design` in later plans.
- It does **not** modify `apps/deer_gui`.

## File Structure

### Workspace

- Modify: `Cargo.toml` - add foundation workspace members incrementally as each crate is introduced.

### Contracts crate

- Create: `crates/foundation/contracts/Cargo.toml` - package manifest for shared contracts.
- Create: `crates/foundation/contracts/src/lib.rs` - public exports.
- Create: `crates/foundation/contracts/src/ids.rs` - strong ID and hash-anchor newtypes.
- Create: `crates/foundation/contracts/src/meta.rs` - identity, correlation, and lineage envelopes.
- Create: `crates/foundation/contracts/src/records.rs` - levels, planes, record families, storage disposition, and `RecordHeader`.
- Create: `crates/foundation/contracts/src/commands.rs` - intent command DTOs.
- Create: `crates/foundation/contracts/src/events.rs` - append-only replay/write envelopes and cursors.
- Create: `crates/foundation/contracts/tests/contracts_smoke.rs` - contract-level tests for IDs, headers, and hash anchors.
- Create: `crates/foundation/contracts/tests/intent_and_replay_contracts.rs` - contract-level tests for command/event DTOs.

### Domain crate

- Create: `crates/foundation/domain/Cargo.toml` - package manifest for canonical records.
- Create: `crates/foundation/domain/src/lib.rs` - grouped exports plus `AnyRecord` enum.
- Create: `crates/foundation/domain/src/common.rs` - shared constructor helpers and header builders.
- Create: `crates/foundation/domain/src/carrier.rs` - `RunRecord`, `SessionRecord`, `TaskRecord`, `MessageRecord`, `ToolCallRecord`, `ArtifactRecord`, `ClarificationRecord`, `RuntimeStatusRecord`, `DeliveryRecord`.
- Create: `crates/foundation/domain/src/semantic.rs` - `L0_SourceRecord` through `L6_OutcomeRecord`.
- Create: `crates/foundation/domain/src/representation.rs` - `AsIsRepresentationRecord`, `ChunkRecord`, `EmbeddingRecord`.
- Create: `crates/foundation/domain/src/governance.rs` - `IntentRecord`, `TransformRecord`, `ExclusionRecord`, `ConflictRecord`, `ResolutionRecord`, `ReplayCheckpointRecord`, `DedupRecord`, `BatchRecord`, `BranchRecord`, `VersionRecord`, `WriteOperationRecord`.
- Create: `crates/foundation/domain/src/structural.rs` - `GraphNodeRecord`, `GraphEdgeRecord`, `KnowledgeEntityRecord`, `KnowledgeRelationRecord`.
- Create: `crates/foundation/domain/tests/domain_record_invariants.rs` - family/level/plane/lineage invariant tests.

### Replay crate

- Create: `crates/foundation/replay/Cargo.toml` - package manifest for replay-safe history.
- Create: `crates/foundation/replay/src/lib.rs` - public exports.
- Create: `crates/foundation/replay/src/error.rs` - replay/fixture error types.
- Create: `crates/foundation/replay/src/event_log.rs` - append-only replay log and cursor queries.
- Create: `crates/foundation/replay/src/fixtures.rs` - JSON fixture loading and conversion into a replay log.
- Create: `crates/foundation/replay/tests/replay_flow.rs` - end-to-end replay/fixture tests.
- Create: `crates/foundation/replay/tests/fixtures/foundation_spine_log.json` - representative append-only history fixture.

## Task 1: Create The Workspace And Contracts Spine

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/foundation/contracts/Cargo.toml`
- Create: `crates/foundation/contracts/src/lib.rs`
- Create: `crates/foundation/contracts/src/ids.rs`
- Create: `crates/foundation/contracts/src/meta.rs`
- Create: `crates/foundation/contracts/src/records.rs`
- Test: `crates/foundation/contracts/tests/contracts_smoke.rs`

**Milestone unlock:** M0 foundation/contracts base types

**Proof path later:** downstream use by `apps/deer_chat_lab`, `apps/deer_replay`, `apps/deer_graph_lab`, and `apps/deer_design`

**Forbidden dependencies:** `bevy`, `bevy_egui`, `apps/deer_gui`, transport adapters, backend-specific schema crates

- [ ] **Step 1: Write the failing contracts test and workspace stubs**

```toml
# Cargo.toml
[workspace]
members = [
    "apps/deer_gui",
    "crates/foundation/contracts",
]
resolver = "2"
```

```toml
# crates/foundation/contracts/Cargo.toml
[package]
name = "deer-foundation-contracts"
version = "0.1.0"
edition = "2021"

[dependencies]
chrono = { version = "0.4.44", features = ["clock", "serde"] }
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.149"
uuid = { version = "1.23.0", features = ["serde", "v4"] }
```

```rust
// crates/foundation/contracts/src/lib.rs
pub mod ids;
pub mod meta;
pub mod records;

pub use ids::*;
pub use meta::*;
pub use records::*;
```

```rust
// crates/foundation/contracts/tests/contracts_smoke.rs
use deer_foundation_contracts::{
    AsIsHash, CanonicalLevel, CanonicalPlane, CorrelationMeta, IdentityMeta,
    LineageMeta, RecordFamily, RecordHeader, RecordId, StorageDisposition,
};

#[test]
fn record_header_carries_hash_anchored_identity() {
    let header = RecordHeader::new(
        RecordId::from_static("rec_1"),
        RecordFamily::AsIsRepresentation,
        CanonicalLevel::L2,
        CanonicalPlane::AsIs,
        StorageDisposition::StorageNative,
        IdentityMeta::hash_anchored(
            RecordId::from_static("rec_1"),
            Some(AsIsHash::from_static("sha256:artifact")),
            None,
            None,
        ),
        CorrelationMeta::default(),
        LineageMeta::root(),
    );

    assert_eq!(header.level, CanonicalLevel::L2);
    assert_eq!(header.plane, CanonicalPlane::AsIs);
    assert_eq!(header.family, RecordFamily::AsIsRepresentation);
    assert_eq!(
        header.identity.as_is_hash.as_ref().map(AsIsHash::as_str),
        Some("sha256:artifact")
    );
}
```

- [ ] **Step 2: Run the contracts smoke test to verify it fails**

Run: `cargo test -p deer-foundation-contracts --test contracts_smoke -v`

Expected: FAIL with module-not-found or unresolved import errors for `ids`, `meta`, `records`, and the missing public types.

- [ ] **Step 3: Write the minimal contracts implementation for IDs, metadata, and record headers**

```rust
// crates/foundation/contracts/src/ids.rs
use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! string_newtype {
    ($name:ident) => {
        #[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn from_static(value: &'static str) -> Self {
                Self(value.to_string())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_string())
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

string_newtype!(RecordId);
string_newtype!(MissionId);
string_newtype!(RunId);
string_newtype!(AgentId);
string_newtype!(ArtifactId);
string_newtype!(TraceId);
string_newtype!(ThreadId);
string_newtype!(TaskId);
string_newtype!(EventId);
string_newtype!(AsIsHash);
string_newtype!(ChunkHash);
string_newtype!(EmbeddingBasisHash);
```

```rust
// crates/foundation/contracts/src/meta.rs
use serde::{Deserialize, Serialize};

use crate::{
    ArtifactId, AsIsHash, ChunkHash, EmbeddingBasisHash, EventId, MissionId,
    RecordFamily, RecordId, RunId, TaskId, ThreadId, TraceId,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordRef {
    pub family: RecordFamily,
    pub record_id: RecordId,
}

impl RecordRef {
    pub fn new(family: RecordFamily, record_id: RecordId) -> Self {
        Self { family, record_id }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityMeta {
    pub primary_id: RecordId,
    pub as_is_hash: Option<AsIsHash>,
    pub chunk_hash: Option<ChunkHash>,
    pub embedding_basis_hash: Option<EmbeddingBasisHash>,
}

impl IdentityMeta {
    pub fn hash_anchored(
        primary_id: RecordId,
        as_is_hash: Option<AsIsHash>,
        chunk_hash: Option<ChunkHash>,
        embedding_basis_hash: Option<EmbeddingBasisHash>,
    ) -> Self {
        Self {
            primary_id,
            as_is_hash,
            chunk_hash,
            embedding_basis_hash,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorrelationMeta {
    pub mission_id: Option<MissionId>,
    pub run_id: Option<RunId>,
    pub artifact_id: Option<ArtifactId>,
    pub trace_id: Option<TraceId>,
    pub thread_id: Option<ThreadId>,
    pub task_id: Option<TaskId>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineageMeta {
    pub derived_from: Vec<RecordRef>,
    pub source_events: Vec<EventId>,
    pub supersedes: Option<RecordRef>,
}

impl LineageMeta {
    pub fn root() -> Self {
        Self::default()
    }

    pub fn derived_from(parent: RecordRef) -> Self {
        Self {
            derived_from: vec![parent],
            source_events: Vec::new(),
            supersedes: None,
        }
    }
}
```

```rust
// crates/foundation/contracts/src/records.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{CorrelationMeta, IdentityMeta, LineageMeta, RecordId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonicalLevel {
    L0,
    L1,
    L2,
    L3,
    L4,
    L5,
    L6,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonicalPlane {
    AsIs,
    Chunks,
    Embeddings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageDisposition {
    StorageNative,
    IndexProjection,
    ClientTransient,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordFamily {
    Run,
    Session,
    Task,
    Message,
    ToolCall,
    Artifact,
    Clarification,
    RuntimeStatus,
    Delivery,
    L0Source,
    L1Sanitized,
    L2View,
    L3Insight,
    L4Prediction,
    L5Prescription,
    L6Outcome,
    AsIsRepresentation,
    Chunk,
    Embedding,
    Intent,
    Transform,
    Exclusion,
    Conflict,
    Resolution,
    ReplayCheckpoint,
    Dedup,
    Batch,
    Branch,
    Version,
    WriteOperation,
    GraphNode,
    GraphEdge,
    KnowledgeEntity,
    KnowledgeRelation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordHeader {
    pub record_id: RecordId,
    pub family: RecordFamily,
    pub level: CanonicalLevel,
    pub plane: CanonicalPlane,
    pub storage: StorageDisposition,
    pub identity: IdentityMeta,
    pub correlations: CorrelationMeta,
    pub lineage: LineageMeta,
    pub observed_at: DateTime<Utc>,
}

impl RecordHeader {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        record_id: RecordId,
        family: RecordFamily,
        level: CanonicalLevel,
        plane: CanonicalPlane,
        storage: StorageDisposition,
        identity: IdentityMeta,
        correlations: CorrelationMeta,
        lineage: LineageMeta,
    ) -> Self {
        Self {
            record_id,
            family,
            level,
            plane,
            storage,
            identity,
            correlations,
            lineage,
            observed_at: Utc::now(),
        }
    }
}

pub trait CanonicalRecord {
    fn header(&self) -> &RecordHeader;
}
```

- [ ] **Step 4: Run the contracts smoke test to verify it passes**

Run: `cargo test -p deer-foundation-contracts --test contracts_smoke -v`

Expected: PASS with `record_header_carries_hash_anchored_identity ... ok`.

- [ ] **Step 5: Commit the contracts spine base**

```bash
git add Cargo.toml crates/foundation/contracts/Cargo.toml crates/foundation/contracts/src/lib.rs crates/foundation/contracts/src/ids.rs crates/foundation/contracts/src/meta.rs crates/foundation/contracts/src/records.rs crates/foundation/contracts/tests/contracts_smoke.rs
git commit -m "feat: add foundation contracts spine"
```

## Task 2: Add Intent And Replay DTOs To Contracts

**Files:**
- Modify: `crates/foundation/contracts/src/lib.rs`
- Create: `crates/foundation/contracts/src/commands.rs`
- Create: `crates/foundation/contracts/src/events.rs`
- Test: `crates/foundation/contracts/tests/intent_and_replay_contracts.rs`

**Milestone unlock:** replay-safe command/event DTOs for domain and replay crates

**Proof path later:** later exercised by `apps/deer_chat_lab` and `apps/deer_replay`

**Forbidden dependencies:** app-local state, Bevy event types, raw backend payload structs

- [ ] **Step 1: Write the failing command/event tests and export stubs**

```rust
// crates/foundation/contracts/src/lib.rs
pub mod commands;
pub mod events;
pub mod ids;
pub mod meta;
pub mod records;

pub use commands::*;
pub use events::*;
pub use ids::*;
pub use meta::*;
pub use records::*;
```

```rust
// crates/foundation/contracts/tests/intent_and_replay_contracts.rs
use chrono::Utc;
use deer_foundation_contracts::{
    EventId, IntentAction, IntentCommand, RecordFamily, RecordId, RecordRef,
    ReplayDirection, ReplayEnvelope, ThreadId, WriteOperation, WriteOperationKind,
};
use serde_json::json;

#[test]
fn intent_command_carries_target_and_payload() {
    let command = IntentCommand::new(
        RecordId::from_static("cmd_1"),
        IntentAction::SubmitMessage,
        Some(RecordRef::new(
            RecordFamily::Session,
            RecordId::from_static("session_1"),
        )),
        Some(ThreadId::from_static("thread_1")),
        json!({"text": "scan sector seven"}),
    );

    assert!(matches!(command.action, IntentAction::SubmitMessage));
    assert_eq!(command.thread_id.as_ref().map(ThreadId::as_str), Some("thread_1"));
}

#[test]
fn replay_envelope_keeps_append_only_links() {
    let envelope = ReplayEnvelope::append(
        7,
        EventId::from_static("evt_7"),
        RecordRef::new(RecordFamily::Run, RecordId::from_static("run_1")),
        WriteOperation::new(
            WriteOperationKind::AppendRecord,
            RecordId::from_static("run_1"),
        ),
        Some(EventId::from_static("evt_6")),
        Utc::now(),
    );

    assert_eq!(envelope.sequence, 7);
    assert_eq!(envelope.cursor().direction, ReplayDirection::Forward);
    assert_eq!(
        envelope.parent_event_id.as_ref().map(EventId::as_str),
        Some("evt_6")
    );
}
```

- [ ] **Step 2: Run the new contracts test to verify it fails**

Run: `cargo test -p deer-foundation-contracts --test intent_and_replay_contracts -v`

Expected: FAIL with unresolved imports for `IntentAction`, `IntentCommand`, `ReplayEnvelope`, `WriteOperation`, or `WriteOperationKind`.

- [ ] **Step 3: Implement the shared command and replay DTOs**

```rust
// crates/foundation/contracts/src/commands.rs
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{RecordId, RecordRef, ThreadId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentAction {
    SubmitMessage,
    StopRun,
    ResumeClarification,
    OpenArtifact,
    ApplyPolicyOverlay,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IntentCommand {
    pub command_id: RecordId,
    pub action: IntentAction,
    pub target: Option<RecordRef>,
    pub thread_id: Option<ThreadId>,
    pub payload: Value,
}

impl IntentCommand {
    pub fn new(
        command_id: RecordId,
        action: IntentAction,
        target: Option<RecordRef>,
        thread_id: Option<ThreadId>,
        payload: Value,
    ) -> Self {
        Self {
            command_id,
            action,
            target,
            thread_id,
            payload,
        }
    }
}
```

```rust
// crates/foundation/contracts/src/events.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{EventId, RecordId, RecordRef};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WriteOperationKind {
    AppendRecord,
    AppendGovernanceRecord,
    EmitCheckpoint,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriteOperation {
    pub kind: WriteOperationKind,
    pub record_id: RecordId,
}

impl WriteOperation {
    pub fn new(kind: WriteOperationKind, record_id: RecordId) -> Self {
        Self { kind, record_id }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplayDirection {
    Forward,
    Reverse,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayCursor {
    pub sequence: u64,
    pub direction: ReplayDirection,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayEnvelope {
    pub sequence: u64,
    pub event_id: EventId,
    pub record_ref: RecordRef,
    pub write: WriteOperation,
    pub parent_event_id: Option<EventId>,
    pub occurred_at: DateTime<Utc>,
}

impl ReplayEnvelope {
    pub fn append(
        sequence: u64,
        event_id: EventId,
        record_ref: RecordRef,
        write: WriteOperation,
        parent_event_id: Option<EventId>,
        occurred_at: DateTime<Utc>,
    ) -> Self {
        Self {
            sequence,
            event_id,
            record_ref,
            write,
            parent_event_id,
            occurred_at,
        }
    }

    pub fn cursor(&self) -> ReplayCursor {
        ReplayCursor {
            sequence: self.sequence,
            direction: ReplayDirection::Forward,
        }
    }
}
```

- [ ] **Step 4: Run the contracts command/event test to verify it passes**

Run: `cargo test -p deer-foundation-contracts --test intent_and_replay_contracts -v`

Expected: PASS with both tests reporting `... ok`.

- [ ] **Step 5: Commit the command/event contracts**

```bash
git add crates/foundation/contracts/src/lib.rs crates/foundation/contracts/src/commands.rs crates/foundation/contracts/src/events.rs crates/foundation/contracts/tests/intent_and_replay_contracts.rs
git commit -m "feat: add foundation intent and replay contracts"
```

## Task 3: Define Canonical Domain Record Families

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/foundation/domain/Cargo.toml`
- Create: `crates/foundation/domain/src/lib.rs`
- Create: `crates/foundation/domain/src/common.rs`
- Create: `crates/foundation/domain/src/carrier.rs`
- Create: `crates/foundation/domain/src/semantic.rs`
- Create: `crates/foundation/domain/src/representation.rs`
- Create: `crates/foundation/domain/src/governance.rs`
- Create: `crates/foundation/domain/src/structural.rs`
- Test: `crates/foundation/domain/tests/domain_record_invariants.rs`

**Milestone unlock:** M0 foundation/domain canonical record spine

**Proof path later:** canonical records become the shared input to later normalizers, derivations, replay UI, and proof apps

**Forbidden dependencies:** raw backend schemas, Bevy/egui types, app-specific fantasy/world structs

- [ ] **Step 1: Write the failing domain invariants test and crate stubs**

```toml
# Cargo.toml
[workspace]
members = [
    "apps/deer_gui",
    "crates/foundation/contracts",
    "crates/foundation/domain",
]
resolver = "2"
```

```toml
# crates/foundation/domain/Cargo.toml
[package]
name = "deer-foundation-domain"
version = "0.1.0"
edition = "2021"

[dependencies]
chrono = { version = "0.4.44", features = ["clock", "serde"] }
deer-foundation-contracts = { path = "../contracts" }
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.149"
```

```rust
// crates/foundation/domain/src/lib.rs
pub mod carrier;
pub mod common;
pub mod governance;
pub mod representation;
pub mod semantic;
pub mod structural;
```

```rust
// crates/foundation/domain/tests/domain_record_invariants.rs
use deer_foundation_contracts::{
    AsIsHash, CanonicalLevel, CanonicalPlane, CanonicalRecord, ChunkHash,
    IdentityMeta, LineageMeta, RecordFamily, RecordId,
};
use deer_foundation_domain::{
    AnyRecord, EmbeddingBody, EmbeddingRecord, ExclusionBody, ExclusionRecord,
    RunBody, RunRecord,
};

#[test]
fn any_record_reports_expected_family_level_and_plane() {
    let run = RunRecord::new(
        RecordId::from_static("run_1"),
        IdentityMeta::hash_anchored(RecordId::from_static("run_1"), None, None, None),
        LineageMeta::root(),
        RunBody {
            title: "mission alpha".into(),
            status: "running".into(),
        },
    );

    let record = AnyRecord::Run(run);
    assert_eq!(record.header().family, RecordFamily::Run);
    assert_eq!(record.header().level, CanonicalLevel::L2);
    assert_eq!(record.header().plane, CanonicalPlane::AsIs);
}

#[test]
fn embedding_records_require_hash_anchored_identity() {
    let record = EmbeddingRecord::new(
        RecordId::from_static("embed_1"),
        IdentityMeta::hash_anchored(
            RecordId::from_static("embed_1"),
            Some(AsIsHash::from_static("sha256:artifact")),
            Some(ChunkHash::from_static("sha256:chunk")),
            Some(deer_foundation_contracts::EmbeddingBasisHash::from_static(
                "sha256:basis",
            )),
        ),
        LineageMeta::root(),
        EmbeddingBody {
            model: "text-embedding-3-large".into(),
            dimensions: 3072,
        },
    );

    assert_eq!(record.header.identity.as_is_hash.unwrap().as_str(), "sha256:artifact");
    assert_eq!(record.header.identity.chunk_hash.unwrap().as_str(), "sha256:chunk");
}

#[test]
fn exclusion_records_preserve_append_only_lineage() {
    let exclusion = ExclusionRecord::new(
        RecordId::from_static("exclusion_1"),
        IdentityMeta::hash_anchored(RecordId::from_static("exclusion_1"), None, None, None),
        LineageMeta::derived_from(deer_foundation_contracts::RecordRef::new(
            RecordFamily::Artifact,
            RecordId::from_static("artifact_9"),
        )),
        ExclusionBody {
            reason: "policy hidden".into(),
        },
    );

    assert_eq!(exclusion.header.lineage.derived_from.len(), 1);
    assert_eq!(
        exclusion.header.lineage.derived_from[0].record_id.as_str(),
        "artifact_9"
    );
}
```

- [ ] **Step 2: Run the domain invariants test to verify it fails**

Run: `cargo test -p deer-foundation-domain --test domain_record_invariants -v`

Expected: FAIL with unresolved imports for `AnyRecord`, `RunRecord`, `EmbeddingRecord`, or the grouped domain modules.

- [ ] **Step 3: Implement the grouped canonical domain record families**

```rust
// crates/foundation/domain/src/common.rs
use deer_foundation_contracts::{
    CanonicalLevel, CanonicalPlane, CanonicalRecord, CorrelationMeta, IdentityMeta,
    LineageMeta, RecordFamily, RecordHeader, RecordId, StorageDisposition,
};

pub fn build_header(
    record_id: RecordId,
    family: RecordFamily,
    level: CanonicalLevel,
    plane: CanonicalPlane,
    storage: StorageDisposition,
    identity: IdentityMeta,
    lineage: LineageMeta,
) -> RecordHeader {
    RecordHeader::new(
        record_id,
        family,
        level,
        plane,
        storage,
        identity,
        CorrelationMeta::default(),
        lineage,
    )
}

#[macro_export]
macro_rules! define_record {
    ($name:ident, $body:ty, $family:expr, $level:expr, $plane:expr, $storage:expr) => {
        #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        pub struct $name {
            pub header: deer_foundation_contracts::RecordHeader,
            pub body: $body,
        }

        impl $name {
            pub fn new(
                record_id: deer_foundation_contracts::RecordId,
                identity: deer_foundation_contracts::IdentityMeta,
                lineage: deer_foundation_contracts::LineageMeta,
                body: $body,
            ) -> Self {
                Self {
                    header: crate::common::build_header(
                        record_id,
                        $family,
                        $level,
                        $plane,
                        $storage,
                        identity,
                        lineage,
                    ),
                    body,
                }
            }
        }

        impl deer_foundation_contracts::CanonicalRecord for $name {
            fn header(&self) -> &deer_foundation_contracts::RecordHeader {
                &self.header
            }
        }
    };
}
```

```rust
// crates/foundation/domain/src/carrier.rs
use deer_foundation_contracts::{CanonicalLevel, CanonicalPlane, RecordFamily, StorageDisposition};
use serde::{Deserialize, Serialize};

use crate::define_record;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunBody {
    pub title: String,
    pub status: String,
}
define_record!(RunRecord, RunBody, RecordFamily::Run, CanonicalLevel::L2, CanonicalPlane::AsIs, StorageDisposition::StorageNative);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionBody {
    pub name: String,
}
define_record!(SessionRecord, SessionBody, RecordFamily::Session, CanonicalLevel::L2, CanonicalPlane::AsIs, StorageDisposition::StorageNative);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskBody {
    pub label: String,
    pub status: String,
}
define_record!(TaskRecord, TaskBody, RecordFamily::Task, CanonicalLevel::L2, CanonicalPlane::AsIs, StorageDisposition::StorageNative);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageBody {
    pub role: String,
    pub text: String,
}
define_record!(MessageRecord, MessageBody, RecordFamily::Message, CanonicalLevel::L2, CanonicalPlane::AsIs, StorageDisposition::StorageNative);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCallBody {
    pub tool_name: String,
    pub status: String,
}
define_record!(ToolCallRecord, ToolCallBody, RecordFamily::ToolCall, CanonicalLevel::L2, CanonicalPlane::AsIs, StorageDisposition::StorageNative);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactBody {
    pub label: String,
    pub media_type: String,
}
define_record!(ArtifactRecord, ArtifactBody, RecordFamily::Artifact, CanonicalLevel::L2, CanonicalPlane::AsIs, StorageDisposition::StorageNative);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClarificationBody {
    pub prompt: String,
    pub resolved: bool,
}
define_record!(ClarificationRecord, ClarificationBody, RecordFamily::Clarification, CanonicalLevel::L2, CanonicalPlane::AsIs, StorageDisposition::StorageNative);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeStatusBody {
    pub status: String,
    pub detail: String,
}
define_record!(RuntimeStatusRecord, RuntimeStatusBody, RecordFamily::RuntimeStatus, CanonicalLevel::L2, CanonicalPlane::AsIs, StorageDisposition::StorageNative);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeliveryBody {
    pub channel: String,
    pub delivered: bool,
}
define_record!(DeliveryRecord, DeliveryBody, RecordFamily::Delivery, CanonicalLevel::L2, CanonicalPlane::AsIs, StorageDisposition::StorageNative);
```

```rust
// crates/foundation/domain/src/semantic.rs
use deer_foundation_contracts::{CanonicalLevel, CanonicalPlane, RecordFamily, StorageDisposition};
use serde::{Deserialize, Serialize};

use crate::define_record;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct L0SourceBody { pub summary: String }
define_record!(L0SourceRecord, L0SourceBody, RecordFamily::L0Source, CanonicalLevel::L0, CanonicalPlane::AsIs, StorageDisposition::StorageNative);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct L1SanitizedBody { pub summary: String }
define_record!(L1SanitizedRecord, L1SanitizedBody, RecordFamily::L1Sanitized, CanonicalLevel::L1, CanonicalPlane::AsIs, StorageDisposition::StorageNative);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct L2ViewBody { pub summary: String }
define_record!(L2ViewRecord, L2ViewBody, RecordFamily::L2View, CanonicalLevel::L2, CanonicalPlane::AsIs, StorageDisposition::StorageNative);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct L3InsightBody { pub summary: String }
define_record!(L3InsightRecord, L3InsightBody, RecordFamily::L3Insight, CanonicalLevel::L3, CanonicalPlane::AsIs, StorageDisposition::StorageNative);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct L4PredictionBody { pub summary: String }
define_record!(L4PredictionRecord, L4PredictionBody, RecordFamily::L4Prediction, CanonicalLevel::L4, CanonicalPlane::AsIs, StorageDisposition::StorageNative);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct L5PrescriptionBody { pub summary: String }
define_record!(L5PrescriptionRecord, L5PrescriptionBody, RecordFamily::L5Prescription, CanonicalLevel::L5, CanonicalPlane::AsIs, StorageDisposition::StorageNative);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct L6OutcomeBody { pub summary: String }
define_record!(L6OutcomeRecord, L6OutcomeBody, RecordFamily::L6Outcome, CanonicalLevel::L6, CanonicalPlane::AsIs, StorageDisposition::StorageNative);
```

```rust
// crates/foundation/domain/src/representation.rs
use deer_foundation_contracts::{CanonicalLevel, CanonicalPlane, RecordFamily, StorageDisposition};
use serde::{Deserialize, Serialize};

use crate::define_record;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AsIsRepresentationBody {
    pub media_type: String,
}
define_record!(AsIsRepresentationRecord, AsIsRepresentationBody, RecordFamily::AsIsRepresentation, CanonicalLevel::L2, CanonicalPlane::AsIs, StorageDisposition::StorageNative);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkBody {
    pub chunk_index: u32,
    pub text: String,
}
define_record!(ChunkRecord, ChunkBody, RecordFamily::Chunk, CanonicalLevel::L2, CanonicalPlane::Chunks, StorageDisposition::StorageNative);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddingBody {
    pub model: String,
    pub dimensions: usize,
}
define_record!(EmbeddingRecord, EmbeddingBody, RecordFamily::Embedding, CanonicalLevel::L2, CanonicalPlane::Embeddings, StorageDisposition::IndexProjection);
```

```rust
// crates/foundation/domain/src/governance.rs
use deer_foundation_contracts::{CanonicalLevel, CanonicalPlane, RecordFamily, StorageDisposition};
use serde::{Deserialize, Serialize};

use crate::define_record;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentBody { pub action: String }
define_record!(IntentRecord, IntentBody, RecordFamily::Intent, CanonicalLevel::L2, CanonicalPlane::AsIs, StorageDisposition::StorageNative);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransformBody { pub producer: String }
define_record!(TransformRecord, TransformBody, RecordFamily::Transform, CanonicalLevel::L2, CanonicalPlane::AsIs, StorageDisposition::StorageNative);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExclusionBody { pub reason: String }
define_record!(ExclusionRecord, ExclusionBody, RecordFamily::Exclusion, CanonicalLevel::L2, CanonicalPlane::AsIs, StorageDisposition::StorageNative);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictBody { pub reason: String }
define_record!(ConflictRecord, ConflictBody, RecordFamily::Conflict, CanonicalLevel::L2, CanonicalPlane::AsIs, StorageDisposition::StorageNative);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolutionBody { pub strategy: String }
define_record!(ResolutionRecord, ResolutionBody, RecordFamily::Resolution, CanonicalLevel::L2, CanonicalPlane::AsIs, StorageDisposition::StorageNative);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayCheckpointBody { pub label: String }
define_record!(ReplayCheckpointRecord, ReplayCheckpointBody, RecordFamily::ReplayCheckpoint, CanonicalLevel::L2, CanonicalPlane::AsIs, StorageDisposition::StorageNative);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DedupBody { pub dedup_key: String }
define_record!(DedupRecord, DedupBody, RecordFamily::Dedup, CanonicalLevel::L2, CanonicalPlane::AsIs, StorageDisposition::StorageNative);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchBody { pub batch_key: String }
define_record!(BatchRecord, BatchBody, RecordFamily::Batch, CanonicalLevel::L2, CanonicalPlane::AsIs, StorageDisposition::StorageNative);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchBody { pub branch_name: String }
define_record!(BranchRecord, BranchBody, RecordFamily::Branch, CanonicalLevel::L2, CanonicalPlane::AsIs, StorageDisposition::StorageNative);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionBody { pub version: String }
define_record!(VersionRecord, VersionBody, RecordFamily::Version, CanonicalLevel::L2, CanonicalPlane::AsIs, StorageDisposition::StorageNative);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriteOperationBody { pub op: String }
define_record!(WriteOperationRecord, WriteOperationBody, RecordFamily::WriteOperation, CanonicalLevel::L2, CanonicalPlane::AsIs, StorageDisposition::StorageNative);
```

```rust
// crates/foundation/domain/src/structural.rs
use deer_foundation_contracts::{CanonicalLevel, CanonicalPlane, RecordFamily, StorageDisposition};
use serde::{Deserialize, Serialize};

use crate::define_record;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNodeBody { pub label: String }
define_record!(GraphNodeRecord, GraphNodeBody, RecordFamily::GraphNode, CanonicalLevel::L3, CanonicalPlane::AsIs, StorageDisposition::StorageNative);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphEdgeBody { pub label: String }
define_record!(GraphEdgeRecord, GraphEdgeBody, RecordFamily::GraphEdge, CanonicalLevel::L3, CanonicalPlane::AsIs, StorageDisposition::StorageNative);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeEntityBody { pub name: String }
define_record!(KnowledgeEntityRecord, KnowledgeEntityBody, RecordFamily::KnowledgeEntity, CanonicalLevel::L3, CanonicalPlane::AsIs, StorageDisposition::StorageNative);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeRelationBody { pub relation: String }
define_record!(KnowledgeRelationRecord, KnowledgeRelationBody, RecordFamily::KnowledgeRelation, CanonicalLevel::L3, CanonicalPlane::AsIs, StorageDisposition::StorageNative);
```

```rust
// crates/foundation/domain/src/lib.rs
pub mod carrier;
pub mod common;
pub mod governance;
pub mod representation;
pub mod semantic;
pub mod structural;

pub use carrier::*;
pub use governance::*;
pub use representation::*;
pub use semantic::*;
pub use structural::*;

use deer_foundation_contracts::{CanonicalRecord, RecordHeader};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnyRecord {
    Run(RunRecord),
    Session(SessionRecord),
    Task(TaskRecord),
    Message(MessageRecord),
    ToolCall(ToolCallRecord),
    Artifact(ArtifactRecord),
    Clarification(ClarificationRecord),
    RuntimeStatus(RuntimeStatusRecord),
    Delivery(DeliveryRecord),
    L0Source(L0SourceRecord),
    L1Sanitized(L1SanitizedRecord),
    L2View(L2ViewRecord),
    L3Insight(L3InsightRecord),
    L4Prediction(L4PredictionRecord),
    L5Prescription(L5PrescriptionRecord),
    L6Outcome(L6OutcomeRecord),
    AsIsRepresentation(AsIsRepresentationRecord),
    Chunk(ChunkRecord),
    Embedding(EmbeddingRecord),
    Intent(IntentRecord),
    Transform(TransformRecord),
    Exclusion(ExclusionRecord),
    Conflict(ConflictRecord),
    Resolution(ResolutionRecord),
    ReplayCheckpoint(ReplayCheckpointRecord),
    Dedup(DedupRecord),
    Batch(BatchRecord),
    Branch(BranchRecord),
    Version(VersionRecord),
    WriteOperation(WriteOperationRecord),
    GraphNode(GraphNodeRecord),
    GraphEdge(GraphEdgeRecord),
    KnowledgeEntity(KnowledgeEntityRecord),
    KnowledgeRelation(KnowledgeRelationRecord),
}

impl CanonicalRecord for AnyRecord {
    fn header(&self) -> &RecordHeader {
        match self {
            AnyRecord::Run(record) => &record.header,
            AnyRecord::Session(record) => &record.header,
            AnyRecord::Task(record) => &record.header,
            AnyRecord::Message(record) => &record.header,
            AnyRecord::ToolCall(record) => &record.header,
            AnyRecord::Artifact(record) => &record.header,
            AnyRecord::Clarification(record) => &record.header,
            AnyRecord::RuntimeStatus(record) => &record.header,
            AnyRecord::Delivery(record) => &record.header,
            AnyRecord::L0Source(record) => &record.header,
            AnyRecord::L1Sanitized(record) => &record.header,
            AnyRecord::L2View(record) => &record.header,
            AnyRecord::L3Insight(record) => &record.header,
            AnyRecord::L4Prediction(record) => &record.header,
            AnyRecord::L5Prescription(record) => &record.header,
            AnyRecord::L6Outcome(record) => &record.header,
            AnyRecord::AsIsRepresentation(record) => &record.header,
            AnyRecord::Chunk(record) => &record.header,
            AnyRecord::Embedding(record) => &record.header,
            AnyRecord::Intent(record) => &record.header,
            AnyRecord::Transform(record) => &record.header,
            AnyRecord::Exclusion(record) => &record.header,
            AnyRecord::Conflict(record) => &record.header,
            AnyRecord::Resolution(record) => &record.header,
            AnyRecord::ReplayCheckpoint(record) => &record.header,
            AnyRecord::Dedup(record) => &record.header,
            AnyRecord::Batch(record) => &record.header,
            AnyRecord::Branch(record) => &record.header,
            AnyRecord::Version(record) => &record.header,
            AnyRecord::WriteOperation(record) => &record.header,
            AnyRecord::GraphNode(record) => &record.header,
            AnyRecord::GraphEdge(record) => &record.header,
            AnyRecord::KnowledgeEntity(record) => &record.header,
            AnyRecord::KnowledgeRelation(record) => &record.header,
        }
    }
}
```

- [ ] **Step 4: Run the domain invariants test to verify it passes**

Run: `cargo test -p deer-foundation-domain --test domain_record_invariants -v`

Expected: PASS with all three invariants tests reporting `... ok`.

- [ ] **Step 5: Commit the canonical domain spine**

```bash
git add crates/foundation/domain/Cargo.toml crates/foundation/domain/src/lib.rs crates/foundation/domain/src/common.rs crates/foundation/domain/src/carrier.rs crates/foundation/domain/src/semantic.rs crates/foundation/domain/src/representation.rs crates/foundation/domain/src/governance.rs crates/foundation/domain/src/structural.rs crates/foundation/domain/tests/domain_record_invariants.rs
git commit -m "feat: add canonical foundation domain records"
```

## Task 4: Add Replay Log, Fixture Loading, And Cross-Crate History Tests

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/foundation/replay/Cargo.toml`
- Create: `crates/foundation/replay/src/lib.rs`
- Create: `crates/foundation/replay/src/error.rs`
- Create: `crates/foundation/replay/src/event_log.rs`
- Create: `crates/foundation/replay/src/fixtures.rs`
- Test: `crates/foundation/replay/tests/replay_flow.rs`
- Test Fixture: `crates/foundation/replay/tests/fixtures/foundation_spine_log.json`

**Milestone unlock:** M0 foundation/replay append-only history and fixture proof

**Proof path later:** later replay/timeline proof in `apps/deer_replay`

**Forbidden dependencies:** Bevy timeline widgets, egui state, raw generator stream schemas

- [ ] **Step 1: Write the failing replay tests, fixture, and crate stubs**

```toml
# Cargo.toml
[workspace]
members = [
    "apps/deer_gui",
    "crates/foundation/contracts",
    "crates/foundation/domain",
    "crates/foundation/replay",
]
resolver = "2"
```

```toml
# crates/foundation/replay/Cargo.toml
[package]
name = "deer-foundation-replay"
version = "0.1.0"
edition = "2021"

[dependencies]
chrono = { version = "0.4.44", features = ["clock", "serde"] }
deer-foundation-contracts = { path = "../contracts" }
deer-foundation-domain = { path = "../domain" }
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.149"
thiserror = "2"
```

```rust
// crates/foundation/replay/src/lib.rs
pub mod error;
pub mod event_log;
pub mod fixtures;

pub use error::*;
pub use event_log::*;
pub use fixtures::*;
```

```json
// crates/foundation/replay/tests/fixtures/foundation_spine_log.json
{
  "entries": [
    {
      "envelope": {
        "sequence": 1,
        "event_id": "evt_1",
        "record_ref": { "family": "Run", "record_id": "run_1" },
        "write": { "kind": "AppendRecord", "record_id": "run_1" },
        "parent_event_id": null,
        "occurred_at": "2026-04-02T00:00:00Z"
      },
      "record": {
        "Run": {
          "header": {
            "record_id": "run_1",
            "family": "Run",
            "level": "L2",
            "plane": "AsIs",
            "storage": "StorageNative",
            "identity": {
              "primary_id": "run_1",
              "as_is_hash": null,
              "chunk_hash": null,
              "embedding_basis_hash": null
            },
            "correlations": {
              "mission_id": null,
              "run_id": null,
              "artifact_id": null,
              "trace_id": null,
              "thread_id": null,
              "task_id": null
            },
            "lineage": {
              "derived_from": [],
              "source_events": [],
              "supersedes": null
            },
            "observed_at": "2026-04-02T00:00:00Z"
          },
          "body": {
            "title": "mission alpha",
            "status": "running"
          }
        }
      }
    },
    {
      "envelope": {
        "sequence": 2,
        "event_id": "evt_2",
        "record_ref": { "family": "Artifact", "record_id": "artifact_1" },
        "write": { "kind": "AppendRecord", "record_id": "artifact_1" },
        "parent_event_id": "evt_1",
        "occurred_at": "2026-04-02T00:01:00Z"
      },
      "record": {
        "Artifact": {
          "header": {
            "record_id": "artifact_1",
            "family": "Artifact",
            "level": "L2",
            "plane": "AsIs",
            "storage": "StorageNative",
            "identity": {
              "primary_id": "artifact_1",
              "as_is_hash": "sha256:artifact",
              "chunk_hash": null,
              "embedding_basis_hash": null
            },
            "correlations": {
              "mission_id": null,
              "run_id": null,
              "artifact_id": null,
              "trace_id": null,
              "thread_id": null,
              "task_id": null
            },
            "lineage": {
              "derived_from": [
                { "family": "Run", "record_id": "run_1" }
              ],
              "source_events": [],
              "supersedes": null
            },
            "observed_at": "2026-04-02T00:01:00Z"
          },
          "body": {
            "label": "sector report",
            "media_type": "text/markdown"
          }
        }
      }
    }
  ]
}
```

```rust
// crates/foundation/replay/tests/replay_flow.rs
use std::path::PathBuf;

use deer_foundation_contracts::{
    EventId, RecordFamily, RecordId, RecordRef, ReplayEnvelope, WriteOperation,
    WriteOperationKind,
};
use deer_foundation_domain::{AnyRecord, RunBody, RunRecord};
use deer_foundation_replay::{ReplayFixture, ReplayLog};

#[test]
fn fixture_log_replays_in_strict_sequence() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/foundation_spine_log.json");
    let fixture = ReplayFixture::load(&fixture_path).unwrap();
    let log = fixture.into_log().unwrap();

    assert_eq!(log.entries().len(), 2);
    assert_eq!(log.after(Some(&log.entries()[0].envelope.cursor())).len(), 1);
}

#[test]
fn append_rejects_non_monotonic_sequence() {
    let mut log = ReplayLog::default();

    let first_record = AnyRecord::Run(RunRecord::new(
        RecordId::from_static("run_1"),
        deer_foundation_contracts::IdentityMeta::hash_anchored(
            RecordId::from_static("run_1"),
            None,
            None,
            None,
        ),
        deer_foundation_contracts::LineageMeta::root(),
        RunBody {
            title: "mission alpha".into(),
            status: "running".into(),
        },
    ));

    log.append(
        ReplayEnvelope::append(
            2,
            EventId::from_static("evt_2"),
            RecordRef::new(RecordFamily::Run, RecordId::from_static("run_1")),
            WriteOperation::new(WriteOperationKind::AppendRecord, RecordId::from_static("run_1")),
            None,
            chrono::Utc::now(),
        ),
        first_record.clone(),
    )
    .unwrap();

    let error = log
        .append(
            ReplayEnvelope::append(
                1,
                EventId::from_static("evt_1"),
                RecordRef::new(RecordFamily::Run, RecordId::from_static("run_1")),
                WriteOperation::new(WriteOperationKind::AppendRecord, RecordId::from_static("run_1")),
                None,
                chrono::Utc::now(),
            ),
            first_record,
        )
        .unwrap_err();

    assert!(error.to_string().contains("strictly increasing"));
}
```

- [ ] **Step 2: Run the replay tests to verify they fail**

Run: `cargo test -p deer-foundation-replay --test replay_flow -v`

Expected: FAIL with unresolved import or missing type errors for `ReplayFixture`, `ReplayLog`, or the replay modules.

- [ ] **Step 3: Implement append-only replay history and fixture loading**

```rust
// crates/foundation/replay/src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReplayError {
    #[error("event sequence must be strictly increasing: last={last}, next={next}")]
    NonMonotonicSequence { last: u64, next: u64 },

    #[error("failed to read replay fixture {path}: {source}")]
    FixtureRead {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse replay fixture {path}: {source}")]
    FixtureParse {
        path: String,
        #[source]
        source: serde_json::Error,
    },
}
```

```rust
// crates/foundation/replay/src/event_log.rs
use deer_foundation_contracts::{RecordId, ReplayCursor, ReplayEnvelope};
use deer_foundation_domain::AnyRecord;

use crate::ReplayError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayEntry {
    pub envelope: ReplayEnvelope,
    pub record: AnyRecord,
}

#[derive(Clone, Debug, Default)]
pub struct ReplayLog {
    entries: Vec<ReplayEntry>,
}

impl ReplayLog {
    pub fn append(&mut self, envelope: ReplayEnvelope, record: AnyRecord) -> Result<(), ReplayError> {
        if let Some(last) = self.entries.last().map(|entry| entry.envelope.sequence) {
            if envelope.sequence <= last {
                return Err(ReplayError::NonMonotonicSequence {
                    last,
                    next: envelope.sequence,
                });
            }
        }

        self.entries.push(ReplayEntry { envelope, record });
        Ok(())
    }

    pub fn entries(&self) -> &[ReplayEntry] {
        &self.entries
    }

    pub fn after(&self, cursor: Option<&ReplayCursor>) -> Vec<&ReplayEntry> {
        match cursor {
            Some(cursor) => self
                .entries
                .iter()
                .filter(|entry| entry.envelope.sequence > cursor.sequence)
                .collect(),
            None => self.entries.iter().collect(),
        }
    }

    pub fn latest_for(&self, record_id: &RecordId) -> Option<&ReplayEntry> {
        self.entries
            .iter()
            .rev()
            .find(|entry| &entry.envelope.record_ref.record_id == record_id)
    }
}
```

```rust
// crates/foundation/replay/src/fixtures.rs
use std::path::Path;

use deer_foundation_contracts::ReplayEnvelope;
use deer_foundation_domain::AnyRecord;
use serde::{Deserialize, Serialize};

use crate::{ReplayError, ReplayLog};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayFixtureEntry {
    pub envelope: ReplayEnvelope,
    pub record: AnyRecord,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayFixture {
    pub entries: Vec<ReplayFixtureEntry>,
}

impl ReplayFixture {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ReplayError> {
        let path = path.as_ref();
        let raw = std::fs::read_to_string(path).map_err(|source| ReplayError::FixtureRead {
            path: path.display().to_string(),
            source,
        })?;

        serde_json::from_str(&raw).map_err(|source| ReplayError::FixtureParse {
            path: path.display().to_string(),
            source,
        })
    }

    pub fn into_log(self) -> Result<ReplayLog, ReplayError> {
        let mut log = ReplayLog::default();

        for entry in self.entries {
            log.append(entry.envelope, entry.record)?;
        }

        Ok(log)
    }
}
```

```rust
// crates/foundation/replay/src/lib.rs
pub mod error;
pub mod event_log;
pub mod fixtures;

pub use error::*;
pub use event_log::*;
pub use fixtures::*;
```

- [ ] **Step 4: Run the replay test and then the full foundation-spine test sweep**

Run: `cargo test -p deer-foundation-replay --test replay_flow -v`

Expected: PASS with `fixture_log_replays_in_strict_sequence ... ok` and `append_rejects_non_monotonic_sequence ... ok`.

Run: `cargo test -p deer-foundation-contracts -p deer-foundation-domain -p deer-foundation-replay -v`

Expected: PASS with all foundation spine tests green.

- [ ] **Step 5: Commit the replay spine and full M0 foundation slice**

```bash
git add crates/foundation/replay/Cargo.toml crates/foundation/replay/src/lib.rs crates/foundation/replay/src/error.rs crates/foundation/replay/src/event_log.rs crates/foundation/replay/src/fixtures.rs crates/foundation/replay/tests/replay_flow.rs crates/foundation/replay/tests/fixtures/foundation_spine_log.json
git commit -m "feat: add append-only foundation replay spine"
```

## Verification Checklist

- [ ] `cargo test -p deer-foundation-contracts -v`
- [ ] `cargo test -p deer-foundation-domain -v`
- [ ] `cargo test -p deer-foundation-replay -v`
- [ ] `cargo test -p deer-foundation-contracts -p deer-foundation-domain -p deer-foundation-replay -v`
- [ ] confirm no crate in this plan imports `bevy`, `bevy_egui`, or `apps/deer_gui`
- [ ] confirm the workspace still contains no proof-app crates after this plan; proof apps belong to later plans

## Handoff Notes

- This plan intentionally stops at canonical contracts/domain/replay.
- The immediate follow-on plan is `docs/superpowers/plans/2026-04-01-normalization-derivation-read-model-spine.md`.
- Do not start `apps/deer_chat_lab`, `apps/deer_replay`, `apps/deer_graph_lab`, `apps/deer_design`, or `apps/deer_gui` composition work while executing this plan.
