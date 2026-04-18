use agent_client_protocol as acp;

use crate::acp_client::live_event::{AcpResponseStreamEvent, AcpResponseStreamEventKind};
use crate::acp_client::{ChatRunId, ChatSessionId};

pub fn map_session_notification_to_live_event(
    notification: &acp::SessionNotification,
    chat_run_id: &ChatRunId,
) -> Option<AcpResponseStreamEvent> {
    let chat_session_id = ChatSessionId::new(notification.session_id.to_string());
    let kind = map_session_update_kind(&notification.update)?;
    Some(AcpResponseStreamEvent::new(
        chat_session_id,
        chat_run_id.clone(),
        kind,
    ))
}

fn map_session_update_kind(update: &acp::SessionUpdate) -> Option<AcpResponseStreamEventKind> {
    match update {
        acp::SessionUpdate::AgentThoughtChunk(chunk) => extract_text(chunk)
            .map(|text| AcpResponseStreamEventKind::AssistantThoughtStatusChunk { text }),
        acp::SessionUpdate::AgentMessageChunk(chunk) => {
            extract_text(chunk).map(|text| AcpResponseStreamEventKind::AssistantTextFinal { text })
        }
        acp::SessionUpdate::ToolCall(tool_call) => {
            Some(AcpResponseStreamEventKind::ToolCallStarted {
                tool_name: tool_call.title.clone(),
            })
        }
        acp::SessionUpdate::ToolCallUpdate(tool_call_update) => {
            let status = tool_call_update.fields.status?;
            if status == acp::ToolCallStatus::Completed {
                return Some(AcpResponseStreamEventKind::ToolCallCompleted {
                    tool_name: tool_call_update
                        .fields
                        .title
                        .clone()
                        .unwrap_or_else(|| tool_call_update.tool_call_id.to_string()),
                });
            }
            None
        }
        _ => None,
    }
}

fn extract_text(chunk: &acp::ContentChunk) -> Option<String> {
    match &chunk.content {
        acp::ContentBlock::Text(text) if !text.text.is_empty() => Some(text.text.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_agent_message_chunk_to_final_text_event() {
        let notification = acp::SessionNotification::new(
            "session-1",
            acp::SessionUpdate::AgentMessageChunk(acp::ContentChunk::new("hello".into())),
        );

        let event =
            map_session_notification_to_live_event(&notification, &ChatRunId::from("run-1"))
                .expect("message chunk should map");

        assert!(matches!(
            event.kind,
            AcpResponseStreamEventKind::AssistantTextFinal { ref text } if text == "hello"
        ));
    }
}
