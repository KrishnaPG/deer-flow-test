use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::acp_client::ids::{
    AcpJsonRpcRequestId, AcpSessionSequenceNumber, AcpSubprocessId, ChatRunId, ChatSessionId,
    ChatThreadId,
};

/// Timestamp wrapper for durable ACP protocol capture.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AcpCapturedEventTimestamp(pub DateTime<Utc>);

impl AcpCapturedEventTimestamp {
    pub fn now() -> Self {
        Self(Utc::now())
    }
}

/// Transport-observation kind for one captured ACP boundary event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcpProtocolFrameKind {
    ClientControlIntent,
    AcpClientRequest,
    AcpServerResponse,
    AcpServerNotification,
    SubprocessLifecycle,
    ClientError,
}

/// Durable raw event persisted to Redpanda before any domain interpretation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AcpCapturedProtocolEvent {
    pub engine_name: String,
    pub schema_version: String,
    pub chat_session_id: ChatSessionId,
    pub chat_run_id: ChatRunId,
    pub chat_thread_id: Option<ChatThreadId>,
    pub acp_subprocess_id: AcpSubprocessId,
    pub acp_session_sequence_number: AcpSessionSequenceNumber,
    pub captured_at: AcpCapturedEventTimestamp,
    pub acp_protocol_frame_kind: AcpProtocolFrameKind,
    pub acp_json_rpc_request_id: Option<AcpJsonRpcRequestId>,
    pub message_id: Option<String>,
    pub tool_call_id: Option<String>,
    pub raw_json_rpc_payload: Box<RawValue>,
}

impl AcpCapturedProtocolEvent {
    pub fn new(
        chat_session_id: ChatSessionId,
        chat_run_id: ChatRunId,
        acp_subprocess_id: AcpSubprocessId,
        acp_session_sequence_number: AcpSessionSequenceNumber,
        acp_protocol_frame_kind: AcpProtocolFrameKind,
        raw_json_rpc_payload: Box<RawValue>,
    ) -> Self {
        Self {
            engine_name: "hermes".to_string(),
            schema_version: "v1".to_string(),
            chat_session_id,
            chat_run_id,
            chat_thread_id: None,
            acp_subprocess_id,
            acp_session_sequence_number,
            captured_at: AcpCapturedEventTimestamp::now(),
            acp_protocol_frame_kind,
            acp_json_rpc_request_id: None,
            message_id: None,
            tool_call_id: None,
            raw_json_rpc_payload,
        }
    }
}
