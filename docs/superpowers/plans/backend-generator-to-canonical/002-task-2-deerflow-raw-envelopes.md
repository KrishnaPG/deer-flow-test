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
- Test fixture: `crates/pipeline/raw_sources/tests/fixtures/deerflow_intent.json`
- Test fixture: `crates/pipeline/raw_sources/tests/fixtures/deerflow_artifact_preview.json`
- Test fixture: `crates/pipeline/raw_sources/tests/fixtures/deerflow_exclusion_intent.json`
- Test fixture: `crates/pipeline/raw_sources/tests/fixtures/deerflow_conflicting_intents.json`
- Test fixture: `crates/pipeline/raw_sources/tests/fixtures/deerflow_storage_backpressure.json`
- Test fixture: `crates/pipeline/raw_sources/tests/fixtures/deerflow_replay_checkpoint.json`

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

```json
// crates/pipeline/raw_sources/tests/fixtures/deerflow_intent.json
{
  "intent_id": "intent_1",
  "thread_id": "thread_1",
  "run_id": "run_1",
  "agent_id": "agent_lead",
  "action": "dispatch_scouts",
  "payload": { "target": "east_ridge", "priority": "high" },
  "requested_at": "2026-04-03T10:00:00Z"
}
```

```json
// crates/pipeline/raw_sources/tests/fixtures/deerflow_exclusion_intent.json
{
  "intent_id": "excl_1",
  "thread_id": "thread_1",
  "run_id": "run_1",
  "agent_id": "operator_1",
  "action": "request_exclusion",
  "payload": { "target_hash": "sha256:abc123", "reason": "compliance_request", "target_record_id": "artifact_map_1" },
  "requested_at": "2026-04-03T12:00:00Z"
}
```

```json
// crates/pipeline/raw_sources/tests/fixtures/deerflow_artifact_preview.json
{
  "artifact_id": "artifact_map_1",
  "media_type": "text/markdown",
  "preview_mode": "inline_text",
  "authorization_kind": "presigned_url",
  "access_url": "https://state-server.example/artifacts/artifact_map_1?token=abc",
  "expires_at": "2026-04-03T12:15:00Z"
}
```

```json
// crates/pipeline/raw_sources/tests/fixtures/deerflow_conflicting_intents.json
[
  {
    "intent_id": "intent_glossary_a",
    "thread_id": "thread_1",
    "run_id": "run_1",
    "agent_id": "operator_a",
    "action": "update_glossary",
    "sequence_id": 101,
    "payload": { "term": "Apples", "value": "Red" },
    "requested_at": "2026-04-03T12:00:00Z"
  },
  {
    "intent_id": "intent_glossary_b",
    "thread_id": "thread_1",
    "run_id": "run_1",
    "agent_id": "operator_b",
    "action": "update_glossary",
    "sequence_id": 102,
    "payload": { "term": "Apples", "value": "Green" },
    "requested_at": "2026-04-03T12:00:00Z"
  }
]
```

```json
// crates/pipeline/raw_sources/tests/fixtures/deerflow_storage_backpressure.json
{
  "event_id": "evt_backpressure_1",
  "thread_id": "thread_1",
  "run_id": "run_1",
  "queue_depth": 2048,
  "throttle_state": "slow_down",
  "retry_after_ms": 5000,
  "storage_provider": "s3"
}
```

```json
// crates/pipeline/raw_sources/tests/fixtures/deerflow_replay_checkpoint.json
{
  "checkpoint_id": "chk_1",
  "thread_id": "thread_1",
  "run_id": "run_1",
  "last_sequence_id": 4242,
  "captured_at": "2026-04-03T12:20:00Z"
}
```

