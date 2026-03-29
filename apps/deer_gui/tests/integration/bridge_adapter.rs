//! Functional tests for the bridge adapter + dispatch pipeline.
//!
//! Each test feeds a real JSON payload (as the Python bridge would produce)
//! through `serde_json::from_str -> dispatch_envelope -> channel -> BridgeEvent`,
//! verifying the domain event that comes out the other side.
//!
//! NO mocks, NO stubs — real JSON, real deserialization, real dispatch.

use crossbeam_channel::unbounded;

use deer_gui::bridge::{dispatch_envelope, BridgeEvent, BridgeResponse, InboundEnvelope};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Deserialize a JSON string into an InboundEnvelope, dispatch it through
/// the real pipeline, and return the resulting BridgeEvent.
fn dispatch_json(json: &str) -> BridgeEvent {
    let envelope: InboundEnvelope =
        serde_json::from_str(json).expect("test JSON must parse as InboundEnvelope");
    let (tx, rx) = unbounded();
    dispatch_envelope(envelope, &tx);
    rx.try_recv().expect("dispatch must produce an event")
}

// ---------------------------------------------------------------------------
// T-ADAPTER-01  Ready envelope
// ---------------------------------------------------------------------------

#[test]
fn t_adapter_01_ready_envelope() {
    let event = dispatch_json(r#"{"kind":"ready"}"#);
    assert!(
        matches!(event, BridgeEvent::Ready),
        "expected Ready, got {event:?}"
    );
}

// ---------------------------------------------------------------------------
// T-ADAPTER-02  Response with threads list
// ---------------------------------------------------------------------------

#[test]
fn t_adapter_02_response_threads() {
    let json = r#"{
        "kind": "response",
        "id": "req-1",
        "ok": true,
        "result": {
            "threads": [
                {
                    "thread_id": "t-1",
                    "title": "First",
                    "created_at": "2025-01-01",
                    "updated_at": "2025-01-02",
                    "message_count": 3,
                    "artifacts": []
                }
            ]
        }
    }"#;
    let event = dispatch_json(json);
    match event {
        BridgeEvent::Response {
            request_id,
            response: BridgeResponse::Threads(threads),
        } => {
            assert_eq!(request_id.as_deref(), Some("req-1"));
            assert_eq!(threads.len(), 1);
            assert_eq!(threads[0].thread_id, "t-1");
            assert_eq!(threads[0].title, "First");
            assert_eq!(threads[0].message_count, 3);
        }
        _ => panic!("expected Response/Threads, got {event:?}"),
    }
}

// ---------------------------------------------------------------------------
// T-ADAPTER-03  Response with single thread
// ---------------------------------------------------------------------------

#[test]
fn t_adapter_03_response_single_thread() {
    let json = r#"{
        "kind": "response",
        "id": "req-2",
        "ok": true,
        "result": {
            "thread": {
                "thread_id": "t-2",
                "title": "My Thread",
                "created_at": "2025-06-01",
                "updated_at": "2025-06-02",
                "messages": [
                    {
                        "id": "msg-1",
                        "role": "user",
                        "content": "hello",
                        "created_at": "2025-06-01"
                    }
                ],
                "artifacts": [],
                "todos": [],
                "suggestions": []
            }
        }
    }"#;
    let event = dispatch_json(json);
    match event {
        BridgeEvent::Response {
            request_id,
            response: BridgeResponse::Thread(record),
        } => {
            assert_eq!(request_id.as_deref(), Some("req-2"));
            assert_eq!(record.thread_id, "t-2");
            assert_eq!(record.messages.len(), 1);
            assert_eq!(record.messages[0].role, "user");
        }
        _ => panic!("expected Response/Thread, got {event:?}"),
    }
}

// ---------------------------------------------------------------------------
// T-ADAPTER-04  Response with created_thread
// ---------------------------------------------------------------------------

