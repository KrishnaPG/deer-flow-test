# Shared Pipeline And DeerFlow Onboarding Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend the existing reusable pipeline so DeerFlow can flow from generator capture through storage-aware raw envelopes, state-server-mediated reads, lineage-safe canonical records, and L2 game-facing projections that later generators can follow.

**Architecture:** Reuse the current foundation, raw-source, normalizer, derivation, and world-projection crates instead of inventing app-local glue. Start by hardening foundation metadata for source level/plane and correlation needs, then add a DeerFlow-specific raw/mediated adapter, normalize that into L0/L1/L2 canonical records, and finish with reusable game-facing derivations plus world objects backed by lineage and backlinks.

**Tech Stack:** Rust 2021 workspace crates, `serde`, `serde_json`, `chrono`, `thiserror`, `insta`, existing DeerFlow fixture/bridge shapes as reference only

---

## Scope Boundary

- This plan covers the shared reusable spine plus DeerFlow onboarding only.
- It modifies existing crates: `foundation/contracts`, `foundation/domain`, `pipeline/raw_sources`, `pipeline/normalizers`, `pipeline/derivations`, and `runtime/world_projection`.
- It does **not** modify `apps/deer_gui` or any proof app.
- It does **not** onboard Hermes, Rowboat, or PocketFlow yet; the resulting pipeline must make those follow-on adapters straightforward.
- It does **not** implement a production state server. It introduces state-server-aligned mediated read DTOs and fixture-backed DeerFlow adapters that preserve the boundary.
- It does **not** allow direct backend payloads, direct artifact paths, or app-local mutable truth to bypass storage/lineage semantics.

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

## Task 1: Harden Foundation Metadata For Source-Level Generator Onboarding

**Files:**
- Modify: `crates/foundation/contracts/src/meta.rs`
- Modify: `crates/foundation/contracts/src/records.rs`
- Modify: `crates/foundation/domain/src/common.rs`
- Test: `crates/foundation/contracts/tests/source_metadata_contracts.rs`
- Test: `crates/foundation/domain/tests/source_aware_record_builders.rs`

**Milestone unlock:** foundation records can express source level/plane, storage flags, and agent-aware correlations required by state-server-aligned generator pipelines

**Forbidden shortcuts:** do not break existing `new(...)` constructors; do not push source metadata down into app code; do not encode level/plane only in ad hoc strings

- [ ] **Step 1: Write the failing contract tests**

```rust
// crates/foundation/contracts/tests/source_metadata_contracts.rs
use deer_foundation_contracts::{
    AgentId, CanonicalLevel, CanonicalPlane, CorrelationMeta, IdentityMeta, LineageMeta,
    RecordFamily, RecordHeader, RecordId, StorageDisposition,
};

#[test]
fn record_header_tracks_source_level_plane_and_storage_flags() {
    let header = RecordHeader::new(
        RecordId::from_static("rec_1"),
        RecordFamily::L2View,
        CanonicalLevel::L2,
        CanonicalPlane::AsIs,
        StorageDisposition::StorageNative,
        IdentityMeta::hash_anchored(RecordId::from_static("rec_1"), None, None, None),
        CorrelationMeta {
            agent_id: Some(AgentId::from_static("agent_1")),
            ..Default::default()
        },
        LineageMeta::root(),
    );

    assert_eq!(header.source_level, CanonicalLevel::L2);
    assert_eq!(header.source_plane, CanonicalPlane::AsIs);
    assert!(header.is_persisted_truth);
    assert!(!header.is_index_projection);
    assert!(!header.is_client_transient);
    assert_eq!(header.correlations.agent_id.unwrap().as_str(), "agent_1");
}
```

```rust
// crates/foundation/domain/tests/source_aware_record_builders.rs
use deer_foundation_contracts::{
    AgentId, CanonicalLevel, CanonicalPlane, CorrelationMeta, IdentityMeta, LineageMeta,
    RecordId,
};
use deer_foundation_domain::{MessageBody, MessageRecord};

#[test]
fn source_aware_record_builder_preserves_correlation_and_source_dimensions() {
    let record = MessageRecord::new_with_meta(
        RecordId::from_static("msg_1"),
        IdentityMeta::hash_anchored(RecordId::from_static("msg_1"), None, None, None),
        CorrelationMeta {
            agent_id: Some(AgentId::from_static("agent_1")),
            ..Default::default()
        },
        LineageMeta::root(),
        CanonicalLevel::L0,
        CanonicalPlane::AsIs,
        MessageBody {
            role: "assistant".into(),
            text: "Scouting ridge".into(),
        },
    );

    assert_eq!(record.header.level, CanonicalLevel::L2);
    assert_eq!(record.header.source_level, CanonicalLevel::L0);
    assert_eq!(record.header.source_plane, CanonicalPlane::AsIs);
    assert_eq!(
        record.header.correlations.agent_id.as_ref().map(|id| id.as_str()),
        Some("agent_1")
    );
}
```

- [ ] **Step 2: Run the targeted foundation tests and confirm they fail**

Run: `cargo test -p deer-foundation-contracts --test source_metadata_contracts -v && cargo test -p deer-foundation-domain --test source_aware_record_builders -v`

Expected: FAIL with missing `agent_id`, `source_level`, `source_plane`, storage flag fields, and missing `new_with_meta`.

