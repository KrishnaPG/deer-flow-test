//! Outbound command and inbound envelope types for the bridge protocol.
//!
//! These structures define the JSON wire format used to communicate
//! with the Python bridge subprocess over stdin/stdout.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A command sent to the bridge process via stdin.
#[derive(Debug, Serialize)]
pub struct OutboundCommand<'a, T: Serialize> {
    pub id: &'a str,
    pub command: &'a str,
    pub payload: T,
}

/// A parsed JSON envelope received from the bridge process via stdout.
#[derive(Debug, Deserialize)]
pub struct InboundEnvelope {
    pub kind: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub ok: Option<bool>,
    #[serde(default)]
    pub result: Option<Value>,
    #[serde(default)]
    pub event: Option<String>,
    #[serde(default)]
    pub data: Option<Value>,
    #[serde(default)]
    pub error: Option<String>,
}
