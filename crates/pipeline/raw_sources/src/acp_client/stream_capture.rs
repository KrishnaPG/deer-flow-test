use agent_client_protocol as acp;

use crate::acp_client::captured_event::{AcpCapturedProtocolEvent, AcpProtocolFrameKind};
use crate::acp_client::ids::{
    AcpJsonRpcRequestId, AcpSessionSequenceNumber, AcpSubprocessId, ChatRunId, ChatSessionId,
};

pub fn capture_stream_message(
    chat_session_id: ChatSessionId,
    chat_run_id: ChatRunId,
    acp_subprocess_id: AcpSubprocessId,
    sequence_number: AcpSessionSequenceNumber,
    message: &acp::StreamMessage,
) -> Result<AcpCapturedProtocolEvent, serde_json::Error> {
    let raw_payload = raw_payload_from_stream_message(message)?;
    let mut event = AcpCapturedProtocolEvent::new(
        chat_session_id,
        chat_run_id,
        acp_subprocess_id,
        sequence_number,
        protocol_frame_kind(message),
        raw_payload,
    );
    event.acp_json_rpc_request_id = request_id_from_stream_message(message);
    Ok(event)
}

fn raw_payload_from_stream_message(
    message: &acp::StreamMessage,
) -> Result<Box<serde_json::value::RawValue>, serde_json::Error> {
    match &message.message {
        acp::StreamMessageContent::Request { id, method, params } => {
            serde_json::value::to_raw_value(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": request_id_json(id),
                "method": method,
                "params": params,
            }))
        }
        acp::StreamMessageContent::Response { id, result } => {
            serde_json::value::to_raw_value(&response_json(id, result))
        }
        acp::StreamMessageContent::Notification { method, params } => {
            serde_json::value::to_raw_value(&serde_json::json!({
                "jsonrpc": "2.0",
                "method": method,
                "params": params,
            }))
        }
    }
}

fn response_json(
    id: &acp::RequestId,
    result: &acp::Result<Option<serde_json::Value>>,
) -> serde_json::Value {
    match result {
        Ok(value) => serde_json::json!({
            "jsonrpc": "2.0",
            "id": request_id_json(id),
            "result": value,
        }),
        Err(error) => serde_json::json!({
            "jsonrpc": "2.0",
            "id": request_id_json(id),
            "error": error,
        }),
    }
}

fn protocol_frame_kind(message: &acp::StreamMessage) -> AcpProtocolFrameKind {
    match (&message.direction, &message.message) {
        (acp::StreamMessageDirection::Outgoing, acp::StreamMessageContent::Request { .. }) => {
            AcpProtocolFrameKind::AcpClientRequest
        }
        (acp::StreamMessageDirection::Outgoing, acp::StreamMessageContent::Response { .. }) => {
            AcpProtocolFrameKind::AcpClientResponse
        }
        (acp::StreamMessageDirection::Outgoing, acp::StreamMessageContent::Notification { .. }) => {
            AcpProtocolFrameKind::AcpClientNotification
        }
        (acp::StreamMessageDirection::Incoming, acp::StreamMessageContent::Request { .. }) => {
            AcpProtocolFrameKind::AcpServerRequest
        }
        (acp::StreamMessageDirection::Incoming, acp::StreamMessageContent::Response { .. }) => {
            AcpProtocolFrameKind::AcpServerResponse
        }
        (acp::StreamMessageDirection::Incoming, acp::StreamMessageContent::Notification { .. }) => {
            AcpProtocolFrameKind::AcpServerNotification
        }
    }
}

fn request_id_from_stream_message(message: &acp::StreamMessage) -> Option<AcpJsonRpcRequestId> {
    match &message.message {
        acp::StreamMessageContent::Request { id, .. }
        | acp::StreamMessageContent::Response { id, .. } => {
            Some(AcpJsonRpcRequestId::new(request_id_string(id)))
        }
        acp::StreamMessageContent::Notification { .. } => None,
    }
}

fn request_id_json(id: &acp::RequestId) -> serde_json::Value {
    match id {
        acp::RequestId::Str(text) => serde_json::Value::String(text.to_string()),
        acp::RequestId::Number(number) => serde_json::Value::Number((*number).into()),
        acp::RequestId::Null => serde_json::Value::Null,
    }
}

fn request_id_string(id: &acp::RequestId) -> String {
    match id {
        acp::RequestId::Str(text) => text.to_string(),
        acp::RequestId::Number(number) => number.to_string(),
        acp::RequestId::Null => "null".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outgoing_requests_are_captured_as_client_requests() {
        let message = acp::StreamMessage {
            direction: acp::StreamMessageDirection::Outgoing,
            message: acp::StreamMessageContent::Request {
                id: acp::RequestId::Number(7),
                method: "session/prompt".into(),
                params: Some(serde_json::json!({"x": 1})),
            },
        };

        let event = capture_stream_message(
            ChatSessionId::from("session-1"),
            ChatRunId::from("run-1"),
            AcpSubprocessId::from("proc-1"),
            AcpSessionSequenceNumber::new(3),
            &message,
        )
        .unwrap();

        assert_eq!(
            event.acp_protocol_frame_kind,
            AcpProtocolFrameKind::AcpClientRequest
        );
        assert_eq!(event.acp_json_rpc_request_id.unwrap().as_str(), "7");
    }
}
