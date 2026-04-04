### Task 2: DeerFlow Raw Landing into Shared A/B Families

**Files:**
- Create: `crates/pipeline/raw_sources/src/deerflow.rs`
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

- [ ] **Step 1: Create test fixtures**

Create `crates/pipeline/raw_sources/tests/fixtures/deerflow_thread_snapshot.json`:

```json
{
  "generator": "deerflow",
  "source_kind": "thread_snapshot",
  "thread_id": "thread_df_1",
  "session_id": "session_df_1",
  "title": "Survey the ridge",
  "run_id": "run_df_1",
  "agent_ids": ["agent_alpha", "agent_beta"],
  "observed_at": "2026-04-04T00:00:00Z"
}
```

Create `crates/pipeline/raw_sources/tests/fixtures/deerflow_live_activity.json`:

```json
{
  "generator": "deerflow",
  "source_kind": "live_activity",
  "thread_id": "thread_df_1",
  "run_id": "run_df_1",
  "events": [
    {
      "event_id": "evt_msg_1",
      "kind": "message",
      "message_id": "msg_df_1",
      "role": "operator",
      "text": "Survey the ridge.",
      "agent_id": "agent_alpha"
    },
    {
      "event_id": "evt_task_1",
      "kind": "task_progress",
      "task_id": "task_df_1",
      "state": "running",
      "label": "Gather terrain notes"
    },
    {
      "event_id": "evt_art_1",
      "kind": "artifact_presented",
      "artifact_id": "artifact_df_1",
      "name": "scan.png",
      "task_id": "task_df_1"
    },
    {
      "event_id": "evt_rt_1",
      "kind": "runtime_status",
      "state": "live",
      "agent_id": "agent_alpha"
    }
  ]
}
```

Create `crates/pipeline/raw_sources/tests/fixtures/deerflow_replay_window.json`:

```json
{
  "generator": "deerflow",
  "source_kind": "replay_window",
  "thread_id": "thread_df_1",
  "window_id": "window_df_1",
  "start_sequence": 1,
  "end_sequence": 10,
  "run_id": "run_df_1"
}
```

Create `crates/pipeline/raw_sources/tests/fixtures/deerflow_intent.json`:

```json
{
  "generator": "deerflow",
  "source_kind": "intent",
  "intent_id": "intent_df_1",
  "thread_id": "thread_df_1",
  "actor_id": "operator_1",
  "action": "survey",
  "stage": "submitted"
}
```

Create `crates/pipeline/raw_sources/tests/fixtures/deerflow_artifact_preview.json`:

```json
{
  "generator": "deerflow",
  "source_kind": "artifact_access",
  "artifact_id": "artifact_df_1",
  "access_id": "access_df_1",
  "mime_type": "image/png",
  "task_id": "task_df_1",
  "run_id": "run_df_1"
}
```

Create `crates/pipeline/raw_sources/tests/fixtures/deerflow_exclusion_intent.json`:

```json
{
  "generator": "deerflow",
  "source_kind": "exclusion",
  "exclusion_id": "excl_df_1",
  "target_id": "artifact_df_1",
  "target_kind": "artifact",
  "reason": "classified",
  "thread_id": "thread_df_1"
}
```

Create `crates/pipeline/raw_sources/tests/fixtures/deerflow_conflicting_intents.json`:

```json
{
  "generator": "deerflow",
  "source_kind": "conflict",
  "conflict_id": "conflict_df_1",
  "target_id": "task_df_1",
  "target_kind": "task",
  "conflicting_intents": ["intent_df_1", "intent_df_2"],
  "thread_id": "thread_df_1"
}
```

Create `crates/pipeline/raw_sources/tests/fixtures/deerflow_storage_backpressure.json`:

```json
{
  "generator": "deerflow",
  "source_kind": "backpressure",
  "backpressure_id": "bp_df_1",
  "run_id": "run_df_1",
  "severity": "high",
  "source": "artifact_store"
}
```

