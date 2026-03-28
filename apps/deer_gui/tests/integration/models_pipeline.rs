//! Integration tests for models pipeline types.
//!
//! Verifies that domain model types survive serialization round-trips
//! and maintain their expected structure through the pipeline.

use deer_gui::models::{ChatMessage, ModelInfo, ThreadSummary};

// ---------------------------------------------------------------------------
// T-MODEL-01  ChatMessage default fields
// ---------------------------------------------------------------------------

#[test]
fn t_model_01_chat_message_default() {
    let msg = ChatMessage::default();
    assert!(msg.id.is_empty(), "default id must be empty");
    assert!(msg.role.is_empty(), "default role must be empty");
    assert!(msg.content.is_empty(), "default content must be empty");
    assert!(
        msg.tool_calls.is_empty(),
        "default tool_calls must be empty"
    );
    assert!(
        msg.attachments.is_empty(),
        "default attachments must be empty"
    );
    assert!(msg.usage.is_none(), "default usage must be None");
}

// ---------------------------------------------------------------------------
// T-MODEL-02  ChatMessage survives serde round-trip
// ---------------------------------------------------------------------------

#[test]
fn t_model_02_chat_message_serde_round_trip() {
    let msg = ChatMessage {
        id: "msg-1".to_string(),
        role: "user".to_string(),
        content: "Tell me about Rust".to_string(),
        created_at: "2025-06-01T00:00:00Z".to_string(),
        name: Some("Alice".to_string()),
        tool_calls: Vec::new(),
        attachments: Vec::new(),
        usage: None,
    };

    let json = serde_json::to_string(&msg).expect("serialize");
    let restored: ChatMessage = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(restored.id, "msg-1");
    assert_eq!(restored.role, "user");
    assert_eq!(restored.content, "Tell me about Rust");
    assert_eq!(restored.name.as_deref(), Some("Alice"));
}

// ---------------------------------------------------------------------------
// T-MODEL-03  ModelInfo and ThreadSummary survive serde round-trip
// ---------------------------------------------------------------------------

#[test]
fn t_model_03_model_info_and_thread_summary_serde() {
    let model = ModelInfo {
        name: "gpt-4".to_string(),
        display_name: Some("GPT-4".to_string()),
        model: Some("gpt-4-0613".to_string()),
        supports_thinking: Some(true),
        supports_reasoning_effort: Some(false),
    };

    let json = serde_json::to_string(&model).expect("serialize ModelInfo");
    let restored: ModelInfo = serde_json::from_str(&json).expect("deserialize ModelInfo");
    assert_eq!(restored.name, "gpt-4");
    assert_eq!(restored.supports_thinking, Some(true));

    let summary = ThreadSummary {
        thread_id: "t-1".to_string(),
        title: "Test thread".to_string(),
        created_at: "2025-01-01".to_string(),
        updated_at: "2025-01-02".to_string(),
        message_count: 5,
        artifacts: vec!["a.txt".to_string()],
    };

    let json = serde_json::to_string(&summary).expect("serialize ThreadSummary");
    let restored: ThreadSummary = serde_json::from_str(&json).expect("deserialize ThreadSummary");
    assert_eq!(restored.thread_id, "t-1");
    assert_eq!(restored.message_count, 5);
    assert_eq!(restored.artifacts.len(), 1);
}
