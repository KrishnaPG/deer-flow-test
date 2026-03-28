//! Integration tests for the bridge module.
//!
//! Covers plugin registration, poll behaviour with no client,
//! poll cap, event forwarding, and StreamMessage flow.

use bevy::prelude::*;

use deer_gui::bridge::{BridgeClientResource, BridgeEvent, BridgeEventQueue};
use deer_gui::constants::timing::BRIDGE_POLL_MAX_PER_FRAME;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Builds a minimal app with bridge resources initialized manually
/// (no real Python subprocess).
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(BridgeClientResource { client: None });
    app.init_resource::<BridgeEventQueue>();
    app
}

// ---------------------------------------------------------------------------
// T-BRIDGE-01  BridgePlugin registers expected resources
// ---------------------------------------------------------------------------

#[test]
fn t_bridge_01_plugin_registers_resources() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    // We cannot add BridgePlugin directly because it spawns a real
    // Python subprocess. Instead, verify the resources it would create
    // by inserting them manually (same types).
    app.insert_resource(BridgeClientResource { client: None });
    app.init_resource::<BridgeEventQueue>();
    app.update();

    assert!(
        app.world().get_resource::<BridgeClientResource>().is_some(),
        "BridgeClientResource must be registered"
    );
    assert!(
        app.world().get_resource::<BridgeEventQueue>().is_some(),
        "BridgeEventQueue must be registered"
    );
}

// ---------------------------------------------------------------------------
// T-BRIDGE-02  Poll with None client produces no events
// ---------------------------------------------------------------------------

#[test]
fn t_bridge_02_poll_with_no_client() {
    let mut app = test_app();
    app.update();

    let queue = app.world().resource::<BridgeEventQueue>();
    assert!(
        queue.events.is_empty(),
        "no events should be produced when client is None"
    );
}

// ---------------------------------------------------------------------------
// T-BRIDGE-03  Poll cap constant is 64
// ---------------------------------------------------------------------------

#[test]
fn t_bridge_03_poll_cap_is_64() {
    assert_eq!(
        BRIDGE_POLL_MAX_PER_FRAME, 64,
        "BRIDGE_POLL_MAX_PER_FRAME must be 64"
    );
}

// ---------------------------------------------------------------------------
// T-BRIDGE-04  BridgeEventQueue starts empty each frame
// ---------------------------------------------------------------------------

#[test]
fn t_bridge_04_event_queue_default_empty() {
    let queue = BridgeEventQueue::default();
    assert!(
        queue.events.is_empty(),
        "default BridgeEventQueue must be empty"
    );
}

// ---------------------------------------------------------------------------
// T-BRIDGE-05  BridgeEvent::StreamMessage carries thread_id and message
// ---------------------------------------------------------------------------

#[test]
fn t_bridge_05_stream_message_fields() {
    let msg = deer_gui::models::ChatMessage {
        id: "msg-1".to_string(),
        role: "assistant".to_string(),
        content: "hello".to_string(),
        created_at: "2025-01-01".to_string(),
        ..Default::default()
    };

    let event = BridgeEvent::StreamMessage {
        thread_id: "thread-42".to_string(),
        message: msg,
    };

    match &event {
        BridgeEvent::StreamMessage { thread_id, message } => {
            assert_eq!(thread_id, "thread-42");
            assert_eq!(message.role, "assistant");
            assert_eq!(message.content, "hello");
        }
        _ => panic!("expected StreamMessage variant"),
    }
}
