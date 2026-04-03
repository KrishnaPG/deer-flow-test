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
    assert!(!game_face.knowledge_graph.nodes.is_empty());
    assert!(matches!(game_face.telemetry.queue_pressure.pressure_level, "low" | "moderate" | "high"));
    assert!(game_face.telemetry.event_stories.iter().any(|s| s.contains("artifact")));
    assert!(game_face.artifact_families.iter().any(|family| family.family == ArtifactFamilyVm::Text));
    assert!(game_face.interventions.iter().any(|vm| vm.kind == "operator_override"));
    assert_eq!(game_face.session.state, "running");
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UnitActorVm {
    pub agent_id: String,
    pub source_record_id: String,
    pub status: String,
    pub panel_target: &'static str,
    pub health: AgentHealthVm,
    pub performance: AgentPerformanceVm,
    pub track_record: AgentTrackRecordVm,
}

#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct AgentHealthVm {
    pub current_status: String,
    pub fatigue_level: f32,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct AgentPerformanceVm {
    pub success_rate: f32,
    pub avg_completion_time_ms: Option<u64>,
    pub current_streak: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct AgentTrackRecordVm {
    pub total_tasks: usize,
    pub total_artifacts: usize,
    pub veteran_tier: &'static str,
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
    pub priority: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HistoryRowVm {
    pub event_id: String,
    pub source_record_id: String,
    pub label: String,
    pub event_story: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KnowledgeNodeVm {
    pub node_id: String,
    pub label: String,
    pub source_record_id: String,
    pub links: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ArtifactFamilyVm {
    Text,
    Code,
    Dataset,
    Image,
    Audio,
    Video,
    Model,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArtifactFamilyEntryVm {
    pub source_record_id: String,
    pub family: ArtifactFamilyVm,
    pub media_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InterventionVm {
    pub source_record_id: String,
    pub kind: &'static str,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SessionStateVm {
    pub thread_id: String,
    pub run_id: Option<String>,
    pub state: String,
    pub branch_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TelemetryVm {
    pub active_agents: usize,
    pub running_tasks: usize,
    pub artifact_count: usize,
    pub queue_pressure: QueuePressureVm,
    pub event_stories: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QueuePressureVm {
    pub queued_count: usize,
    pub running_count: usize,
    pub pressure_level: &'static str,
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
pub struct KnowledgeGraphVm {
    pub nodes: Vec<KnowledgeNodeVm>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GameFaceVm {
    pub units: Vec<UnitActorVm>,
    pub task_forces: Vec<TaskForceLinkVm>,
    pub queue: QueueVm,
    pub history: HistoryVm,
    pub knowledge_graph: KnowledgeGraphVm,
    pub artifact_families: Vec<ArtifactFamilyEntryVm>,
    pub interventions: Vec<InterventionVm>,
    pub session: SessionStateVm,
    pub telemetry: TelemetryVm,
}

pub fn derive_game_face_vm(records: &[AnyRecord]) -> GameFaceVm {
    let mut units = Vec::new();
    let mut task_forces = Vec::new();
    let mut queue_rows = Vec::new();
    let mut history_rows = Vec::new();
    let mut knowledge_nodes = Vec::new();
    let mut artifact_families = Vec::new();
    let mut interventions = Vec::new();
    let mut agent_ids = BTreeSet::new();
    let mut artifact_count = 0usize;
    let mut running_count = 0usize;
    let mut queued_count = 0usize;
    let mut agent_task_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut agent_artifact_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut event_stories = Vec::new();
    let mut session_state = SessionStateVm {
        thread_id: "unknown".into(),
        run_id: None,
        state: "unknown".into(),
        branch_label: "main".into(),
    };

    for record in records {
        let header = record.header();
        if let Some(agent_id) = header.correlations.agent_id.as_ref() {
            agent_ids.insert(agent_id.to_string());
        }
        if let Some(thread_id) = header.correlations.thread_id.as_ref() {
            session_state.thread_id = thread_id.to_string();
        }
        if let Some(run_id) = header.correlations.run_id.as_ref() {
            session_state.run_id = Some(run_id.to_string());
        }

        match record {
            AnyRecord::Session(session) => {
                session_state.state = session.body.status.clone();
            }
            AnyRecord::Task(task) => {
                let agent_id = header.correlations.agent_id.as_ref().map(|id| id.to_string()).unwrap_or_else(|| "unassigned".into());
                *agent_task_counts.entry(agent_id.clone()).or_default() += 1;

                let is_running = task.body.status == "running";
                if is_running {
                    running_count += 1;
                } else {
                    queued_count += 1;
                }

                units.push(UnitActorVm {
                    agent_id: agent_id.clone(),
                    source_record_id: task.record_id().to_string(),
                    status: task.body.status.clone(),
                    panel_target: "task_detail",
                    health: AgentHealthVm {
                        current_status: if is_running { "active".into() } else { "idle".into() },
                        fatigue_level: 0.0,
                        last_error: None,
                    },
                    performance: AgentPerformanceVm::default(),
                    track_record: AgentTrackRecordVm {
                        total_tasks: *agent_task_counts.get(&agent_id).unwrap_or(&0),
                        total_artifacts: *agent_artifact_counts.get(&agent_id).unwrap_or(&0),
                        veteran_tier: if *agent_task_counts.get(&agent_id).unwrap_or(&0) >= 10 { "veteran" } else { "rookie" },
                    },
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
                    priority: None,
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

                if let Some(agent_id) = header.correlations.agent_id.as_ref() {
                    *agent_artifact_counts.entry(agent_id.to_string()).or_default() += 1;
                }

                history_rows.push(HistoryRowVm {
                    event_id: event_id.clone(),
                    source_record_id: artifact.record_id().to_string(),
                    label: artifact.body.label.clone(),
                    event_story: Some(format!("Artifact '{}' produced", artifact.body.label)),
                });

                knowledge_nodes.push(KnowledgeNodeVm {
                    node_id: artifact.record_id().to_string(),
                    label: artifact.body.label.clone(),
                    source_record_id: artifact.record_id().to_string(),
                    links: artifact.header.lineage.source_events.iter().map(|e| e.to_string()).collect(),
                });

                artifact_families.push(ArtifactFamilyEntryVm {
                    source_record_id: artifact.record_id().to_string(),
                    family: classify_artifact_family(artifact.body.media_type.as_deref().unwrap_or("application/octet-stream")),
                    media_type: artifact.body.media_type.clone().unwrap_or_else(|| "application/octet-stream".into()),
                });

                event_stories.push(format!("artifact:{}", artifact.body.label));
            }
            AnyRecord::Intent(intent) => {
                interventions.push(InterventionVm {
                    source_record_id: intent.record_id().to_string(),
                    kind: "operator_override",
                    status: intent.body.action.clone(),
                });
            }
            _ => {}
        }
    }

    let pressure_level = if queued_count > 10 { "high" } else if queued_count > 3 { "moderate" } else { "low" };

    GameFaceVm {
        units,
        task_forces,
        queue: QueueVm { rows: queue_rows },
        history: HistoryVm { rows: history_rows },
        knowledge_graph: KnowledgeGraphVm { nodes: knowledge_nodes },
        artifact_families,
        interventions,
        session: session_state,
        telemetry: TelemetryVm {
            active_agents: agent_ids.len(),
            running_tasks: running_count,
            artifact_count,
            queue_pressure: QueuePressureVm {
                queued_count,
                running_count,
                pressure_level,
            },
            event_stories,
        },
    }
}

fn classify_artifact_family(media_type: &str) -> ArtifactFamilyVm {
    match media_type {
        mt if mt.starts_with("text/") => ArtifactFamilyVm::Text,
        "application/json" | "application/yaml" | "text/x-rust" | "text/x-python" => ArtifactFamilyVm::Code,
        "application/parquet" | "text/csv" => ArtifactFamilyVm::Dataset,
        mt if mt.starts_with("image/") => ArtifactFamilyVm::Image,
        mt if mt.starts_with("audio/") => ArtifactFamilyVm::Audio,
        mt if mt.starts_with("video/") => ArtifactFamilyVm::Video,
        "application/octet-stream+model" => ArtifactFamilyVm::Model,
        _ => ArtifactFamilyVm::Unknown,
    }
}
```

```rust
// crates/pipeline/derivations/src/lib.rs
pub mod game_face;

pub use game_face::{
    derive_game_face_vm, AgentHealthVm, AgentPerformanceVm, AgentTrackRecordVm, ArtifactFamilyEntryVm,
    ArtifactFamilyVm, GameFaceVm, HistoryRowVm, InterventionVm, KnowledgeGraphVm, KnowledgeNodeVm,
    QueuePressureVm, QueueRowVm, SessionStateVm, TaskForceLinkVm, TelemetryVm, UnitActorVm,
};
```

- [ ] **Step 4: Re-run the DeerFlow game-facing derivation test**

Run: `cargo test -p deer-pipeline-derivations --test deerflow_game_face -v`

Expected: PASS with stable unit, relationship, queue, history, and telemetry derivations.

- [ ] **Step 5: Commit the game-facing derivation layer**

```bash
git add crates/pipeline/derivations/src/lib.rs crates/pipeline/derivations/src/game_face.rs crates/pipeline/derivations/tests/deerflow_game_face.rs
git commit -m "feat: derive deerflow game-facing projections"
```
