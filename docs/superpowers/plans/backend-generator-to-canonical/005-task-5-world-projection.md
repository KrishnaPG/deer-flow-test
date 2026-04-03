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