- [ ] **Step 3: Implement the minimal contract and domain metadata extensions**

```rust
// crates/foundation/contracts/src/meta.rs
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorrelationMeta {
    pub mission_id: Option<MissionId>,
    pub run_id: Option<RunId>,
    pub artifact_id: Option<ArtifactId>,
    pub trace_id: Option<TraceId>,
    pub thread_id: Option<ThreadId>,
    pub task_id: Option<TaskId>,
    pub agent_id: Option<AgentId>,
}
```

```rust
// crates/foundation/contracts/src/records.rs
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordHeader {
    pub record_id: RecordId,
    pub family: RecordFamily,
    pub level: CanonicalLevel,
    pub plane: CanonicalPlane,
    pub source_level: CanonicalLevel,
    pub source_plane: CanonicalPlane,
    pub storage: StorageDisposition,
    pub is_persisted_truth: bool,
    pub is_index_projection: bool,
    pub is_client_transient: bool,
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
        Self::new_with_source(
            record_id,
            family,
            level,
            plane,
            level,
            plane,
            storage,
            identity,
            correlations,
            lineage,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_source(
        record_id: RecordId,
        family: RecordFamily,
        level: CanonicalLevel,
        plane: CanonicalPlane,
        source_level: CanonicalLevel,
        source_plane: CanonicalPlane,
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
            source_level,
            source_plane,
            is_persisted_truth: matches!(storage, StorageDisposition::StorageNative),
            is_index_projection: matches!(storage, StorageDisposition::IndexProjection),
            is_client_transient: matches!(storage, StorageDisposition::ClientTransient),
            storage,
            identity,
            correlations,
            lineage,
            observed_at: Utc::now(),
        }
    }
}
```

```rust
// crates/foundation/domain/src/common.rs
pub fn build_header(
    record_id: RecordId,
    family: RecordFamily,
    level: CanonicalLevel,
    plane: CanonicalPlane,
    storage: StorageDisposition,
    identity: IdentityMeta,
    lineage: LineageMeta,
) -> RecordHeader {
    build_header_with_meta(
        record_id,
        family,
        level,
        plane,
        level,
        plane,
        storage,
        identity,
        CorrelationMeta::default(),
        lineage,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn build_header_with_meta(
    record_id: RecordId,
    family: RecordFamily,
    level: CanonicalLevel,
    plane: CanonicalPlane,
    source_level: CanonicalLevel,
    source_plane: CanonicalPlane,
    storage: StorageDisposition,
    identity: IdentityMeta,
    correlations: CorrelationMeta,
    lineage: LineageMeta,
) -> RecordHeader {
    RecordHeader::new_with_source(
        record_id,
        family,
        level,
        plane,
        source_level,
        source_plane,
        storage,
        identity,
        correlations,
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
                        record_id, $family, $level, $plane, $storage, identity, lineage,
                    ),
                    body,
                }
            }

            pub fn new_with_meta(
                record_id: deer_foundation_contracts::RecordId,
                identity: deer_foundation_contracts::IdentityMeta,
                correlations: deer_foundation_contracts::CorrelationMeta,
                lineage: deer_foundation_contracts::LineageMeta,
                source_level: deer_foundation_contracts::CanonicalLevel,
                source_plane: deer_foundation_contracts::CanonicalPlane,
                body: $body,
            ) -> Self {
                Self {
                    header: crate::common::build_header_with_meta(
                        record_id,
                        $family,
                        $level,
                        $plane,
                        source_level,
                        source_plane,
                        $storage,
                        identity,
                        correlations,
                        lineage,
                    ),
                    body,
                }
            }
        }
    };
}
```

- [ ] **Step 4: Re-run the targeted foundation tests**

Run: `cargo test -p deer-foundation-contracts --test source_metadata_contracts -v && cargo test -p deer-foundation-domain --test source_aware_record_builders -v`

Expected: PASS with source-level/source-plane metadata and agent-aware correlations preserved.

- [ ] **Step 5: Commit the foundation metadata hardening**

```bash
git add crates/foundation/contracts/src/meta.rs crates/foundation/contracts/src/records.rs crates/foundation/contracts/tests/source_metadata_contracts.rs crates/foundation/domain/src/common.rs crates/foundation/domain/tests/source_aware_record_builders.rs
git commit -m "feat: add source-aware record metadata"
```

## Task 2: Add DeerFlow Raw Envelope And Mediated-Read Surfaces

**Files:**
- Modify: `crates/pipeline/raw_sources/Cargo.toml`
- Create: `crates/pipeline/raw_sources/src/envelopes.rs`
- Create: `crates/pipeline/raw_sources/src/deerflow.rs`
- Create: `crates/pipeline/raw_sources/src/mediated_reads.rs`
- Modify: `crates/pipeline/raw_sources/src/lib.rs`
- Test: `crates/pipeline/raw_sources/tests/deerflow_adapter.rs`
- Test fixture: `crates/pipeline/raw_sources/tests/fixtures/deerflow_thread_snapshot.json`
- Test fixture: `crates/pipeline/raw_sources/tests/fixtures/deerflow_live_activity.json`
- Test fixture: `crates/pipeline/raw_sources/tests/fixtures/deerflow_replay_window.json`

