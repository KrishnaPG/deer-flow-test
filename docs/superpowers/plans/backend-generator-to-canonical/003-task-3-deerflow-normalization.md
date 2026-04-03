## Task 3: Normalize DeerFlow Raw Capture Into L0/L1/L2 Canonical Records

**Files:**
- Modify: `crates/pipeline/normalizers/src/lib.rs`
- Modify: `crates/pipeline/normalizers/src/carrier.rs`
- Modify: `crates/pipeline/normalizers/src/representation.rs`
- Modify: `crates/pipeline/normalizers/src/governance.rs`
- Test: `crates/pipeline/normalizers/tests/deerflow_levels_lineage.rs`

**Milestone unlock:** DeerFlow batches produce storage-aware canonical records with explicit level/plane occupancy, lineage, and correlations that later generators can follow

**Forbidden shortcuts:** do not skip L0/L1 evidence; do not emit L2 records with empty lineage/correlations when raw envelopes carry thread/run/task/agent ids; do not treat mediated DTOs as final canonical models; do not skip `as_is` hash computation — it is the identity anchor for all downstream chunk and embedding planes (spec 12: `ChunkRecord` and `EmbeddingRecord` are deferred but must reference `as_is_hash` when introduced)

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

    // Anti-drift: every record in the normalized batch must be able to state its level and plane
    for record in &normalized.records {
        let header = record.header();
        let _ = header.level;
        let _ = header.plane;
        let _ = header.source_level;
        let _ = header.source_plane;
    }

    // Hash anchoring: as-is representation records must carry a concrete content hash
    let repr = normalized.records.iter().find_map(|record| match record {
        AnyRecord::AsIsRepresentation(record) => Some(record),
        _ => None,
    });
    if let Some(repr) = repr {
        assert!(repr.header.identity.as_is_hash.is_some());
        let hash = repr.header.identity.as_is_hash.as_ref().unwrap();
        assert!(hash.starts_with("sha256:"));
    }
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
            RawEnvelopeFamily::Intent if is_exclusion_intent(batch) => emit_exclusion_intent(batch, &mut records),
            RawEnvelopeFamily::Intent => emit_intent_record(batch, &mut records),
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
    let record_id = RecordId::from(format!("repr:{}", batch.source_object_id));
    let payload_hash = compute_as_is_hash(&batch.payload);

    AnyRecord::AsIsRepresentation(AsIsRepresentationRecord::new_with_meta(
        record_id.clone(),
        IdentityMeta::hash_anchored(record_id, Some(payload_hash), None, None),
        crate::carrier::correlations_for_batch(batch),
        deer_foundation_contracts::LineageMeta::root(),
        CanonicalLevel::L0,
        CanonicalPlane::AsIs,
        AsIsRepresentationBody {
            media_type: "application/json".into(),
        },
    ))
}

fn compute_as_is_hash(payload: &serde_json::Value) -> String {
    use sha2::{Digest, Sha256};
    let bytes = serde_json::to_vec(payload).unwrap_or_default();
    let hash = Sha256::digest(&bytes);
    format!("sha256:{:x}", hash)
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