```rust
// crates/pipeline/raw_sources/tests/deerflow_adapter.rs
use deer_pipeline_raw_sources::{
    deerflow::{
        load_deerflow_artifact_preview, load_deerflow_conflicting_intents, load_deerflow_live_activity,
        load_deerflow_replay_checkpoint, load_deerflow_replay_window, load_deerflow_storage_backpressure,
        load_deerflow_thread_snapshot,
    },
    mediated_reads::{
        ArtifactPreviewPayload, LiveActivityStream, ReplayCheckpoint, ReplayWindow, StorageBackpressure,
        ThreadStateSnapshot,
    },
    envelopes::{CanonicalPlaneRef, CanonicalLevelRef, RawEnvelopeBatch, RawEnvelopeFamily},
};

#[test]
fn deerflow_adapter_emits_raw_envelopes_and_mediated_reads() {
    let snapshot = load_deerflow_thread_snapshot(include_str!("fixtures/deerflow_thread_snapshot.json")).unwrap();
    let activity = load_deerflow_live_activity(include_str!("fixtures/deerflow_live_activity.json")).unwrap();
    let replay = load_deerflow_replay_window(include_str!("fixtures/deerflow_replay_window.json")).unwrap();
    let preview = load_deerflow_artifact_preview(include_str!("fixtures/deerflow_artifact_preview.json")).unwrap();
    let conflicting = load_deerflow_conflicting_intents(include_str!("fixtures/deerflow_conflicting_intents.json")).unwrap();
    let backpressure = load_deerflow_storage_backpressure(include_str!("fixtures/deerflow_storage_backpressure.json")).unwrap();
    let checkpoint = load_deerflow_replay_checkpoint(include_str!("fixtures/deerflow_replay_checkpoint.json")).unwrap();

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
    assert_eq!(preview, ArtifactPreviewPayload {
        artifact_id: "artifact_map_1".into(),
        media_type: "text/markdown".into(),
        preview_mode: "inline_text".into(),
        authorization_kind: "presigned_url".into(),
        access_url: "https://state-server.example/artifacts/artifact_map_1?token=abc".into(),
        expires_at: "2026-04-03T12:15:00Z".into(),
    });
    assert_eq!(conflicting.len(), 2);
    assert_eq!(conflicting[0].event_id.as_deref(), Some("intent_glossary_a"));
    assert_eq!(conflicting[1].event_id.as_deref(), Some("intent_glossary_b"));
    assert_eq!(backpressure, StorageBackpressure {
        event_id: "evt_backpressure_1".into(),
        thread_id: "thread_1".into(),
        run_id: "run_1".into(),
        queue_depth: 2048,
        throttle_state: "slow_down".into(),
        retry_after_ms: 5000,
        storage_provider: "s3".into(),
    });
    assert_eq!(checkpoint, ReplayCheckpoint {
        checkpoint_id: "chk_1".into(),
        thread_id: "thread_1".into(),
        run_id: "run_1".into(),
        last_sequence_id: 4242,
        captured_at: "2026-04-03T12:20:00Z".into(),
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
    pub authorization_kind: String,
    pub access_url: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageBackpressure {
    pub event_id: String,
    pub thread_id: String,
    pub run_id: String,
    pub queue_depth: usize,
    pub throttle_state: String,
    pub retry_after_ms: u64,
    pub storage_provider: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayCheckpoint {
    pub checkpoint_id: String,
    pub thread_id: String,
    pub run_id: String,
    pub last_sequence_id: u64,
    pub captured_at: String,
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

#[derive(Debug, Deserialize)]
struct DeerFlowIntentFixture {
    intent_id: String,
    thread_id: String,
    run_id: String,
    agent_id: String,
    action: String,
    sequence_id: Option<u64>,
    payload: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct DeerFlowArtifactPreviewFixture {
    artifact_id: String,
    media_type: String,
    preview_mode: String,
    authorization_kind: String,
    access_url: String,
    expires_at: String,
}

#[derive(Debug, Deserialize)]
struct DeerFlowStorageBackpressureFixture {
    event_id: String,
    thread_id: String,
    run_id: String,
    queue_depth: usize,
    throttle_state: String,
    retry_after_ms: u64,
    storage_provider: String,
}

#[derive(Debug, Deserialize)]
struct DeerFlowReplayCheckpointFixture {
    checkpoint_id: String,
    thread_id: String,
    run_id: String,
    last_sequence_id: u64,
    captured_at: String,
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

pub fn load_deerflow_intent(fixture_json: &str) -> Result<RawEnvelopeBatch, RawSourceError> {
    let fixture: DeerFlowIntentFixture =
        serde_json::from_str(fixture_json).map_err(|_| RawSourceError::InvalidFixture)?;

    Ok(RawEnvelopeBatch {
        family: RawEnvelopeFamily::Intent,
        level: CanonicalLevelRef::L0,
        plane: CanonicalPlaneRef::AsIs,
        source_object_id: format!("intent:{}", fixture.intent_id),
        event_id: Some(fixture.intent_id),
        thread_id: Some(fixture.thread_id),
        run_id: Some(fixture.run_id),
        task_id: None,
        agent_id: Some(fixture.agent_id),
        artifact_id: None,
        payload: serde_json::json!({
            "action": fixture.action,
            "sequence_id": fixture.sequence_id,
            "payload": fixture.payload,
        }),
    })
}

pub fn load_deerflow_conflicting_intents(fixture_json: &str) -> Result<Vec<RawEnvelopeBatch>, RawSourceError> {
    let fixtures: Vec<DeerFlowIntentFixture> =
        serde_json::from_str(fixture_json).map_err(|_| RawSourceError::InvalidFixture)?;

    Ok(fixtures
        .into_iter()
        .map(|fixture| RawEnvelopeBatch {
            family: RawEnvelopeFamily::Intent,
            level: CanonicalLevelRef::L0,
            plane: CanonicalPlaneRef::AsIs,
            source_object_id: format!("intent:{}", fixture.intent_id),
            event_id: Some(fixture.intent_id),
            thread_id: Some(fixture.thread_id),
            run_id: Some(fixture.run_id),
            task_id: None,
            agent_id: Some(fixture.agent_id),
            artifact_id: None,
            payload: serde_json::json!({
                "action": fixture.action,
                "sequence_id": fixture.sequence_id,
                "payload": fixture.payload,
            }),
        })
        .collect())
}

#[derive(Debug, Deserialize)]
struct DeerFlowExclusionIntentFixture {
    intent_id: String,
    thread_id: String,
    run_id: String,
    agent_id: String,
    action: String,
    payload: ExclusionIntentPayload,
}

#[derive(Debug, Deserialize)]
struct ExclusionIntentPayload {
    target_hash: String,
    reason: String,
    target_record_id: String,
}

pub fn load_deerflow_exclusion_intent(fixture_json: &str) -> Result<RawEnvelopeBatch, RawSourceError> {
    let fixture: DeerFlowExclusionIntentFixture =
        serde_json::from_str(fixture_json).map_err(|_| RawSourceError::InvalidFixture)?;

    Ok(RawEnvelopeBatch {
        family: RawEnvelopeFamily::Intent,
        level: CanonicalLevelRef::L0,
        plane: CanonicalPlaneRef::AsIs,
        source_object_id: format!("exclusion:{}", fixture.intent_id),
        event_id: Some(fixture.intent_id),
        thread_id: Some(fixture.thread_id),
        run_id: Some(fixture.run_id),
        task_id: None,
        agent_id: Some(fixture.agent_id),
        artifact_id: None,
        payload: serde_json::json!({
            "action": fixture.action,
            "target_hash": fixture.payload.target_hash,
            "reason": fixture.payload.reason,
            "target_record_id": fixture.payload.target_record_id,
        }),
    })
}

pub fn load_deerflow_artifact_preview(
    fixture_json: &str,
) -> Result<ArtifactPreviewPayload, RawSourceError> {
    serde_json::from_str(fixture_json).map_err(|_| RawSourceError::InvalidFixture)
}

pub fn load_deerflow_storage_backpressure(
    fixture_json: &str,
) -> Result<StorageBackpressure, RawSourceError> {
    serde_json::from_str(fixture_json).map_err(|_| RawSourceError::InvalidFixture)
}

pub fn load_deerflow_replay_checkpoint(
    fixture_json: &str,
) -> Result<ReplayCheckpoint, RawSourceError> {
    serde_json::from_str(fixture_json).map_err(|_| RawSourceError::InvalidFixture)
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
    load_deerflow_artifact_preview, load_deerflow_conflicting_intents, load_deerflow_exclusion_intent,
    load_deerflow_intent, load_deerflow_live_activity, load_deerflow_replay_checkpoint,
    load_deerflow_replay_window, load_deerflow_storage_backpressure, load_deerflow_thread_snapshot,
};
pub use envelopes::{CanonicalLevelRef, CanonicalPlaneRef, RawEnvelopeBatch, RawEnvelopeFamily};
pub use mediated_reads::{
    ArtifactPreviewPayload, LiveActivityStream, ReplayCheckpoint, ReplayWindow, StorageBackpressure,
    ThreadStateSnapshot,
};
```