**Milestone unlock:** DeerFlow can be represented as spec-aligned raw envelopes and state-server-like mediated payloads without leaking backend-specific internals above the raw-source layer

**Forbidden shortcuts:** do not put canonical records in raw-sources; do not use `apps/deer_gui` bridge DTOs here; do not flatten multi-agent activity into one summary blob

- [ ] **Step 1: Write the failing DeerFlow adapter test and fixtures**

```json
// crates/pipeline/raw_sources/tests/fixtures/deerflow_thread_snapshot.json
{
  "thread_id": "thread_1",
  "title": "Survey ridge perimeter",
  "run_id": "run_1",
  "status": "running",
  "agents": [
    { "agent_id": "agent_lead", "role": "planner" },
    { "agent_id": "agent_scout", "role": "scout" }
  ]
}
```

```json
// crates/pipeline/raw_sources/tests/fixtures/deerflow_live_activity.json
{
  "source_objects": [
    {
      "source_object_id": "src_msg_1",
      "object_kind": "message",
      "level": "L0",
      "plane": "AsIs",
      "thread_id": "thread_1",
      "run_id": "run_1",
      "agent_id": "agent_lead",
      "payload": { "message_id": "msg_1", "role": "assistant", "text": "Dispatching scouts." }
    }
  ],
  "stream_deltas": [
    {
      "event_id": "evt_task_running",
      "delta_kind": "task_progress",
      "thread_id": "thread_1",
      "run_id": "run_1",
      "task_id": "task_scout",
      "agent_id": "agent_scout",
      "payload": { "label": "Scout east ridge", "state": "running" }
    },
    {
      "event_id": "evt_artifact_presented",
      "delta_kind": "artifact_presented",
      "thread_id": "thread_1",
      "run_id": "run_1",
      "task_id": "task_scout",
      "agent_id": "agent_scout",
      "artifact_id": "artifact_map_1",
      "payload": { "name": "ridge_map.md", "as_is_hash": "sha256:ridge-map" }
    }
  ],
  "runtime_status": [
    {
      "event_id": "evt_runtime_1",
      "status": "streaming",
      "thread_id": "thread_1",
      "run_id": "run_1"
    }
  ]
}
```

```json
// crates/pipeline/raw_sources/tests/fixtures/deerflow_replay_window.json
{
  "thread_id": "thread_1",
  "run_id": "run_1",
  "window_start_event_id": "evt_task_running",
  "window_end_event_id": "evt_artifact_presented",
  "events": ["evt_task_running", "evt_artifact_presented"]
}
```

```rust
// crates/pipeline/raw_sources/tests/deerflow_adapter.rs
use deer_pipeline_raw_sources::{
    deerflow::{load_deerflow_live_activity, load_deerflow_replay_window, load_deerflow_thread_snapshot},
    mediated_reads::{ArtifactPreviewPayload, LiveActivityStream, ReplayWindow, ThreadStateSnapshot},
    envelopes::{CanonicalPlaneRef, CanonicalLevelRef, RawEnvelopeBatch, RawEnvelopeFamily},
};

#[test]
fn deerflow_adapter_emits_raw_envelopes_and_mediated_reads() {
    let snapshot = load_deerflow_thread_snapshot(include_str!("fixtures/deerflow_thread_snapshot.json")).unwrap();
    let activity = load_deerflow_live_activity(include_str!("fixtures/deerflow_live_activity.json")).unwrap();
    let replay = load_deerflow_replay_window(include_str!("fixtures/deerflow_replay_window.json")).unwrap();

    assert_eq!(snapshot, ThreadStateSnapshot {
        thread_id: "thread_1".into(),
        run_id: Some("run_1".into()),
        title: "Survey ridge perimeter".into(),
        status: "running".into(),
    });
    assert_eq!(replay, ReplayWindow {
        thread_id: "thread_1".into(),
        run_id: "run_1".into(),
        window_start_event_id: "evt_task_running".into(),
        window_end_event_id: "evt_artifact_presented".into(),
        event_ids: vec!["evt_task_running".into(), "evt_artifact_presented".into()],
    });
    assert!(matches!(activity, LiveActivityStream { .. }));
    assert!(activity.batches.iter().any(|batch| matches!(
        batch,
        RawEnvelopeBatch {
            family: RawEnvelopeFamily::StreamDelta,
            level: CanonicalLevelRef::L0,
            plane: CanonicalPlaneRef::AsIs,
            ..
        }
    )));
}
```

- [ ] **Step 2: Run the raw-source adapter test and confirm it fails**

Run: `cargo test -p deer-pipeline-raw-sources --test deerflow_adapter -v`

Expected: FAIL with missing `envelopes`, `deerflow`, and `mediated_reads` modules plus missing loader functions and DTOs.

- [ ] **Step 3: Implement the DeerFlow raw envelope and mediated-read modules**

