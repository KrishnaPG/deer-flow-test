use std::path::PathBuf;

use deer_foundation_contracts::{
    EventId, RecordFamily, RecordId, RecordRef, ReplayEnvelope, WriteOperation, WriteOperationKind,
};
use deer_foundation_domain::{AnyRecord, RunBody, RunRecord};
use deer_foundation_replay::{ReplayFixture, ReplayLog};

#[test]
fn fixture_log_replays_in_strict_sequence() {
    let fixture_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/foundation_spine_log.json");
    let fixture = ReplayFixture::load(&fixture_path).unwrap();
    let log = fixture.into_log().unwrap();

    assert_eq!(log.entries().len(), 2);
    assert_eq!(
        log.after(Some(&log.entries()[0].envelope.cursor())).len(),
        1
    );
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
            WriteOperation::new(
                WriteOperationKind::AppendRecord,
                RecordId::from_static("run_1"),
            ),
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
                WriteOperation::new(
                    WriteOperationKind::AppendRecord,
                    RecordId::from_static("run_1"),
                ),
                None,
                chrono::Utc::now(),
            ),
            first_record,
        )
        .unwrap_err();

    assert!(error.to_string().contains("strictly increasing"));
}
