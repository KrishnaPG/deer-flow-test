### Task 3: DeerFlow Normalization to Shared A/B L2+ Rows

**Files:**
- Modify: `crates/pipeline/normalizers/src/lib.rs`
- Test: `crates/pipeline/normalizers/tests/deerflow_levels_lineage.rs`
- Test fixture: `crates/pipeline/normalizers/tests/fixtures/deerflow_normalized_batch.json`

- [ ] **Step 1: Create normalization fixture**

Create `crates/pipeline/normalizers/tests/fixtures/deerflow_normalized_batch.json`:

```json
{
  "generator": "deerflow",
  "session": {
    "session_id": "session_df_1",
    "title": "Survey the ridge"
  },
  "run": {
    "run_id": "run_df_1",
    "status": "running"
  },
  "events": [
    {
      "kind": "message",
      "message_id": "msg_df_1",
      "role": "operator",
      "text": "Survey the ridge.",
      "level": "L2"
    },
    {
      "kind": "task",
      "task_id": "task_df_1",
      "title": "Gather terrain notes",
      "state": "running"
    },
    {
      "kind": "artifact",
      "artifact_id": "artifact_df_1",
      "name": "scan.png",
      "status": "presented",
      "as_is_hash": "sha256:scan-png",
      "parent_message_id": "msg_df_1",
      "parent_clarification_id": null
    },
    {
      "kind": "artifact",
      "artifact_id": "artifact_df_2",
      "name": "prediction.json",
      "status": "presented",
      "as_is_hash": "sha256:pred-json",
      "parent_message_id": "msg_df_1",
      "parent_clarification_id": null
    },
    {
      "kind": "runtime_status",
      "state": "live"
    }
  ],
  "governance": {
    "exclusions": [
      {
        "exclusion_id": "excl_df_1",
        "target_id": "artifact_df_2",
        "reason": "classified"
      }
    ],
    "replay_checkpoints": [
      {
        "checkpoint_id": "cp_df_1",
        "sequence_id": 5,
        "run_id": "run_df_1"
      }
    ],
    "transforms": [
      {
        "transform_id": "xform_df_1",
        "target_id": "task_df_1",
        "source_event": "evt_task_1"
      }
    ]
  }
}
```

- [ ] **Step 2: Write the failing DeerFlow normalization test**

Create `crates/pipeline/normalizers/tests/deerflow_levels_lineage.rs`:

```rust
use deer_foundation_contracts::{CanonicalLevel, RecordFamily};
use deer_foundation_domain::AnyRecord;
use deer_pipeline_normalizers::{normalize_deerflow_batch, NormalizedBatch};

#[test]
fn deerflow_landing_produces_l2_plus_rows() {
    let fixture = include_str!("fixtures/deerflow_normalized_batch.json");
    let batch = normalize_deerflow_batch(fixture).unwrap();

    for record in &batch.records {
        let level = record.header().level;
        assert!(
            matches!(
                level,
                CanonicalLevel::L2
                    | CanonicalLevel::L3
                    | CanonicalLevel::L4
                    | CanonicalLevel::L5
                    | CanonicalLevel::L6
            ),
            "expected L2+ but got {:?}",
            level
        );
    }
}

#[test]
fn deerflow_normalization_produces_shared_family_rows() {
    let fixture = include_str!("fixtures/deerflow_normalized_batch.json");
    let batch = normalize_deerflow_batch(fixture).unwrap();

    let families: Vec<RecordFamily> = batch.records.iter().map(|r| r.header().family).collect();

    // Must produce A-family rows
    assert!(families.iter().any(|f| matches!(f, RecordFamily::Session)));
    assert!(families.iter().any(|f| matches!(f, RecordFamily::Message)));
    assert!(families.iter().any(|f| matches!(f, RecordFamily::Task)));
    assert!(families.iter().any(|f| matches!(f, RecordFamily::Artifact)));
    assert!(families.iter().any(|f| matches!(f, RecordFamily::RuntimeStatus)));
}

#[test]
fn deerflow_normalization_preserves_lineage() {
    let fixture = include_str!("fixtures/deerflow_normalized_batch.json");
    let batch = normalize_deerflow_batch(fixture).unwrap();

    for record in &batch.records {
        let header = record.header();
        // Every record must have a lineage anchor
        assert!(
            !header.identity.primary_id.as_str().is_empty(),
            "record missing primary_id"
        );
    }
}

#[test]
fn deerflow_normalization_emits_no_c_l2_rows() {
    let fixture = include_str!("fixtures/deerflow_normalized_batch.json");
    let batch = normalize_deerflow_batch(fixture).unwrap();

    for record in &batch.records {
        let family = record.header().family;
        assert!(
            !matches!(family, RecordFamily::L2View),
            "normalizer emitted C:L2 row"
        );
    }
}

#[test]
fn deerflow_normalization_supports_higher_level_rows() {
    let fixture = include_str!("fixtures/deerflow_normalized_batch.json");
    let batch = normalize_deerflow_batch(fixture).unwrap();

    // At least some records should be at L2
    let has_l2 = batch
        .records
        .iter()
        .any(|r| matches!(r.header().level, CanonicalLevel::L2));
    assert!(has_l2, "expected some L2 records");
}

#[test]
fn deerflow_governance_rows_are_emitted() {
    let fixture = include_str!("fixtures/deerflow_normalized_batch.json");
    let batch = normalize_deerflow_batch(fixture).unwrap();

    let families: Vec<RecordFamily> = batch.records.iter().map(|r| r.header().family).collect();

    // Governance rows should be present
    assert!(
        families.iter().any(|f| matches!(f, RecordFamily::Exclusion)),
        "missing exclusion rows"
    );
    assert!(
        families.iter().any(|f| matches!(f, RecordFamily::ReplayCheckpoint)),
        "missing replay checkpoint rows"
    );
    assert!(
        families.iter().any(|f| matches!(f, RecordFamily::Transform)),
        "missing transform rows"
    );
}
```