```rust
// crates/pipeline/raw_sources/src/envelopes.rs
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonicalLevelRef {
    L0,
    L1,
    L2,
    L3,
    L4,
    L5,
    L6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonicalPlaneRef {
    AsIs,
    Chunks,
    Embeddings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawEnvelopeFamily {
    SourceObject,
    StreamDelta,
    Snapshot,
    Artifact,
    LineageEvent,
    Progress,
    RuntimeStatus,
    Intent,
    RetrievalHit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawEnvelopeBatch {
    pub family: RawEnvelopeFamily,
    pub level: CanonicalLevelRef,
    pub plane: CanonicalPlaneRef,
    pub source_object_id: String,
    pub event_id: Option<String>,
    pub thread_id: Option<String>,
    pub run_id: Option<String>,
    pub task_id: Option<String>,
    pub agent_id: Option<String>,
    pub artifact_id: Option<String>,
    pub payload: Value,
}
```

```rust
// crates/pipeline/raw_sources/src/mediated_reads.rs
use serde::{Deserialize, Serialize};

use crate::envelopes::RawEnvelopeBatch;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThreadStateSnapshot {
    pub thread_id: String,
    pub run_id: Option<String>,
    pub title: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayWindow {
    pub thread_id: String,
    pub run_id: String,
    pub window_start_event_id: String,
    pub window_end_event_id: String,
    pub event_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactPreviewPayload {
    pub artifact_id: String,
    pub media_type: String,
    pub preview_mode: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveActivityStream {
    pub batches: Vec<RawEnvelopeBatch>,
}
```

```rust
// crates/pipeline/raw_sources/src/deerflow.rs
use serde::Deserialize;

use crate::{
    envelopes::{CanonicalLevelRef, CanonicalPlaneRef, RawEnvelopeBatch, RawEnvelopeFamily},
    error::RawSourceError,
    mediated_reads::{LiveActivityStream, ReplayWindow, ThreadStateSnapshot},
};

#[derive(Debug, Deserialize)]
struct DeerFlowActivityFixture {
    source_objects: Vec<RawEnvelopeBatch>,
    stream_deltas: Vec<RawEnvelopeBatch>,
    runtime_status: Vec<RuntimeEnvelopeFixture>,
}

#[derive(Debug, Deserialize)]
struct RuntimeEnvelopeFixture {
    event_id: String,
    status: String,
    thread_id: String,
    run_id: String,
}

pub fn load_deerflow_thread_snapshot(fixture_json: &str) -> Result<ThreadStateSnapshot, RawSourceError> {
    serde_json::from_str(fixture_json).map_err(|_| RawSourceError::InvalidFixture)
}

pub fn load_deerflow_replay_window(fixture_json: &str) -> Result<ReplayWindow, RawSourceError> {
    serde_json::from_str(fixture_json).map_err(|_| RawSourceError::InvalidFixture)
}

pub fn load_deerflow_live_activity(fixture_json: &str) -> Result<LiveActivityStream, RawSourceError> {
    let fixture: DeerFlowActivityFixture =
        serde_json::from_str(fixture_json).map_err(|_| RawSourceError::InvalidFixture)?;

    let mut batches = fixture.source_objects;
    batches.extend(fixture.stream_deltas);
    batches.extend(fixture.runtime_status.into_iter().map(|status| RawEnvelopeBatch {
        family: RawEnvelopeFamily::RuntimeStatus,
        level: CanonicalLevelRef::L0,
        plane: CanonicalPlaneRef::AsIs,
        source_object_id: format!("runtime:{}", status.event_id),
        event_id: Some(status.event_id),
        thread_id: Some(status.thread_id),
        run_id: Some(status.run_id),
        task_id: None,
        agent_id: None,
        artifact_id: None,
        payload: serde_json::json!({ "status": status.status }),
    }));

    Ok(LiveActivityStream { batches })
}
```

```rust
// crates/pipeline/raw_sources/src/lib.rs
pub mod artifact_gateway;
pub mod deerflow;
pub mod envelopes;
pub mod error;
pub mod mediated_reads;
pub mod run_stream;
pub mod thread_gateway;
pub mod upload_gateway;

pub use deerflow::{
    load_deerflow_live_activity, load_deerflow_replay_window, load_deerflow_thread_snapshot,
};
pub use envelopes::{CanonicalLevelRef, CanonicalPlaneRef, RawEnvelopeBatch, RawEnvelopeFamily};
pub use mediated_reads::{ArtifactPreviewPayload, LiveActivityStream, ReplayWindow, ThreadStateSnapshot};
```

- [ ] **Step 4: Re-run the DeerFlow raw-source test**

Run: `cargo test -p deer-pipeline-raw-sources --test deerflow_adapter -v`

Expected: PASS with DeerFlow fixtures loading into raw envelopes and mediated-read DTOs.

- [ ] **Step 5: Commit the DeerFlow raw-source layer**

```bash
git add crates/pipeline/raw_sources/Cargo.toml crates/pipeline/raw_sources/src/lib.rs crates/pipeline/raw_sources/src/envelopes.rs crates/pipeline/raw_sources/src/deerflow.rs crates/pipeline/raw_sources/src/mediated_reads.rs crates/pipeline/raw_sources/tests/deerflow_adapter.rs crates/pipeline/raw_sources/tests/fixtures/deerflow_thread_snapshot.json crates/pipeline/raw_sources/tests/fixtures/deerflow_live_activity.json crates/pipeline/raw_sources/tests/fixtures/deerflow_replay_window.json
git commit -m "feat: add deerflow raw envelopes and mediated reads"
```

## Task 3: Normalize DeerFlow Raw Capture Into L0/L1/L2 Canonical Records

