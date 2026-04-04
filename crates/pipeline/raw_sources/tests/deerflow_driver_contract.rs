use deer_pipeline_raw_sources::DeerFlowDriver;

#[test]
fn deerflow_driver_emits_storage_requests_without_owning_paths() {
    let driver = DeerFlowDriver::default();
    let request = driver.map_transcript_line("mission_1", "hello");

    assert_eq!(
        request.layout.level,
        deer_foundation_contracts::CanonicalLevel::L0
    );
    assert_eq!(request.layout.hierarchy.as_str(), "A");
    assert_eq!(request.layout.payload_kind.as_str(), "chat-note");
}

#[test]
fn deerflow_driver_emits_control_requests_separately() {
    let driver = DeerFlowDriver::default();
    let request = driver.map_exclusion("mission_1", "as_is_hash_xyz");

    assert_eq!(request.target_refs[0], "as_is_hash_xyz");
}