Create `crates/pipeline/raw_sources/tests/fixtures/deerflow_replay_checkpoint.json`:

```json
{
  "generator": "deerflow",
  "source_kind": "replay_checkpoint",
  "checkpoint_id": "cp_df_1",
  "run_id": "run_df_1",
  "sequence_id": 5,
  "event_time": "2026-04-04T00:05:00Z"
}
```

- [ ] **Step 2: Write the failing DeerFlow landing test**

Create `crates/pipeline/raw_sources/tests/deerflow_adapter.rs`:

```rust
use deer_pipeline_raw_sources::deerflow::{
    bind_deerflow_fixtures, DeerFlowBinding, DeerFlowSourceKind,
};
use deer_pipeline_raw_sources::storage_contract::{Hierarchy, StorageContract};

#[test]
fn thread_snapshot_binds_to_a_session_snapshot() {
    let fixture = include_str!("fixtures/deerflow_thread_snapshot.json");
    let bindings = bind_deerflow_fixtures(fixture).unwrap();

    let session_binding = bindings
        .iter()
        .find(|b| b.target_family == "a_session_snapshot")
        .expect("a_session_snapshot binding not found");

    assert_eq!(session_binding.source_kind, DeerFlowSourceKind::ThreadSnapshot);
    assert_eq!(session_binding.generator_key, "deerflow");
    assert!(!session_binding.immutable_keys.is_empty());
}

#[test]
fn live_activity_binds_to_correct_a_families() {
    let fixture = include_str!("fixtures/deerflow_live_activity.json");
    let bindings = bind_deerflow_fixtures(fixture).unwrap();

    let families: Vec<&str> = bindings.iter().map(|b| b.target_family.as_str()).collect();

    assert!(families.contains(&"a_message_event"));
    assert!(families.contains(&"a_task_event"));
    assert!(families.contains(&"a_artifact_event"));
    assert!(families.contains(&"a_runtime_event"));
}

#[test]
fn replay_window_binds_to_b_replay_window() {
    let fixture = include_str!("fixtures/deerflow_replay_window.json");
    let bindings = bind_deerflow_fixtures(fixture).unwrap();

    let replay_binding = bindings
        .iter()
        .find(|b| b.target_family == "b_replay_window")
        .expect("b_replay_window binding not found");

    assert_eq!(replay_binding.source_kind, DeerFlowSourceKind::ReplayWindow);
}

#[test]
fn intent_binds_to_a_intent_event_and_b_intent() {
    let fixture = include_str!("fixtures/deerflow_intent.json");
    let bindings = bind_deerflow_fixtures(fixture).unwrap();

    let families: Vec<&str> = bindings.iter().map(|b| b.target_family.as_str()).collect();
    assert!(families.contains(&"a_intent_event"));
    assert!(families.contains(&"b_intent"));
}

#[test]
fn artifact_preview_binds_to_a_artifact_event_and_b_artifact_access() {
    let fixture = include_str!("fixtures/deerflow_artifact_preview.json");
    let bindings = bind_deerflow_fixtures(fixture).unwrap();

    let families: Vec<&str> = bindings.iter().map(|b| b.target_family.as_str()).collect();
    assert!(families.contains(&"a_artifact_event"));
    assert!(families.contains(&"b_artifact_access"));
}

#[test]
fn exclusion_binds_to_a_exclusion_event_and_b_exclusion() {
    let fixture = include_str!("fixtures/deerflow_exclusion_intent.json");
    let bindings = bind_deerflow_fixtures(fixture).unwrap();

    let families: Vec<&str> = bindings.iter().map(|b| b.target_family.as_str()).collect();
    assert!(families.contains(&"a_exclusion_event"));
    assert!(families.contains(&"b_exclusion"));
}

#[test]
fn conflict_binds_to_b_conflict() {
    let fixture = include_str!("fixtures/deerflow_conflicting_intents.json");
    let bindings = bind_deerflow_fixtures(fixture).unwrap();

    let conflict_binding = bindings
        .iter()
        .find(|b| b.target_family == "b_conflict")
        .expect("b_conflict binding not found");

    assert_eq!(conflict_binding.source_kind, DeerFlowSourceKind::Conflict);
}

#[test]
fn backpressure_binds_to_a_backpressure_event_and_b_backpressure() {
    let fixture = include_str!("fixtures/deerflow_storage_backpressure.json");
    let bindings = bind_deerflow_fixtures(fixture).unwrap();

    let families: Vec<&str> = bindings.iter().map(|b| b.target_family.as_str()).collect();
    assert!(families.contains(&"a_backpressure_event"));
    assert!(families.contains(&"b_backpressure"));
}

#[test]
fn replay_checkpoint_binds_to_a_replay_checkpoint_and_b_replay_checkpoint() {
    let fixture = include_str!("fixtures/deerflow_replay_checkpoint.json");
    let bindings = bind_deerflow_fixtures(fixture).unwrap();

    let families: Vec<&str> = bindings.iter().map(|b| b.target_family.as_str()).collect();
    assert!(families.contains(&"a_replay_checkpoint"));
    assert!(families.contains(&"b_replay_checkpoint"));
}

#[test]
fn all_bindings_reference_shared_families_not_deerflow_only() {
    let fixtures = vec![
        include_str!("fixtures/deerflow_thread_snapshot.json"),
        include_str!("fixtures/deerflow_live_activity.json"),
        include_str!("fixtures/deerflow_replay_window.json"),
        include_str!("fixtures/deerflow_intent.json"),
        include_str!("fixtures/deerflow_artifact_preview.json"),
        include_str!("fixtures/deerflow_exclusion_intent.json"),
        include_str!("fixtures/deerflow_conflicting_intents.json"),
        include_str!("fixtures/deerflow_storage_backpressure.json"),
        include_str!("fixtures/deerflow_replay_checkpoint.json"),
    ];

    let contract = StorageContract::default();

    for fixture in fixtures {
        let bindings = bind_deerflow_fixtures(fixture).unwrap();
        for binding in &bindings {
            // Every target family must exist in the shared contract
            assert!(
                contract.get_family(&binding.target_family).is_some(),
                "DeerFlow-only family leaked: {}",
                binding.target_family
            );
        }
    }
}

#[test]
fn all_bindings_include_lineage_and_sequence() {
    let fixture = include_str!("fixtures/deerflow_live_activity.json");
    let bindings = bind_deerflow_fixtures(fixture).unwrap();

    for binding in &bindings {
        assert!(!binding.lineage_anchor.is_empty());
        assert!(binding.sequence_id > 0);
    }
}

#[test]
fn no_consumer_c_l2_rows_emitted_from_raw_source() {
    let fixtures = vec![
        include_str!("fixtures/deerflow_thread_snapshot.json"),
        include_str!("fixtures/deerflow_live_activity.json"),
    ];

    for fixture in fixtures {
        let bindings = bind_deerflow_fixtures(fixture).unwrap();
        for binding in &bindings {
            assert!(
                !binding.target_family.starts_with("c_l2_"),
                "C:L2 row leaked from raw source: {}",
                binding.target_family
            );
        }
    }
}
```