**Files:**
- Modify: `crates/pipeline/normalizers/src/lib.rs`
- Modify: `crates/pipeline/normalizers/src/carrier.rs`
- Modify: `crates/pipeline/normalizers/src/representation.rs`
- Modify: `crates/pipeline/normalizers/src/governance.rs`
- Test: `crates/pipeline/normalizers/tests/deerflow_levels_lineage.rs`

**Milestone unlock:** DeerFlow batches produce storage-aware canonical records with explicit level/plane occupancy, lineage, and correlations that later generators can follow

**Forbidden shortcuts:** do not skip L0/L1 evidence; do not emit L2 records with empty lineage/correlations when raw envelopes carry thread/run/task/agent ids; do not treat mediated DTOs as final canonical models

- [ ] **Step 1: Write the failing DeerFlow normalization test**

```rust
// crates/pipeline/normalizers/tests/deerflow_levels_lineage.rs
use deer_foundation_contracts::{CanonicalLevel, CanonicalPlane, RecordFamily};
use deer_foundation_domain::AnyRecord;
use deer_pipeline_normalizers::normalize_deerflow_live_activity;
use deer_pipeline_raw_sources::load_deerflow_live_activity;

#[test]
fn normalizes_deerflow_raw_batches_into_l0_l1_l2_records_with_lineage() {
    let activity = load_deerflow_live_activity(include_str!("../../raw_sources/tests/fixtures/deerflow_live_activity.json")).unwrap();

    let normalized = normalize_deerflow_live_activity(&activity).unwrap();

    assert!(normalized.records.iter().any(|record| matches!(record, AnyRecord::L0Source(_))));
    assert!(normalized.records.iter().any(|record| matches!(record, AnyRecord::L1Sanitized(_))));
    assert!(normalized.records.iter().any(|record| matches!(record, AnyRecord::Task(_))));
    assert!(normalized.records.iter().any(|record| matches!(record, AnyRecord::Artifact(_))));

    let artifact = normalized.records.iter().find_map(|record| match record {
        AnyRecord::Artifact(record) => Some(record),
        _ => None,
    }).unwrap();

    assert_eq!(artifact.header.level, CanonicalLevel::L2);
    assert_eq!(artifact.header.source_level, CanonicalLevel::L0);
    assert_eq!(artifact.header.plane, CanonicalPlane::AsIs);
    assert_eq!(artifact.header.source_plane, CanonicalPlane::AsIs);
    assert_eq!(artifact.header.family, RecordFamily::Artifact);
    assert_eq!(artifact.header.correlations.run_id.as_ref().map(|id| id.as_str()), Some("run_1"));
    assert_eq!(artifact.header.correlations.task_id.as_ref().map(|id| id.as_str()), Some("task_scout"));
    assert_eq!(artifact.header.correlations.agent_id.as_ref().map(|id| id.as_str()), Some("agent_scout"));
    assert!(!artifact.header.lineage.source_events.is_empty());
}
```

- [ ] **Step 2: Run the DeerFlow normalization test and confirm it fails**

Run: `cargo test -p deer-pipeline-normalizers --test deerflow_levels_lineage -v`

Expected: FAIL with missing `normalize_deerflow_live_activity`, missing L0/L1 records, and missing correlation/source-level metadata.

- [ ] **Step 3: Implement the DeerFlow normalization path**

```rust
// crates/pipeline/normalizers/src/lib.rs
pub use carrier::{normalize_batch, normalize_deerflow_live_activity, normalize_stream_batch, NormalizedBatch};
```

```rust
// crates/pipeline/normalizers/src/carrier.rs
use deer_foundation_contracts::{
    AgentId, ArtifactId, CanonicalLevel, CanonicalPlane, CorrelationMeta, EventId, IdentityMeta,
    LineageMeta, RecordFamily, RecordId, RecordRef, RunId, TaskId, ThreadId,
};
use deer_foundation_domain::{
    AnyRecord, ArtifactBody, ArtifactRecord, L0SourceBody, L0SourceRecord, L1SanitizedBody,
    L1SanitizedRecord, MessageBody, MessageRecord, RuntimeStatusBody, RuntimeStatusRecord,
    SessionBody, SessionRecord, TaskBody, TaskRecord,
};
use deer_pipeline_raw_sources::{LiveActivityStream, RawEnvelopeBatch, RawEnvelopeFamily};

pub fn normalize_deerflow_live_activity(
    activity: &LiveActivityStream,
) -> Result<NormalizedBatch, NormalizationError> {
    let mut records = Vec::new();

    for batch in &activity.batches {
        let raw_record_id = RecordId::from(format!("raw:{}", batch.source_object_id));
        records.push(AnyRecord::L0Source(L0SourceRecord::new_with_meta(
            raw_record_id.clone(),
            IdentityMeta::hash_anchored(raw_record_id.clone(), None, None, None),
            correlations_for_batch(batch),
            LineageMeta {
                derived_from: Vec::new(),
                source_events: batch.event_id.clone().into_iter().map(EventId::from).collect(),
                supersedes: None,
            },
            CanonicalLevel::L0,
            CanonicalPlane::AsIs,
            L0SourceBody {
                summary: format!("{:?}:{}", batch.family, batch.source_object_id),
            },
        )));

        let sanitized_record_id = RecordId::from(format!("san:{}", batch.source_object_id));
        records.push(AnyRecord::L1Sanitized(L1SanitizedRecord::new_with_meta(
            sanitized_record_id.clone(),
            IdentityMeta::hash_anchored(sanitized_record_id.clone(), None, None, None),
            correlations_for_batch(batch),
            LineageMeta::derived_from(RecordRef::new(RecordFamily::L0Source, raw_record_id)),
            CanonicalLevel::L0,
            CanonicalPlane::AsIs,
            L1SanitizedBody {
                summary: format!("sanitized:{:?}:{}", batch.family, batch.source_object_id),
            },
        )));

        match batch.family {
            RawEnvelopeFamily::StreamDelta => emit_l2_from_stream_delta(batch, &mut records),
            RawEnvelopeFamily::RuntimeStatus => emit_runtime_status(batch, &mut records),
            RawEnvelopeFamily::SourceObject => emit_message_source(batch, &mut records),
            _ => {}
        }
    }

    Ok(NormalizedBatch { records })
}

fn correlations_for_batch(batch: &RawEnvelopeBatch) -> CorrelationMeta {
    CorrelationMeta {
        run_id: batch.run_id.as_deref().map(RunId::from),
        thread_id: batch.thread_id.as_deref().map(ThreadId::from),
        task_id: batch.task_id.as_deref().map(TaskId::from),
        artifact_id: batch.artifact_id.as_deref().map(ArtifactId::from),
        agent_id: batch.agent_id.as_deref().map(AgentId::from),
        ..Default::default()
    }
}
```

