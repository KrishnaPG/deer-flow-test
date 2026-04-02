use deer_foundation_domain::AnyRecord;
use deer_pipeline_normalizers::{normalize_batch, RawEnvelopeBatch};

#[test]
fn normalizes_deerflow_events_into_canonical_records() {
    let fixture = include_str!("fixtures/deerflow_live_run.json");
    let batch: RawEnvelopeBatch = serde_json::from_str(fixture).unwrap();

    let normalized = normalize_batch(&batch).unwrap();

    assert!(normalized
        .records
        .iter()
        .any(|record| matches!(record, AnyRecord::Session(_))));
    assert!(normalized
        .records
        .iter()
        .any(|record| matches!(record, AnyRecord::Run(_))));
    assert!(normalized
        .records
        .iter()
        .any(|record| matches!(record, AnyRecord::Message(_))));
    assert!(normalized
        .records
        .iter()
        .any(|record| matches!(record, AnyRecord::Task(_))));
    assert!(normalized
        .records
        .iter()
        .any(|record| matches!(record, AnyRecord::ToolCall(_))));
    assert!(normalized
        .records
        .iter()
        .any(|record| matches!(record, AnyRecord::Artifact(_))));
    assert!(normalized
        .records
        .iter()
        .any(|record| matches!(record, AnyRecord::Clarification(_))));
    assert!(normalized
        .records
        .iter()
        .any(|record| matches!(record, AnyRecord::RuntimeStatus(_))));
    assert!(!normalized
        .records
        .iter()
        .any(|record| matches!(record, AnyRecord::Intent(_))));
}

#[test]
fn normalizes_hermes_events_into_canonical_records() {
    let fixture = include_str!("fixtures/hermes_run.json");
    let batch: RawEnvelopeBatch = serde_json::from_str(fixture).unwrap();

    let normalized = normalize_batch(&batch).unwrap();

    assert!(normalized
        .records
        .iter()
        .any(|record| matches!(record, AnyRecord::Session(_))));
    assert!(normalized
        .records
        .iter()
        .any(|record| matches!(record, AnyRecord::Run(_))));
    assert!(normalized
        .records
        .iter()
        .any(|record| matches!(record, AnyRecord::Message(_))));
}