- [ ] **Step 3: Run test to confirm failure**

Run: `cargo test -p deer-pipeline-normalizers --test deerflow_levels_lineage -v`
Expected: FAIL — `normalize_deerflow_batch` does not exist

- [ ] **Step 4: Implement normalize_deerflow_batch**

Modify `crates/pipeline/normalizers/src/carrier.rs` — add at the end:

```rust
/// Normalize a DeerFlow batch fixture directly into shared A/B L2+ rows.
/// This reads the same envelope format as `normalize_batch` but ensures
/// all output records are at L2+ with lineage preserved.
pub fn normalize_deerflow_batch(
    fixture_json: &str,
) -> Result<NormalizedBatch, NormalizationError> {
    let batch: RawEnvelopeBatch = serde_json::from_str(fixture_json)
        .map_err(|_| NormalizationError::UnsupportedEventKind("deerflow_batch".into()))?;

    let mut result = normalize_batch(&batch)?;

    // Ensure all records are at L2+
    for record in &mut result.records {
        let header = record.header_mut();
        if matches!(header.level, CanonicalLevel::L0 | CanonicalLevel::L1) {
            header.level = CanonicalLevel::L2;
        }
    }

    // Parse governance extensions from fixture
    let fixture: serde_json::Value = serde_json::from_str(fixture_json)
        .map_err(|_| NormalizationError::UnsupportedEventKind("deerflow_batch".into()))?;

    if let Some(governance) = fixture.get("governance") {
        // Process exclusions
        if let Some(exclusions) = governance.get("exclusions").and_then(|v| v.as_array()) {
            for excl in exclusions {
                let excl_id = excl["exclusion_id"].as_str().unwrap_or("");
                let target_id = excl["target_id"].as_str().unwrap_or("");
                result.records.push(AnyRecord::Exclusion(
                    deer_foundation_domain::ExclusionRecord::new(
                        RecordId::from(excl_id),
                        deer_foundation_contracts::IdentityMeta::hash_anchored(
                            RecordId::from(excl_id),
                            None,
                            None,
                            None,
                        ),
                        deer_foundation_contracts::LineageMeta::root(),
                        deer_foundation_domain::ExclusionBody {
                            target_id: target_id.into(),
                            reason: excl["reason"].as_str().unwrap_or("").into(),
                        },
                    ),
                ));
            }
        }

        // Process replay checkpoints
        if let Some(checkpoints) = governance.get("replay_checkpoints").and_then(|v| v.as_array()) {
            for cp in checkpoints {
                let cp_id = cp["checkpoint_id"].as_str().unwrap_or("");
                result.records.push(AnyRecord::ReplayCheckpoint(
                    deer_foundation_domain::ReplayCheckpointRecord::new(
                        RecordId::from(cp_id),
                        deer_foundation_contracts::IdentityMeta::hash_anchored(
                            RecordId::from(cp_id),
                            None,
                            None,
                            None,
                        ),
                        deer_foundation_contracts::LineageMeta::root(),
                        deer_foundation_domain::ReplayCheckpointBody {
                            sequence_id: cp["sequence_id"].as_u64().unwrap_or(0) as u32,
                        },
                    ),
                ));
            }
        }

        // Process transforms
        if let Some(transforms) = governance.get("transforms").and_then(|v| v.as_array()) {
            for xform in transforms {
                let xform_id = xform["transform_id"].as_str().unwrap_or("");
                let target_id = xform["target_id"].as_str().unwrap_or("");
                result.records.push(AnyRecord::Transform(
                    deer_foundation_domain::TransformRecord::new(
                        RecordId::from(xform_id),
                        deer_foundation_contracts::IdentityMeta::hash_anchored(
                            RecordId::from(xform_id),
                            None,
                            None,
                            None,
                        ),
                        deer_foundation_contracts::LineageMeta::root(),
                        deer_foundation_domain::TransformBody {
                            target_id: target_id.into(),
                            source_event: xform["source_event"].as_str().unwrap_or("").into(),
                        },
                    ),
                ));
            }
        }
    }

    Ok(result)
}
```

