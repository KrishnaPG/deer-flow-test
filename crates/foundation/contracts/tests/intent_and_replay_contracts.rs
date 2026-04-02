use chrono::Utc;
use deer_foundation_contracts::{
    EventId, IntentAction, IntentCommand, RecordFamily, RecordId, RecordRef, ReplayDirection,
    ReplayEnvelope, ThreadId, WriteOperation, WriteOperationKind,
};
use serde_json::json;

#[test]
fn intent_command_carries_target_and_payload() {
    let command = IntentCommand::new(
        RecordId::from_static("cmd_1"),
        IntentAction::SubmitMessage,
        Some(RecordRef::new(
            RecordFamily::Session,
            RecordId::from_static("session_1"),
        )),
        Some(ThreadId::from_static("thread_1")),
        json!({"text": "scan sector seven"}),
    );

    assert!(matches!(command.action, IntentAction::SubmitMessage));
    assert_eq!(
        command.thread_id.as_ref().map(ThreadId::as_str),
        Some("thread_1")
    );
}

#[test]
fn replay_envelope_keeps_append_only_links() {
    let envelope = ReplayEnvelope::append(
        7,
        EventId::from_static("evt_7"),
        RecordRef::new(RecordFamily::Run, RecordId::from_static("run_1")),
        WriteOperation::new(
            WriteOperationKind::AppendRecord,
            RecordId::from_static("run_1"),
        ),
        Some(EventId::from_static("evt_6")),
        Utc::now(),
    );

    assert_eq!(envelope.sequence, 7);
    assert_eq!(envelope.cursor().direction, ReplayDirection::Forward);
    assert_eq!(
        envelope.parent_event_id.as_ref().map(EventId::as_str),
        Some("evt_6")
    );
}
