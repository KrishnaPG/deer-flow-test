use std::sync::Arc;
use std::time::Duration;

use deer_foundation_paths::BaseDir;
use deer_pipeline_raw_sources::{
    AcpClientSessionConfig, OurAcpClient, RawEventFanout, RawEventReader, RedbRawEventPublisher,
};

#[tokio::test]
async fn test_hermes_e2e_capture() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let base_dir = BaseDir::new(temp_dir.path().to_path_buf());

    std::fs::create_dir_all(base_dir.incoming().as_path()).unwrap();
    let staging_db_path = base_dir.incoming().staging_db();

    let publisher =
        Arc::new(RedbRawEventPublisher::new(&staging_db_path).expect("Failed to open DB"));
    let raw_fanout = RawEventFanout::default();
    let mut rx = raw_fanout.subscribe();

    let client = OurAcpClient::new(Arc::clone(&publisher) as Arc<_>, raw_fanout);

    // Auto-discover the ACP agent binary (uses $ACP_AGENT_BIN or searches PATH)
    let config = AcpClientSessionConfig::discover()
        .expect("No ACP agent found. Set $ACP_AGENT_BIN or install hermes.")
        .with_working_directory(temp_dir.path().to_path_buf());

    let _session_id = tokio::time::timeout(Duration::from_secs(30), client.connect_session(config))
        .await
        .expect("Hermes connection timed out after 30s")
        .expect("Hermes connection failed");

    // Give some time for fanout messages to arrive and DB writes to sync
    tokio::time::sleep(Duration::from_millis(500)).await;

    let mut fanout_messages = Vec::new();
    let mut captured_session_id = None;
    while let Ok((sid, _seq, bytes)) = rx.try_recv() {
        if captured_session_id.is_none() {
            captured_session_id = Some(sid);
        }
        fanout_messages.push(bytes);
    }

    assert!(
        !fanout_messages.is_empty(),
        "Expected fanout messages from Hermes ACP session"
    );
    let session_id = captured_session_id.expect("Expected a session_id from fanout");

    // Replay Verification: look up raw events by the session ID used in capture
    let replayed_events = publisher
        .read_session_events(&session_id)
        .await
        .expect("Failed to replay events");
    assert!(
        !replayed_events.is_empty(),
        "Expected replayed events from DB"
    );

    assert_eq!(
        fanout_messages.len(),
        replayed_events.len(),
        "Replay count should match fanout count"
    );

    // Verify contents are valid JSON (allowing trailing newlines, skipping stderr log lines)
    for event in replayed_events {
        let s = std::str::from_utf8(&event).expect("Payload should be valid UTF-8");
        let trimmed = s.trim();
        if !trimmed.starts_with('{') {
            continue; // skip stderr log lines
        }
        let json_value: serde_json::Value = serde_json::from_str(trimmed)
            .unwrap_or_else(|e| panic!("Payload should be valid JSON: {:?}\n  raw: {:?}", e, s));
        assert!(json_value.is_object());
    }

    eprintln!(
        "E2E test successfully verified {} raw protocol events.",
        fanout_messages.len()
    );
}
