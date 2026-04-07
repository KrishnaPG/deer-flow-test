use deer_foundation_contracts::{
    CanonicalLevel, CanonicalPlane, LogicalWriteId, StorageHierarchyTag, StoragePayloadFormat,
    StoragePayloadKind,
};
use deer_storage_core::downstream_handoff::make_file_saved;

#[test]
fn file_saved_carries_full_routing_context() {
    let payload = make_file_saved(
        LogicalWriteId::new("write_1"),
        "A/L0/as-is/chat-note/hash.jsonl",
        StorageHierarchyTag::new("A"),
        CanonicalLevel::L0,
        CanonicalPlane::AsIs,
        StoragePayloadKind::new("chat-note"),
        StoragePayloadFormat::new("jsonl"),
        vec![("mission_id".into(), "mission_1".into())],
        vec!["source_event_1".into()],
        vec![("mission".into(), "mission_1".into())],
        "7bWpKq9xR3mNvHf2Tc8Yd".to_string(),
    );

    match payload.target {
        deer_foundation_contracts::FileSavedTarget::SingleFile { relative_path } => {
            assert_eq!(relative_path, "A/L0/as-is/chat-note/hash.jsonl");
        }
        _ => panic!("expected single-file target"),
    }
    assert_eq!(
        payload.correlation_ids.mission_id.unwrap().to_string(),
        "mission_1"
    );
    assert_eq!(payload.content_hash, "7bWpKq9xR3mNvHf2Tc8Yd");
}
