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

#[derive(Debug, Deserialize)]
struct DeerFlowIntentFixture {
    intent_id: String,
    thread_id: String,
    run_id: String,
    agent_id: String,
    action: String,
    payload: serde_json::Value,
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
            "payload": fixture.payload,
        }),
    })
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
