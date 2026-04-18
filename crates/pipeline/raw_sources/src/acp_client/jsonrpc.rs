use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::acp_client::ids::AcpJsonRpcRequestId;

/// Minimal owned JSON-RPC frame snapshot kept alongside typed ACP handling.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AcpJsonRpcFrame {
    pub jsonrpc: String,
    pub request_id: Option<AcpJsonRpcRequestId>,
    pub method: Option<String>,
    pub payload: Box<RawValue>,
}

impl AcpJsonRpcFrame {
    pub fn from_raw_json(payload: Box<RawValue>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            request_id: extract_request_id(payload.as_ref()),
            method: extract_method_name(payload.as_ref()),
            payload,
        }
    }
}

fn extract_request_id(payload: &RawValue) -> Option<AcpJsonRpcRequestId> {
    let value = serde_json::from_str::<serde_json::Value>(payload.get()).ok()?;
    let request_id = value.get("id")?;
    if let Some(text) = request_id.as_str() {
        return Some(AcpJsonRpcRequestId::new(text));
    }
    request_id
        .as_i64()
        .map(|number| AcpJsonRpcRequestId::new(number.to_string()))
}

fn extract_method_name(payload: &RawValue) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(payload.get())
        .ok()?
        .get("method")?
        .as_str()
        .map(ToString::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_names_can_be_extracted_from_raw_payload() {
        let payload = serde_json::value::to_raw_value(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "session/prompt"
        }))
        .unwrap();

        assert_eq!(
            extract_method_name(payload.as_ref()).as_deref(),
            Some("session/prompt")
        );
    }

    #[test]
    fn numeric_request_ids_can_be_extracted_from_raw_payload() {
        let payload = serde_json::value::to_raw_value(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 42
        }))
        .unwrap();

        assert_eq!(extract_request_id(payload.as_ref()).unwrap().as_str(), "42");
    }
}