- [ ] **Step 3: Run test to confirm failure**

Run: `cargo test -p deer-pipeline-raw-sources --test deerflow_adapter -v`
Expected: FAIL — module `deerflow` does not exist

- [ ] **Step 4: Implement deerflow.rs**

Create `crates/pipeline/raw_sources/src/deerflow.rs`:

```rust
use serde::Deserialize;

use crate::error::RawSourceError;

/// DeerFlow source kind labels
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeerFlowSourceKind {
    ThreadSnapshot,
    LiveActivity,
    ReplayWindow,
    Intent,
    ArtifactAccess,
    Exclusion,
    Conflict,
    Backpressure,
    ReplayCheckpoint,
}

/// A single DeerFlow-to-A/B family binding
#[derive(Clone, Debug)]
pub struct DeerFlowBinding {
    pub generator_key: String,
    pub source_kind: DeerFlowSourceKind,
    pub target_family: String,
    pub immutable_keys: Vec<String>,
    pub lineage_anchor: String,
    pub sequence_id: u64,
    pub correlation_fields: Vec<String>,
}

/// Raw DeerFlow fixture envelope
#[derive(Debug, Deserialize)]
struct DeerFlowRawFixture {
    generator: String,
    source_kind: String,
    #[serde(flatten)]
    payload: serde_json::Value,
}

/// Bind a DeerFlow fixture JSON string to shared A/B family ids
pub fn bind_deerflow_fixtures(
    fixture_json: &str,
) -> Result<Vec<DeerFlowBinding>, RawSourceError> {
    let raw: DeerFlowRawFixture =
        serde_json::from_str(fixture_json).map_err(|_| RawSourceError::InvalidFixture)?;

    let source_kind = parse_source_kind(&raw.source_kind)?;
    let mut bindings = Vec::new();
    let mut seq = 0u64;

    match source_kind {
        DeerFlowSourceKind::ThreadSnapshot => {
            seq += 1;
            bindings.push(DeerFlowBinding {
                generator_key: "deerflow".into(),
                source_kind: source_kind.clone(),
                target_family: "a_session_snapshot".into(),
                immutable_keys: vec![
                    raw.payload["session_id"].as_str().unwrap_or("").into(),
                    raw.payload["thread_id"].as_str().unwrap_or("").into(),
                ],
                lineage_anchor: raw.payload["thread_id"].as_str().unwrap_or("").into(),
                sequence_id: seq,
                correlation_fields: vec!["thread_id".into(), "session_id".into(), "run_id".into()],
            });
        }
        DeerFlowSourceKind::LiveActivity => {
            if let Some(events) = raw.payload["events"].as_array() {
                for event in events {
                    seq += 1;
                    let kind = event["kind"].as_str().unwrap_or("");
                    let target = match kind {
                        "message" => "a_message_event",
                        "task_progress" => "a_task_event",
                        "artifact_presented" => "a_artifact_event",
                        "runtime_status" => "a_runtime_event",
                        _ => continue,
                    };
                    let event_id = event["event_id"].as_str().unwrap_or("");
                    bindings.push(DeerFlowBinding {
                        generator_key: "deerflow".into(),
                        source_kind: source_kind.clone(),
                        target_family: target.into(),
                        immutable_keys: vec![event_id.into()],
                        lineage_anchor: event_id.into(),
                        sequence_id: seq,
                        correlation_fields: vec![
                            "thread_id".into(),
                            "run_id".into(),
                        ],
                    });
                }
            }
        }
        DeerFlowSourceKind::ReplayWindow => {
            seq += 1;
            bindings.push(DeerFlowBinding {
                generator_key: "deerflow".into(),
                source_kind: source_kind.clone(),
                target_family: "b_replay_window".into(),
                immutable_keys: vec![
                    raw.payload["window_id"].as_str().unwrap_or("").into(),
                    raw.payload["thread_id"].as_str().unwrap_or("").into(),
                ],
                lineage_anchor: raw.payload["window_id"].as_str().unwrap_or("").into(),
                sequence_id: seq,
                correlation_fields: vec!["thread_id".into(), "run_id".into()],
            });
        }
        DeerFlowSourceKind::Intent => {
            seq += 1;
            let intent_id = raw.payload["intent_id"].as_str().unwrap_or("");
            bindings.push(DeerFlowBinding {
                generator_key: "deerflow".into(),
                source_kind: source_kind.clone(),
                target_family: "a_intent_event".into(),
                immutable_keys: vec![intent_id.into()],
                lineage_anchor: intent_id.into(),
                sequence_id: seq,
                correlation_fields: vec!["intent_id".into(), "thread_id".into(), "actor_id".into()],
            });
            seq += 1;
            bindings.push(DeerFlowBinding {
                generator_key: "deerflow".into(),
                source_kind: source_kind.clone(),
                target_family: "b_intent".into(),
                immutable_keys: vec![intent_id.into()],
                lineage_anchor: intent_id.into(),
                sequence_id: seq,
                correlation_fields: vec!["intent_id".into(), "thread_id".into(), "actor_id".into()],
            });
        }
        DeerFlowSourceKind::ArtifactAccess => {
            seq += 1;
            let artifact_id = raw.payload["artifact_id"].as_str().unwrap_or("");
            bindings.push(DeerFlowBinding {
                generator_key: "deerflow".into(),
                source_kind: source_kind.clone(),
                target_family: "a_artifact_event".into(),
                immutable_keys: vec![artifact_id.into()],
                lineage_anchor: artifact_id.into(),
                sequence_id: seq,
                correlation_fields: vec!["artifact_id".into(), "task_id".into(), "run_id".into()],
            });
            seq += 1;
            let access_id = raw.payload["access_id"].as_str().unwrap_or("");
            bindings.push(DeerFlowBinding {
                generator_key: "deerflow".into(),
                source_kind: source_kind.clone(),
                target_family: "b_artifact_access".into(),
                immutable_keys: vec![artifact_id.into(), access_id.into()],
                lineage_anchor: access_id.into(),
                sequence_id: seq,
                correlation_fields: vec!["artifact_id".into(), "access_id".into()],
            });
        }
        DeerFlowSourceKind::Exclusion => {
            seq += 1;
            let exclusion_id = raw.payload["exclusion_id"].as_str().unwrap_or("");
            bindings.push(DeerFlowBinding {
                generator_key: "deerflow".into(),
                source_kind: source_kind.clone(),
                target_family: "a_exclusion_event".into(),
                immutable_keys: vec![exclusion_id.into()],
                lineage_anchor: exclusion_id.into(),
                sequence_id: seq,
                correlation_fields: vec!["exclusion_id".into(), "target_id".into(), "thread_id".into()],
            });
            seq += 1;
            bindings.push(DeerFlowBinding {
                generator_key: "deerflow".into(),
                source_kind: source_kind.clone(),
                target_family: "b_exclusion".into(),
                immutable_keys: vec![exclusion_id.into()],
                lineage_anchor: exclusion_id.into(),
                sequence_id: seq,
                correlation_fields: vec!["exclusion_id".into(), "target_id".into()],
            });
        }
        DeerFlowSourceKind::Conflict => {
            seq += 1;
            let conflict_id = raw.payload["conflict_id"].as_str().unwrap_or("");
            bindings.push(DeerFlowBinding {
                generator_key: "deerflow".into(),
                source_kind: source_kind.clone(),
                target_family: "b_conflict".into(),
                immutable_keys: vec![conflict_id.into()],
                lineage_anchor: conflict_id.into(),
                sequence_id: seq,
                correlation_fields: vec!["conflict_id".into(), "target_id".into(), "thread_id".into()],
            });
        }
        DeerFlowSourceKind::Backpressure => {
            seq += 1;
            let bp_id = raw.payload["backpressure_id"].as_str().unwrap_or("");
            bindings.push(DeerFlowBinding {
                generator_key: "deerflow".into(),
                source_kind: source_kind.clone(),
                target_family: "a_backpressure_event".into(),
                immutable_keys: vec![bp_id.into()],
                lineage_anchor: bp_id.into(),
                sequence_id: seq,
                correlation_fields: vec!["backpressure_id".into(), "run_id".into()],
            });
            seq += 1;
            bindings.push(DeerFlowBinding {
                generator_key: "deerflow".into(),
                source_kind: source_kind.clone(),
                target_family: "b_backpressure".into(),
                immutable_keys: vec![bp_id.into()],
                lineage_anchor: bp_id.into(),
                sequence_id: seq,
                correlation_fields: vec!["backpressure_id".into(), "run_id".into()],
            });
        }
        DeerFlowSourceKind::ReplayCheckpoint => {
            seq += 1;
            let cp_id = raw.payload["checkpoint_id"].as_str().unwrap_or("");
            bindings.push(DeerFlowBinding {
                generator_key: "deerflow".into(),
                source_kind: source_kind.clone(),
                target_family: "a_replay_checkpoint".into(),
                immutable_keys: vec![cp_id.into()],
                lineage_anchor: cp_id.into(),
                sequence_id: seq,
                correlation_fields: vec!["checkpoint_id".into(), "run_id".into()],
            });
            seq += 1;
            bindings.push(DeerFlowBinding {
                generator_key: "deerflow".into(),
                source_kind: source_kind.clone(),
                target_family: "b_replay_checkpoint".into(),
                immutable_keys: vec![cp_id.into()],
                lineage_anchor: cp_id.into(),
                sequence_id: seq,
                correlation_fields: vec!["checkpoint_id".into(), "run_id".into()],
            });
        }
    }

    Ok(bindings)
}

fn parse_source_kind(value: &str) -> Result<DeerFlowSourceKind, RawSourceError> {
    match value {
        "thread_snapshot" => Ok(DeerFlowSourceKind::ThreadSnapshot),
        "live_activity" => Ok(DeerFlowSourceKind::LiveActivity),
        "replay_window" => Ok(DeerFlowSourceKind::ReplayWindow),
        "intent" => Ok(DeerFlowSourceKind::Intent),
        "artifact_access" => Ok(DeerFlowSourceKind::ArtifactAccess),
        "exclusion" => Ok(DeerFlowSourceKind::Exclusion),
        "conflict" => Ok(DeerFlowSourceKind::Conflict),
        "backpressure" => Ok(DeerFlowSourceKind::Backpressure),
        "replay_checkpoint" => Ok(DeerFlowSourceKind::ReplayCheckpoint),
        _ => Err(RawSourceError::InvalidFixture),
    }
}
```