#[test]
fn t_adapter_04_response_created_thread() {
    let json = r#"{
        "kind": "response",
        "id": "req-3",
        "ok": true,
        "result": {
            "created_thread": {
                "thread_id": "t-new",
                "title": "",
                "created_at": "2025-06-10",
                "updated_at": "2025-06-10",
                "messages": [],
                "artifacts": [],
                "todos": [],
                "suggestions": []
            }
        }
    }"#;
    let event = dispatch_json(json);
    match event {
        BridgeEvent::Response {
            response: BridgeResponse::CreatedThread(record),
            ..
        } => {
            assert_eq!(record.thread_id, "t-new");
        }
        _ => panic!("expected Response/CreatedThread, got {event:?}"),
    }
}

// ---------------------------------------------------------------------------
// T-ADAPTER-05  Response with models
// ---------------------------------------------------------------------------

#[test]
fn t_adapter_05_response_models() {
    let json = r#"{
        "kind": "response",
        "id": "req-4",
        "ok": true,
        "result": {
            "models": [
                {"name": "gpt-4", "display_name": "GPT-4"},
                {"name": "claude", "display_name": "Claude"}
            ]
        }
    }"#;
    let event = dispatch_json(json);
    match event {
        BridgeEvent::Response {
            response: BridgeResponse::Models(models),
            ..
        } => {
            assert_eq!(models.len(), 2);
            assert_eq!(models[0].name, "gpt-4");
            assert_eq!(models[1].name, "claude");
        }
        _ => panic!("expected Response/Models, got {event:?}"),
    }
}

// ---------------------------------------------------------------------------
// T-ADAPTER-06  Response with renamed_thread
// ---------------------------------------------------------------------------

#[test]
fn t_adapter_06_response_renamed() {
    let json = r#"{
        "kind": "response",
        "id": "req-5",
        "ok": true,
        "result": {
            "renamed_thread": {
                "thread_id": "t-1",
                "title": "New Title",
                "created_at": "2025-01-01",
                "updated_at": "2025-06-15",
                "message_count": 5,
                "artifacts": []
            }
        }
    }"#;
    let event = dispatch_json(json);
    match event {
        BridgeEvent::Response {
            response: BridgeResponse::Renamed(summary),
            ..
        } => {
            assert_eq!(summary.title, "New Title");
        }
        _ => panic!("expected Response/Renamed, got {event:?}"),
    }
}

// ---------------------------------------------------------------------------
// T-ADAPTER-07  Response with deleted_thread_id
// ---------------------------------------------------------------------------

#[test]
fn t_adapter_07_response_deleted() {
    let json = r#"{
        "kind": "response",
        "id": "req-6",
        "ok": true,
        "result": {"deleted_thread_id": "t-gone"}
    }"#;
    let event = dispatch_json(json);
    match event {
        BridgeEvent::Response {
            response: BridgeResponse::Deleted(id),
            ..
        } => {
            assert_eq!(id, "t-gone");
        }
        _ => panic!("expected Response/Deleted, got {event:?}"),
    }
}

// ---------------------------------------------------------------------------
// T-ADAPTER-08  Response with artifact
// ---------------------------------------------------------------------------

#[test]
fn t_adapter_08_response_artifact() {
    let json = r#"{
        "kind": "response",
        "id": "req-7",
        "ok": true,
        "result": {
            "artifact": {
                "thread_id": "t-1",
                "virtual_path": "/docs/readme.md",
                "host_path": "/tmp/readme.md",
                "mime_type": "text/markdown"
            }
        }
    }"#;
    let event = dispatch_json(json);
    match event {
        BridgeEvent::Response {
            response: BridgeResponse::Artifact(info),
            ..
        } => {
            assert_eq!(info.virtual_path, "/docs/readme.md");
            assert_eq!(info.host_path, "/tmp/readme.md");
            assert_eq!(info.mime_type, "text/markdown");
        }
        _ => panic!("expected Response/Artifact, got {event:?}"),
    }
}

// ---------------------------------------------------------------------------
// T-ADAPTER-09  Response ack (ok=true, no result)
// ---------------------------------------------------------------------------

#[test]
fn t_adapter_09_response_ack() {
    let json = r#"{
        "kind": "response",
        "id": "req-8",
        "ok": true
    }"#;
    let event = dispatch_json(json);
    match event {
        BridgeEvent::Response {
            response: BridgeResponse::Ack,
            ..
        } => {}
        _ => panic!("expected Response/Ack, got {event:?}"),
    }
}

// ---------------------------------------------------------------------------
// T-ADAPTER-10  Response error (ok=false)
// ---------------------------------------------------------------------------