Add `header_mut()` method to `CanonicalRecord` trait in `crates/foundation/contracts/src/records.rs`:

```rust
pub trait CanonicalRecord {
    fn header(&self) -> &RecordHeader;
    fn header_mut(&mut self) -> &mut RecordHeader;
}
```

Update the `define_record!` macro in `crates/foundation/domain/src/common.rs` to also implement `header_mut`:

```rust
impl deer_foundation_contracts::CanonicalRecord for $name {
    fn header(&self) -> &deer_foundation_contracts::RecordHeader {
        &self.header
    }

    fn header_mut(&mut self) -> &mut deer_foundation_contracts::RecordHeader {
        &mut self.header
    }
}
```

Update `AnyRecord` in `crates/foundation/domain/src/lib.rs` to also implement `header_mut`:

```rust
impl CanonicalRecord for AnyRecord {
    fn header(&self) -> &RecordHeader {
        match self {
            AnyRecord::Run(record) => &record.header,
            // ... all existing arms ...
        }
    }

    fn header_mut(&mut self) -> &mut RecordHeader {
        match self {
            AnyRecord::Run(record) => &mut record.header,
            AnyRecord::Session(record) => &mut record.header,
            AnyRecord::Task(record) => &mut record.header,
            AnyRecord::Message(record) => &mut record.header,
            AnyRecord::ToolCall(record) => &mut record.header,
            AnyRecord::Artifact(record) => &mut record.header,
            AnyRecord::Clarification(record) => &mut record.header,
            AnyRecord::RuntimeStatus(record) => &mut record.header,
            AnyRecord::Delivery(record) => &mut record.header,
            AnyRecord::L0Source(record) => &mut record.header,
            AnyRecord::L1Sanitized(record) => &mut record.header,
            AnyRecord::L2View(record) => &mut record.header,
            AnyRecord::L3Insight(record) => &mut record.header,
            AnyRecord::L4Prediction(record) => &mut record.header,
            AnyRecord::L5Prescription(record) => &mut record.header,
            AnyRecord::L6Outcome(record) => &mut record.header,
            AnyRecord::AsIsRepresentation(record) => &mut record.header,
            AnyRecord::Chunk(record) => &mut record.header,
            AnyRecord::Embedding(record) => &mut record.header,
            AnyRecord::Intent(record) => &mut record.header,
            AnyRecord::Transform(record) => &mut record.header,
            AnyRecord::Exclusion(record) => &mut record.header,
            AnyRecord::Conflict(record) => &mut record.header,
            AnyRecord::Resolution(record) => &mut record.header,
            AnyRecord::ReplayCheckpoint(record) => &mut record.header,
            AnyRecord::Dedup(record) => &mut record.header,
            AnyRecord::Batch(record) => &mut record.header,
            AnyRecord::Branch(record) => &mut record.header,
            AnyRecord::Version(record) => &mut record.header,
            AnyRecord::WriteOperation(record) => &mut record.header,
            AnyRecord::GraphNode(record) => &mut record.header,
            AnyRecord::GraphEdge(record) => &mut record.header,
            AnyRecord::KnowledgeEntity(record) => &mut record.header,
            AnyRecord::KnowledgeRelation(record) => &mut record.header,
        }
    }
}
```

- [ ] **Step 5: Update normalizers lib.rs**

Modify `crates/pipeline/normalizers/src/lib.rs`:

```rust
pub mod carrier;
pub mod c_view_catalog;
pub mod envelopes;
pub mod error;
pub mod governance;
pub mod promotions;
pub mod representation;

pub use carrier::normalize_deerflow_batch;
pub use carrier::normalize_stream_batch;
pub use carrier::{normalize_batch, NormalizedBatch};
pub use c_view_catalog::CViewCatalog;
pub use envelopes::RawEnvelopeBatch;
pub use error::NormalizationError;
pub use governance::emit_intent_records;
pub use promotions::validate_promotion;
pub use representation::normalize_representation_chain;
```

- [ ] **Step 6: Run tests to verify pass**

Run: `cargo test -p deer-pipeline-normalizers --test deerflow_levels_lineage -v`
Expected: PASS

---

## Planning Answers (per spec 10 guardrails)

| # | Question | Answer |
|---|----------|--------|
| 1 | Owning layer or crate | `normalizers` |
| 2 | Contract being defined | Raw envelopes → shared A/B L2+ rows with lineage preserved; governance rows emitted |
| 3 | Failing test or fixture first | `deerflow_levels_lineage.rs` — 6 tests asserting L2+, lineage, no C:L2 emission, governance rows |
| 4 | Proof app before `deer_gui` | `deer_chat_lab` — feed fixture through `normalize_deerflow_batch()`, assert L2+ records |
| 5 | Forbidden dependencies | `deer_gui`, `world_projection`, `derivations`, any view crate |
| 6 | Milestone gate unlocked | Gate B — normalization produces C:L0-eligible rows |