```rust
// crates/pipeline/normalizers/src/representation.rs
use deer_foundation_contracts::{CanonicalLevel, CanonicalPlane, IdentityMeta, RecordId};
use deer_foundation_domain::{AnyRecord, AsIsRepresentationBody, AsIsRepresentationRecord};
use deer_pipeline_raw_sources::RawEnvelopeBatch;

pub fn normalize_deerflow_as_is(batch: &RawEnvelopeBatch) -> AnyRecord {
    AnyRecord::AsIsRepresentation(AsIsRepresentationRecord::new_with_meta(
        RecordId::from(format!("repr:{}", batch.source_object_id)),
        IdentityMeta::hash_anchored(RecordId::from(format!("repr:{}", batch.source_object_id)), None, None, None),
        crate::carrier::correlations_for_batch(batch),
        deer_foundation_contracts::LineageMeta::root(),
        CanonicalLevel::L0,
        CanonicalPlane::AsIs,
        AsIsRepresentationBody {
            media_type: "application/json".into(),
        },
    ))
}
```

- [ ] **Step 4: Re-run the DeerFlow normalization test**

Run: `cargo test -p deer-pipeline-normalizers --test deerflow_levels_lineage -v`

Expected: PASS with L0/L1/L2 occupancy, source-level/source-plane metadata, and lineage/correlations preserved.

- [ ] **Step 5: Commit the DeerFlow normalization path**

```bash
git add crates/pipeline/normalizers/src/lib.rs crates/pipeline/normalizers/src/carrier.rs crates/pipeline/normalizers/src/representation.rs crates/pipeline/normalizers/src/governance.rs crates/pipeline/normalizers/tests/deerflow_levels_lineage.rs
git commit -m "feat: normalize deerflow activity into canonical records"
```

## Task 4: Derive L2 Game-Facing Views From DeerFlow Canonical Records

**Files:**
- Create: `crates/pipeline/derivations/src/game_face.rs`
- Modify: `crates/pipeline/derivations/src/lib.rs`
- Test: `crates/pipeline/derivations/tests/deerflow_game_face.rs`

**Milestone unlock:** reusable game-facing L2 views exist for units, relationships, queue, history, and telemetry without depending on Bevy or `deer_gui`

**Forbidden shortcuts:** do not invent app-local unit ids; do not lose backlinks to source records; do not mix observed L2 state with forecast/prescription semantics

- [ ] **Step 1: Write the failing game-facing derivation test**

```rust
// crates/pipeline/derivations/tests/deerflow_game_face.rs
use deer_pipeline_derivations::derive_game_face_vm;
use deer_pipeline_normalizers::normalize_deerflow_live_activity;
use deer_pipeline_raw_sources::load_deerflow_live_activity;

#[test]
fn derives_units_queue_history_and_telemetry_from_deerflow_records() {
    let activity = load_deerflow_live_activity(include_str!("../../raw_sources/tests/fixtures/deerflow_live_activity.json")).unwrap();
    let normalized = normalize_deerflow_live_activity(&activity).unwrap();

    let game_face = derive_game_face_vm(&normalized.records);

    assert_eq!(game_face.units.len(), 1);
    assert!(game_face.units.iter().any(|unit| unit.agent_id == "agent_scout"));
    assert!(game_face.task_forces.iter().any(|link| link.task_id == "task_scout"));
    assert!(game_face.queue.rows.iter().any(|row| row.task_id == "task_scout" && row.status == "running"));
    assert!(game_face.history.rows.iter().any(|row| row.event_id == "evt_artifact_presented"));
    assert_eq!(game_face.telemetry.active_agents, 2);
}
```