#[test]
fn t_adapter_10_response_error() {
    let json = r#"{
        "kind": "response",
        "id": "req-9",
        "ok": false,
        "error": "thread not found"
    }"#;
    let event = dispatch_json(json);
    match event {
        BridgeEvent::Error { message } => {
            assert!(
                message.contains("thread not found"),
                "error message should contain cause: {message}"
            );
        }
        _ => panic!("expected Error, got {event:?}"),
    }
}

// ---------------------------------------------------------------------------
// T-ADAPTER-11  Event: message (StreamMessage)
// ---------------------------------------------------------------------------

#[test]
fn t_adapter_11_event_message() {
    let json = r#"{
        "kind": "event",
        "event": "message",
        "data": {
            "thread_id": "t-100",
            "message": {
                "id": "msg-42",
                "role": "assistant",
                "content": "Hello, world!",
                "created_at": "2025-06-20"
            }
        }
    }"#;
    let event = dispatch_json(json);
    match event {
        BridgeEvent::StreamMessage { thread_id, message } => {
            assert_eq!(thread_id, "t-100");
            assert_eq!(message.id, "msg-42");
            assert_eq!(message.role, "assistant");
            assert_eq!(message.content, "Hello, world!");
        }
        _ => panic!("expected StreamMessage, got {event:?}"),
    }
}

// ---------------------------------------------------------------------------
// T-ADAPTER-12  Event: state
// ---------------------------------------------------------------------------

#[test]
fn t_adapter_12_event_state() {
    let json = r#"{
        "kind": "event",
        "event": "state",
        "data": {
            "thread_id": "t-200",
            "title": "Updated Title",
            "artifacts": ["a.png"],
            "todos": [{"content": "do stuff", "status": "pending"}]
        }
    }"#;
    let event = dispatch_json(json);
    match event {
        BridgeEvent::State { state } => {
            assert_eq!(state.thread_id, "t-200");
            assert_eq!(state.title.as_deref(), Some("Updated Title"));
            assert_eq!(state.artifacts, vec!["a.png"]);
            assert_eq!(state.todos.len(), 1);
        }
        _ => panic!("expected State, got {event:?}"),
    }
}

// ---------------------------------------------------------------------------
// T-ADAPTER-13  Event: suggestions
// ---------------------------------------------------------------------------

#[test]
fn t_adapter_13_event_suggestions() {
    let json = r#"{
        "kind": "event",
        "event": "suggestions",
        "data": {
            "thread_id": "t-300",
            "suggestions": ["try this", "try that"]
        }
    }"#;
    let event = dispatch_json(json);
    match event {
        BridgeEvent::Suggestions {
            thread_id,
            suggestions,
        } => {
            assert_eq!(thread_id, "t-300");
            assert_eq!(suggestions, vec!["try this", "try that"]);
        }
        _ => panic!("expected Suggestions, got {event:?}"),
    }
}

// ---------------------------------------------------------------------------
// T-ADAPTER-14  Event: done (with usage)
// ---------------------------------------------------------------------------

#[test]
fn t_adapter_14_event_done_with_usage() {
    let json = r#"{
        "kind": "event",
        "event": "done",
        "data": {
            "thread_id": "t-400",
            "usage": {
                "input_tokens": 100,
                "output_tokens": 50,
                "total_tokens": 150
            }
        }
    }"#;
    let event = dispatch_json(json);
    match event {
        BridgeEvent::Done { thread_id, usage } => {
            assert_eq!(thread_id, "t-400");
            let u = usage.expect("usage should be present");
            assert_eq!(u.input_tokens, Some(100));
            assert_eq!(u.output_tokens, Some(50));
            assert_eq!(u.total_tokens, Some(150));
        }
        _ => panic!("expected Done, got {event:?}"),
    }
}

// ---------------------------------------------------------------------------
// T-ADAPTER-15  Event: done (without usage)
// ---------------------------------------------------------------------------

#[test]
fn t_adapter_15_event_done_no_usage() {
    let json = r#"{
        "kind": "event",
        "event": "done",
        "data": {"thread_id": "t-401"}
    }"#;
    let event = dispatch_json(json);
    match event {
        BridgeEvent::Done { thread_id, usage } => {
            assert_eq!(thread_id, "t-401");
            assert!(usage.is_none(), "usage should be None");
        }
        _ => panic!("expected Done, got {event:?}"),
    }
}

