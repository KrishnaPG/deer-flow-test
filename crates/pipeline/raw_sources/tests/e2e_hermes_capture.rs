
use std::sync::Arc;
use std::time::Duration;

use deer_foundation_paths::BaseDir;
use deer_pipeline_raw_sources::{
    AcpClientSessionConfig, OurAcpClient, RawEventFanout, RedbRawEventPublisher,
    RawEventReader,
};

#[tokio::test]
async fn test_hermes_e2e_capture_mock() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let base_dir = BaseDir::new(temp_dir.path().to_path_buf());
    
    std::fs::create_dir_all(base_dir.incoming().as_path()).unwrap();
    let staging_db_path = base_dir.incoming().staging_db();

    let publisher = Arc::new(RedbRawEventPublisher::new(&staging_db_path).expect("Failed to open DB"));
    let raw_fanout = RawEventFanout::default();
    let mut rx = raw_fanout.subscribe();

    let client = OurAcpClient::new(Arc::clone(&publisher) as Arc<_>, raw_fanout);

    // We'll write a quick bash script that outputs JSON-RPC lines simulating hermes.
    // The script prints the initialize response and new_session response to satisfy client connection.
    let script_path = temp_dir.path().join("mock_hermes.sh");
    std::fs::write(&script_path, r#"#!/bin/bash
echo '{"jsonrpc":"2.0","id":0,"result":{"protocolVersion":"2024-11-05","capabilities":{},"serverInfo":{"name":"mock","version":"1.0"}}}'
echo '{"jsonrpc":"2.0","id":1,"result":{"sessionId":"mock-session-id"}}'
sleep 0.1
echo '{"jsonrpc":"2.0","method":"notifications/test","params":{}}'
sleep 0.1
"#).unwrap();
    
    // make executable
    std::process::Command::new("chmod")
        .arg("+x")
        .arg(&script_path)
        .status()
        .unwrap();

    let config = AcpClientSessionConfig {
        executable: script_path,
        args: vec![],
        working_directory: temp_dir.path().to_path_buf(), 
    };

    let session_id_result = tokio::time::timeout(Duration::from_secs(2), client.connect_session(config)).await;
    // We don't care if the agent-client-protocol strict handshake fails because our mock just echoes
    // fixed IDs which won't match the dynamic ones. What we care about is that the edge captured it.
    
    // The session ID is deterministically hash(first_event) + timestamp + pid.
    // However, the test can't easily guess it since pid and timestamp are dynamic.
    // Let's read from the raw_fanout instead to get the session ID.

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

    assert!(!fanout_messages.is_empty(), "Expected fanout messages from Hermes stdout");
    let session_id = captured_session_id.expect("Expected a session_id from fanout");

    // Phase 4: Replay Verification
    let replayed_events = publisher.read_session_events(&session_id).await.expect("Failed to replay events");
    assert!(!replayed_events.is_empty(), "Expected replayed events from DB");

    assert_eq!(
        fanout_messages.len(),
        replayed_events.len(),
        "Replay count should match fanout count"
    );

    // Verify contents are valid JSON
    for event in replayed_events {
        let json_value: serde_json::Value = serde_json::from_slice(&event).expect("Payload should be valid JSON");
        assert!(json_value.is_object());
    }

    println!("E2E test successfully verified {} raw protocol events.", fanout_messages.len());
}