- [ ] **Step 5: Update lib.rs to export deerflow module**

Modify `crates/pipeline/raw_sources/src/lib.rs`:

```rust
pub mod artifact_gateway;
pub mod c_query_contract;
pub mod deerflow;
pub mod error;
pub mod run_stream;
pub mod storage_contract;
pub mod thread_gateway;
pub mod upload_gateway;

pub use artifact_gateway::{preview_artifact, ArtifactAccess};
pub use c_query_contract::{C2ViewContract, CQueryContract, ConsumerShell, ViewKind};
pub use deerflow::{bind_deerflow_fixtures, DeerFlowBinding, DeerFlowSourceKind};
pub use run_stream::{load_stream_fixture, AdapterEvent, RawStreamEvent};
pub use storage_contract::{
    AFamily, BFamily, CanonicalLevel, FamilyId, FamilyMeta, Hierarchy, StorageContract,
};
pub use thread_gateway::{create_thread, resume_thread, ThreadSnapshot};
pub use upload_gateway::{stage_upload, UploadReceipt};
```

- [ ] **Step 6: Run tests to verify pass**

Run: `cargo test -p deer-pipeline-raw-sources --test deerflow_adapter -v`
Expected: PASS

---

## Planning Answers (per spec 10 guardrails)

| # | Question | Answer |
|---|----------|--------|
| 1 | Owning layer or crate | `raw_sources` |
| 2 | Contract being defined | DeerFlow-to-A/B family binding map (9 source kinds → shared families) |
| 3 | Failing test or fixture first | `deerflow_adapter.rs` — 13 tests across 9 fixtures, asserting correct families, lineage, no C:L2 leak |
| 4 | Proof app before `deer_gui` | `deer_chat_lab` — load fixture, call `bind_deerflow_fixtures()`, assert bindings |
| 5 | Forbidden dependencies | `normalizers`, `deer_gui`, `world_projection`, any view crate |
| 6 | Milestone gate unlocked | Gate A — first generator binds to shared contract |
