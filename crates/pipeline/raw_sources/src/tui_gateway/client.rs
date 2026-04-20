use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::AsyncWriteExt;
use tokio::process::ChildStdin;
use tokio::sync::{broadcast, mpsc};

use crate::acp_client::{ChatSessionId, NoopRawEventPublisher, RawEventFanout, RawEventPublisher};
use crate::tui_gateway::process::spawn_tui_gateway;
use crate::tui_gateway::raw_capture::capture_gateway_stream;
use crate::tui_gateway::{
    TerminalColumnCount, TuiGatewayCommand, TuiGatewayRequestId, TuiGatewayRunId,
    TuiGatewaySessionId,
};

#[derive(Debug, thiserror::Error)]
pub enum TuiGatewayError {
    #[error("TUI gateway stdio missing: {0}")]
    MissingStdio(&'static str),
    #[error("TUI gateway I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("TUI gateway protocol failed: {0}")]
    Protocol(String),
    #[error("TUI gateway response stream closed")]
    ResponseStreamClosed,
}

pub struct TuiGatewayClient {
    raw_session_id: ChatSessionId,
    stdin: ChildStdin,
    frames_rx: mpsc::UnboundedReceiver<Bytes>,
    raw_fanout: RawEventFanout,
}

impl TuiGatewayClient {
    pub async fn spawn(command: TuiGatewayCommand) -> Result<Self, TuiGatewayError> {
        Self::spawn_with_raw_sink(
            command,
            Arc::new(NoopRawEventPublisher),
            RawEventFanout::default(),
        )
        .await
    }

    pub async fn spawn_with_raw_sink(
        command: TuiGatewayCommand,
        raw_publisher: Arc<dyn RawEventPublisher>,
        raw_fanout: RawEventFanout,
    ) -> Result<Self, TuiGatewayError> {
        let mut process = spawn_tui_gateway(&command).await?;
        let raw_session_id =
            ChatSessionId::new(format!("tui_gateway:{}", process.process_id.as_str()));
        let stdin = process
            .child
            .stdin
            .take()
            .ok_or(TuiGatewayError::MissingStdio("stdin"))?;
        let stdout = process
            .child
            .stdout
            .take()
            .ok_or(TuiGatewayError::MissingStdio("stdout"))?;
        let stderr = process
            .child
            .stderr
            .take()
            .ok_or(TuiGatewayError::MissingStdio("stderr"))?;

        let sequence = Arc::new(AtomicU64::new(1));
        let (frames_tx, frames_rx) = mpsc::unbounded_channel();
        tokio::spawn(capture_gateway_stream(
            raw_session_id.clone(),
            Arc::clone(&sequence),
            stdout,
            Arc::clone(&raw_publisher),
            raw_fanout.clone(),
            Some(frames_tx),
        ));
        tokio::spawn(capture_gateway_stream(
            raw_session_id.clone(),
            sequence,
            stderr,
            raw_publisher,
            raw_fanout.clone(),
            None,
        ));

        Ok(Self {
            raw_session_id,
            stdin,
            frames_rx,
            raw_fanout,
        })
    }

    pub fn raw_session_id(&self) -> &ChatSessionId {
        &self.raw_session_id
    }

    pub fn subscribe_raw(&self) -> broadcast::Receiver<(ChatSessionId, u64, Bytes)> {
        self.raw_fanout.subscribe()
    }

    pub async fn create_session(
        &mut self,
        columns: TerminalColumnCount,
    ) -> Result<TuiGatewaySessionId, TuiGatewayError> {
        let request_id = self
            .send_request(
                TuiGatewayMethod::SessionCreate,
                SessionCreateParams { cols: columns },
            )
            .await?;
        let result = self.read_response_result(&request_id).await?;
        let session_id = result
            .get("session_id")
            .and_then(Value::as_str)
            .ok_or_else(|| TuiGatewayError::Protocol("missing session_id".to_string()))?;
        Ok(TuiGatewaySessionId::new(session_id))
    }

    pub async fn submit_prompt(
        &mut self,
        session_id: &TuiGatewaySessionId,
        text: &str,
    ) -> Result<TuiGatewayRunId, TuiGatewayError> {
        let run_id = TuiGatewayRunId::generate();
        let request_id = self
            .send_request(
                TuiGatewayMethod::PromptSubmit,
                PromptSubmitParams { session_id, text },
            )
            .await?;
        let _ = self.read_response_result(&request_id).await?;
        Ok(run_id)
    }

    async fn send_request<P: Serialize>(
        &mut self,
        method: TuiGatewayMethod,
        params: P,
    ) -> Result<TuiGatewayRequestId, TuiGatewayError> {
        let id = TuiGatewayRequestId::generate();
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id: id.as_str(),
            method: method.as_str(),
            params,
        };
        let mut line = serde_json::to_vec(&request)
            .map_err(|err| TuiGatewayError::Protocol(err.to_string()))?;
        line.push(b'\n');
        self.stdin.write_all(&line).await?;
        self.stdin.flush().await?;
        Ok(id)
    }

    async fn read_response_result(
        &mut self,
        request_id: &TuiGatewayRequestId,
    ) -> Result<Value, TuiGatewayError> {
        while let Some(frame) = self.frames_rx.recv().await {
            if let Some(result) = parse_response_result(&frame, request_id)? {
                return result;
            }
        }
        Err(TuiGatewayError::ResponseStreamClosed)
    }
}

fn parse_response_result(
    frame: &[u8],
    request_id: &TuiGatewayRequestId,
) -> Result<Option<Result<Value, TuiGatewayError>>, TuiGatewayError> {
    let Ok(envelope) = serde_json::from_slice::<JsonRpcResponse>(frame) else {
        return Ok(None);
    };
    if envelope.id.as_deref() != Some(request_id.as_str()) {
        return Ok(None);
    }
    if let Some(error) = envelope.error {
        return Ok(Some(Err(TuiGatewayError::Protocol(error.message))));
    }
    Ok(Some(Ok(envelope.result.unwrap_or(Value::Null))))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TuiGatewayMethod {
    SessionCreate,
    PromptSubmit,
}

impl TuiGatewayMethod {
    fn as_str(self) -> &'static str {
        match self {
            Self::SessionCreate => "session.create",
            Self::PromptSubmit => "prompt.submit",
        }
    }
}

#[derive(Serialize)]
struct JsonRpcRequest<'a, P> {
    jsonrpc: &'static str,
    id: &'a str,
    method: &'static str,
    params: P,
}

#[derive(Deserialize)]
struct JsonRpcResponse {
    id: Option<String>,
    result: Option<Value>,
    error: Option<JsonRpcError>,
}

#[derive(Deserialize)]
struct JsonRpcError {
    message: String,
}

#[derive(Serialize)]
struct SessionCreateParams {
    cols: TerminalColumnCount,
}

#[derive(Serialize)]
struct PromptSubmitParams<'a> {
    session_id: &'a TuiGatewaySessionId,
    text: &'a str,
}