// ---------------------------------------------------------------------------
// T-ADAPTER-16  Event: error
// ---------------------------------------------------------------------------

#[test]
fn t_adapter_16_event_error() {
    let json = r#"{
        "kind": "event",
        "event": "error",
        "data": {"message": "something broke"}
    }"#;
    let event = dispatch_json(json);
    match event {
        BridgeEvent::Error { message } => {
            assert!(
                message.contains("something broke"),
                "error should contain message: {message}"
            );
        }
        _ => panic!("expected Error, got {event:?}"),
    }
}

// ---------------------------------------------------------------------------
// T-ADAPTER-17  Unknown envelope kind yields Error
// ---------------------------------------------------------------------------

#[test]
fn t_adapter_17_unknown_kind() {
    let json = r#"{"kind":"bogus"}"#;
    let event = dispatch_json(json);
    match event {
        BridgeEvent::Error { message } => {
            assert!(
                message.contains("bogus"),
                "error should mention unknown kind: {message}"
            );
        }
        _ => panic!("expected Error, got {event:?}"),
    }
}

// ---------------------------------------------------------------------------
// T-ADAPTER-18  Unknown event type yields Error
// ---------------------------------------------------------------------------

#[test]
fn t_adapter_18_unknown_event_type() {
    let json = r#"{
        "kind": "event",
        "event": "nonexistent",
        "data": {}
    }"#;
    let event = dispatch_json(json);
    match event {
        BridgeEvent::Error { message } => {
            assert!(
                message.contains("nonexistent"),
                "error should mention unknown event: {message}"
            );
        }
        _ => panic!("expected Error, got {event:?}"),
    }
}

// ---------------------------------------------------------------------------
// T-ADAPTER-19  Response with unknown result key yields Raw fallback
// ---------------------------------------------------------------------------

#[test]
fn t_adapter_19_response_unknown_key_yields_raw() {
    let json = r#"{
        "kind": "response",
        "id": "req-99",
        "ok": true,
        "result": {"something_new": 42}
    }"#;
    let event = dispatch_json(json);
    match event {
        BridgeEvent::Response {
            response: BridgeResponse::Raw(value),
            ..
        } => {
            assert!(value.is_object(), "raw should be an object");
        }
        _ => panic!("expected Response/Raw, got {event:?}"),
    }
}

// ---------------------------------------------------------------------------
// T-ADAPTER-20  Event with missing data yields Error
// ---------------------------------------------------------------------------

#[test]
fn t_adapter_20_event_missing_data() {
    let json = r#"{
        "kind": "event",
        "event": "message"
    }"#;
    let event = dispatch_json(json);
    match event {
        BridgeEvent::Error { message } => {
            assert!(
                message.contains("message"),
                "error should reference the event type: {message}"
            );
        }
        _ => panic!("expected Error, got {event:?}"),
    }
}

// ---------------------------------------------------------------------------
// T-ADAPTER-21  Message event with tool_calls and attachments
// ---------------------------------------------------------------------------

#[test]
fn t_adapter_21_message_with_tool_calls() {
    let json = r#"{
        "kind": "event",
        "event": "message",
        "data": {
            "thread_id": "t-500",
            "message": {
                "id": "msg-99",
                "role": "assistant",
                "content": "calling tool",
                "created_at": "2025-06-25",
                "tool_calls": [
                    {"name": "search", "args": {"q": "rust"}}
                ],
                "attachments": [
                    {"filename": "report.pdf", "size": 1024}
                ]
            }
        }
    }"#;
    let event = dispatch_json(json);
    match event {
        BridgeEvent::StreamMessage { message, .. } => {
            assert_eq!(message.tool_calls.len(), 1);
            assert_eq!(message.tool_calls[0].name, "search");
            assert_eq!(message.attachments.len(), 1);
            assert_eq!(message.attachments[0].filename, "report.pdf");
            assert_eq!(message.attachments[0].size, Some(1024));
        }
        _ => panic!("expected StreamMessage, got {event:?}"),
    }
}
