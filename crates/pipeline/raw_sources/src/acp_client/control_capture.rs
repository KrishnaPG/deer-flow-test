use serde::Serialize;
use serde_json::value::to_raw_value;

use crate::acp_client::captured_event::{AcpCapturedProtocolEvent, AcpProtocolFrameKind};
use crate::acp_client::ids::{AcpSessionSequenceNumber, AcpSubprocessId, ChatRunId, ChatSessionId};

pub fn capture_client_control_event<T>(
    chat_session_id: ChatSessionId,
    chat_run_id: ChatRunId,
    acp_subprocess_id: AcpSubprocessId,
    sequence_number: AcpSessionSequenceNumber,
    payload: &T,
) -> Result<AcpCapturedProtocolEvent, serde_json::Error>
where
    T: Serialize,
{
    let raw_payload = to_raw_value(payload)?;
    Ok(AcpCapturedProtocolEvent::new(
        chat_session_id,
        chat_run_id,
        acp_subprocess_id,
        sequence_number,
        AcpProtocolFrameKind::ClientControlIntent,
        raw_payload,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_control_capture_marks_transport_kind() {
        let event = capture_client_control_event(
            ChatSessionId::from("session-1"),
            ChatRunId::from("run-1"),
            AcpSubprocessId::from("proc-1"),
            AcpSessionSequenceNumber::new(9),
            &serde_json::json!({"method": "session/prompt"}),
        )
        .unwrap();

        assert_eq!(
            event.acp_protocol_frame_kind,
            AcpProtocolFrameKind::ClientControlIntent
        );
    }
}