- [ ] **Step 2: Run the derivation test and confirm it fails**

Run: `cargo test -p deer-pipeline-derivations --test deerflow_game_face -v`

Expected: FAIL with missing `derive_game_face_vm` and missing unit/queue/history/telemetry VM types.

- [ ] **Step 3: Implement the game-facing derivation module**

```rust
// crates/pipeline/derivations/src/game_face.rs
use deer_foundation_contracts::CanonicalRecord;
use deer_foundation_domain::AnyRecord;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UnitActorVm {
    pub agent_id: String,
    pub source_record_id: String,
    pub status: String,
    pub panel_target: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TaskForceLinkVm {
    pub task_id: String,
    pub agent_id: String,
    pub source_record_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QueueRowVm {
    pub task_id: String,
    pub status: String,
    pub source_record_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HistoryRowVm {
    pub event_id: String,
    pub source_record_id: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TelemetryVm {
    pub active_agents: usize,
    pub running_tasks: usize,
    pub artifact_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QueueVm {
    pub rows: Vec<QueueRowVm>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HistoryVm {
    pub rows: Vec<HistoryRowVm>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GameFaceVm {
    pub units: Vec<UnitActorVm>,
    pub task_forces: Vec<TaskForceLinkVm>,
    pub queue: QueueVm,
    pub history: HistoryVm,
    pub telemetry: TelemetryVm,
}

pub fn derive_game_face_vm(records: &[AnyRecord]) -> GameFaceVm {
    let mut units = Vec::new();
    let mut task_forces = Vec::new();
    let mut queue_rows = Vec::new();
    let mut history_rows = Vec::new();
    let mut agent_ids = BTreeSet::new();
    let mut artifact_count = 0usize;

    for record in records {
        let header = record.header();
        if let Some(agent_id) = header.correlations.agent_id.as_ref() {
            agent_ids.insert(agent_id.to_string());
        }

        match record {
            AnyRecord::Task(task) => {
                let agent_id = header.correlations.agent_id.as_ref().map(|id| id.to_string()).unwrap_or_else(|| "unassigned".into());
                units.push(UnitActorVm {
                    agent_id: agent_id.clone(),
                    source_record_id: task.record_id().to_string(),
                    status: task.body.status.clone(),
                    panel_target: "task_detail",
                });
                task_forces.push(TaskForceLinkVm {
                    task_id: task.record_id().to_string(),
                    agent_id,
                    source_record_id: task.record_id().to_string(),
                });
                queue_rows.push(QueueRowVm {
                    task_id: task.record_id().to_string(),
                    status: task.body.status.clone(),
                    source_record_id: task.record_id().to_string(),
                });
            }
            AnyRecord::Artifact(artifact) => {
                artifact_count += 1;
                let event_id = artifact
                    .header
                    .lineage
                    .source_events
                    .first()
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| format!("artifact:{}", artifact.record_id()));
                history_rows.push(HistoryRowVm {
                    event_id,
                    source_record_id: artifact.record_id().to_string(),
                    label: artifact.body.label.clone(),
                });
            }
            _ => {}
        }
    }

    let running_tasks = queue_rows.iter().filter(|row| row.status == "running").count();

    GameFaceVm {
        units,
        task_forces,
        queue: QueueVm { rows: queue_rows },
        history: HistoryVm { rows: history_rows },
        telemetry: TelemetryVm {
            active_agents: agent_ids.len(),
            running_tasks,
            artifact_count,
        },
    }
}
```

```rust
// crates/pipeline/derivations/src/lib.rs
pub mod game_face;

pub use game_face::{derive_game_face_vm, GameFaceVm, HistoryRowVm, QueueRowVm, TaskForceLinkVm, TelemetryVm, UnitActorVm};
```

- [ ] **Step 4: Re-run the DeerFlow game-facing derivation test**

Run: `cargo test -p deer-pipeline-derivations --test deerflow_game_face -v`

Expected: PASS with stable unit, relationship, queue, history, and telemetry derivations.

- [ ] **Step 5: Commit the game-facing derivation layer**

```bash
git add crates/pipeline/derivations/src/lib.rs crates/pipeline/derivations/src/game_face.rs crates/pipeline/derivations/tests/deerflow_game_face.rs
git commit -m "feat: derive deerflow game-facing projections"
```

## Task 5: Project DeerFlow Canonical Records Into World Objects And Prove The End-To-End Path

**Files:**
- Modify: `crates/runtime/world_projection/src/world_object.rs`
- Modify: `crates/runtime/world_projection/src/projection_rules.rs`
- Test: `crates/runtime/world_projection/tests/deerflow_pipeline_projection.rs`

**Milestone unlock:** the reusable stack proves `DeerFlow -> raw -> canonical -> L2 game-facing -> world objects` without involving `deer_gui`

**Forbidden shortcuts:** do not construct world objects directly from raw envelopes; do not drop level/plane/source metadata at projection time; do not make world objects the canonical truth layer

- [ ] **Step 1: Write the failing end-to-end projection test**