- [ ] **Step 4: Re-run the DeerFlow raw-source test**

Run: `cargo test -p deer-pipeline-raw-sources --test deerflow_adapter -v`

Expected: PASS with DeerFlow fixtures loading into raw envelopes, ordered intents, authorized artifact previews, storage backpressure DTOs, and replay checkpoint DTOs.

- [ ] **Step 5: Commit the DeerFlow raw-source layer**

```bash
git add crates/pipeline/raw_sources/Cargo.toml crates/pipeline/raw_sources/src/lib.rs crates/pipeline/raw_sources/src/envelopes.rs crates/pipeline/raw_sources/src/deerflow.rs crates/pipeline/raw_sources/src/mediated_reads.rs crates/pipeline/raw_sources/tests/deerflow_adapter.rs crates/pipeline/raw_sources/tests/fixtures/deerflow_thread_snapshot.json crates/pipeline/raw_sources/tests/fixtures/deerflow_live_activity.json crates/pipeline/raw_sources/tests/fixtures/deerflow_replay_window.json crates/pipeline/raw_sources/tests/fixtures/deerflow_intent.json crates/pipeline/raw_sources/tests/fixtures/deerflow_artifact_preview.json crates/pipeline/raw_sources/tests/fixtures/deerflow_exclusion_intent.json crates/pipeline/raw_sources/tests/fixtures/deerflow_conflicting_intents.json crates/pipeline/raw_sources/tests/fixtures/deerflow_storage_backpressure.json crates/pipeline/raw_sources/tests/fixtures/deerflow_replay_checkpoint.json
git commit -m "feat: add deerflow raw envelopes and mediated reads"
```
