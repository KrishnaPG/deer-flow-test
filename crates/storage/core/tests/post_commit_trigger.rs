use deer_storage_core::downstream_handoff::derive_trigger_from_manifest;

#[test]
fn manifest_trigger_uses_manifest_target_not_partial_members() {
    let trigger = derive_trigger_from_manifest(
        "write_7",
        "B/L3/chunks/transcript/day/manifest.json",
        vec![("mission_id".into(), "mission_1".into())],
        vec!["parent_1".into()],
        vec![("date".into(), "2026-04-04".into())],
        "7bWpKq9xR3mNvHf2Tc8Yd".to_string(),
        "s3://berg10-storage/7bWpKq9xR3mNvHf2Tc8Yd".to_string(),
    );
    assert_eq!(
        trigger.relative_target,
        "B/L3/chunks/transcript/day/manifest.json"
    );
    assert_eq!(
        trigger.correlation_ids.mission_id.unwrap().to_string(),
        "mission_1"
    );
    assert_eq!(trigger.content_hash, "7bWpKq9xR3mNvHf2Tc8Yd");
}