```rust
// crates/runtime/world_projection/tests/deerflow_pipeline_projection.rs
use deer_pipeline_derivations::derive_game_face_vm;
use deer_pipeline_normalizers::normalize_deerflow_live_activity;
use deer_pipeline_raw_sources::load_deerflow_live_activity;
use deer_runtime_world_projection::project_world_objects;
use insta::assert_yaml_snapshot;

#[test]
fn projects_deerflow_pipeline_into_world_units_queue_and_history_objects() {
    let activity = load_deerflow_live_activity(include_str!("../../pipeline/raw_sources/tests/fixtures/deerflow_live_activity.json")).unwrap();
    let normalized = normalize_deerflow_live_activity(&activity).unwrap();
    let projection = project_world_objects(&normalized.records);
    let game_face = derive_game_face_vm(&normalized.records);

    assert_eq!(game_face.telemetry.active_agents, 2);
    assert_yaml_snapshot!(projection, @r#"
objects:
  - kind: WorldUnitActor
    source_record_id: task_scout
    drill_down_target: task_detail
  - kind: WorldArtifactUnlock
    source_record_id: artifact_map_1
    drill_down_target: artifact_detail
  - kind: WorldHistoryAnchor
    source_record_id: artifact_map_1
    drill_down_target: history_detail
"#);
}
```

- [ ] **Step 2: Run the world-projection pipeline test and confirm it fails**

Run: `cargo test -p deer-runtime-world-projection --test deerflow_pipeline_projection -v`

Expected: FAIL with missing world object kinds and projection rules for DeerFlow-backed units/history anchors.

- [ ] **Step 3: Extend world objects and projection rules**

```rust
// crates/runtime/world_projection/src/world_object.rs
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorldObject {
    pub kind: &'static str,
    pub source_record_id: String,
    pub drill_down_target: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<RecordRef>,
    #[serde(skip_serializing)]
    pub level: &'static str,
    #[serde(skip_serializing)]
    pub plane: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
}

impl WorldObject {
    pub fn unit_actor(source_record_id: &str, drill_down_target: &'static str, agent_id: Option<String>) -> Self {
        Self {
            kind: "WorldUnitActor",
            source_record_id: source_record_id.to_owned(),
            drill_down_target,
            supersedes: None,
            level: "L2",
            plane: "AsIs",
            agent_id,
        }
    }

    pub fn history_anchor(source_record_id: &str, drill_down_target: &'static str) -> Self {
        Self {
            kind: "WorldHistoryAnchor",
            source_record_id: source_record_id.to_owned(),
            drill_down_target,
            supersedes: None,
            level: "L2",
            plane: "AsIs",
            agent_id: None,
        }
    }
}
```

```rust
// crates/runtime/world_projection/src/projection_rules.rs
use deer_foundation_contracts::CanonicalRecord;
use deer_foundation_domain::AnyRecord;

use crate::world_object::{WorldObject, WorldProjection};

pub fn project_world_objects(records: &[AnyRecord]) -> WorldProjection {
    let objects = records
        .iter()
        .filter_map(|record| match record {
            AnyRecord::Task(task) => Some(
                WorldObject::unit_actor(
                    task.record_id().as_str(),
                    "task_detail",
                    task.header
                        .correlations
                        .agent_id
                        .as_ref()
                        .map(|id| id.to_string()),
                )
                .with_supersession(task.header.lineage.supersedes.clone()),
            ),
            AnyRecord::Artifact(artifact) => Some(
                WorldObject::artifact_unlock(artifact.record_id().as_str(), "artifact_detail")
                    .with_supersession(artifact.header.lineage.supersedes.clone()),
            ),
            AnyRecord::Artifact(artifact) => Some(
                WorldObject::history_anchor(artifact.record_id().as_str(), "history_detail")
                    .with_supersession(artifact.header.lineage.supersedes.clone()),
            ),
            _ => None,
        })
        .collect();

    WorldProjection { objects }
}
```

- [ ] **Step 4: Run the end-to-end DeerFlow projection proof and the impacted package sweep**

Run: `cargo test -p deer-runtime-world-projection --test deerflow_pipeline_projection -v && cargo test -p deer-pipeline-raw-sources -v && cargo test -p deer-pipeline-normalizers -v && cargo test -p deer-pipeline-derivations -v && cargo test -p deer-runtime-world-projection -v`

Expected: PASS with a reusable DeerFlow-backed pipeline proving raw capture, canonicalization, L2 game-facing derivation, and world projection.

- [ ] **Step 5: Commit the DeerFlow world-projection proof**

```bash
git add crates/runtime/world_projection/src/world_object.rs crates/runtime/world_projection/src/projection_rules.rs crates/runtime/world_projection/tests/deerflow_pipeline_projection.rs
git commit -m "feat: project deerflow pipeline into world objects"
```

## Self-Review Checklist

- The plan starts in shared reusable crates and does not debut new behavior inside `apps/deer_gui`.
- The pipeline follows the state-server-aligned order: generator -> storage truth -> mediated source -> canonical -> derivation -> world projection.
- L0/L1/L2 occupancy is explicit and tested; L3+ is intentionally out of scope for this first DeerFlow onboarding slice.
- `as_is` is required end to end, and selective chunk/representation support is introduced without making embeddings a blocker.
- Correlation and lineage metadata are preserved through normalization and exposed in later game-facing projections.
- The end-to-end proof shows a pattern that Hermes/Rowboat/PocketFlow can follow by swapping only the raw-source adapter and sparse/rich mapping rules.
