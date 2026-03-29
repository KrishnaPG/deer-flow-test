//! Centralized JSON→Domain parsing adapter.
//!
//! Pure functions that convert raw [`serde_json::Value`] payloads into
//! strongly-typed domain values. All functions take ownership of the
//! incoming `Value` to avoid cloning — callers move values in.

use bevy::log::{debug, error};
use serde_json::Value;

use super::events::{BridgeEvent, BridgeResponse};
use crate::models::{AppStateUpdate, ChatMessage, Usage};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// A structured error from adapter conversion.
#[derive(Debug)]
pub struct AdapterError {
    pub context: &'static str,
    pub detail: String,
}

impl std::fmt::Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.context, self.detail)
    }
}

impl std::error::Error for AdapterError {}

impl From<serde_json::Error> for AdapterError {
    fn from(err: serde_json::Error) -> Self {
        Self {
            context: "serde",
            detail: err.to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Response parsing
// ---------------------------------------------------------------------------

/// Parse a response `result` [`Value`] into a typed [`BridgeResponse`].
///
/// Takes ownership so that inner sub-values can be moved directly into
/// `serde_json::from_value` without cloning.
pub fn parse_response(value: Value) -> Result<BridgeResponse, AdapterError> {
    let mut obj = match value {
        Value::Object(map) => map,
        other => {
            debug!("parse_response: non-object value, returning Raw");
            return Ok(BridgeResponse::Raw(other));
        }
    };

    if let Some(v) = obj.remove("threads") {
        debug!("parse_response: matched key 'threads'");
        return serde_json::from_value(v)
            .map(BridgeResponse::Threads)
            .map_err(|e| AdapterError {
                context: "parse_response/threads",
                detail: e.to_string(),
            });
    }
    if let Some(v) = obj.remove("thread") {
        debug!("parse_response: matched key 'thread'");
        return serde_json::from_value(v)
            .map(BridgeResponse::Thread)
            .map_err(|e| AdapterError {
                context: "parse_response/thread",
                detail: e.to_string(),
            });
    }
    if let Some(v) = obj.remove("created_thread") {
        debug!("parse_response: matched key 'created_thread'");
        return serde_json::from_value(v)
            .map(BridgeResponse::CreatedThread)
            .map_err(|e| AdapterError {
                context: "parse_response/created_thread",
                detail: e.to_string(),
            });
    }
    if let Some(v) = obj.remove("models") {
        debug!("parse_response: matched key 'models'");
        return serde_json::from_value(v)
            .map(BridgeResponse::Models)
            .map_err(|e| AdapterError {
                context: "parse_response/models",
                detail: e.to_string(),
            });
    }
    if let Some(v) = obj.remove("renamed_thread") {
        debug!("parse_response: matched key 'renamed_thread'");
        return serde_json::from_value(v)
            .map(BridgeResponse::Renamed)
            .map_err(|e| AdapterError {
                context: "parse_response/renamed_thread",
                detail: e.to_string(),
            });
    }
    if let Some(v) = obj.remove("deleted_thread_id") {
        debug!("parse_response: matched key 'deleted_thread_id'");
        let id = v.as_str().map(ToOwned::to_owned).ok_or(AdapterError {
            context: "parse_response/deleted_thread_id",
            detail: "expected string value".to_string(),
        })?;
        return Ok(BridgeResponse::Deleted(id));
    }
    if let Some(v) = obj.remove("artifact") {
        debug!("parse_response: matched key 'artifact'");
        return serde_json::from_value(v)
            .map(BridgeResponse::Artifact)
            .map_err(|e| AdapterError {
                context: "parse_response/artifact",
                detail: e.to_string(),
            });
    }

    debug!(
        "parse_response: no known key — remaining keys={:?}",
        obj.keys().collect::<Vec<_>>()
    );
    Ok(BridgeResponse::Raw(Value::Object(obj)))
}

// ---------------------------------------------------------------------------
// Event parsing
// ---------------------------------------------------------------------------

/// Parse a `"message"` event data payload into [`BridgeEvent::StreamMessage`].
pub fn parse_message_event(data: Value) -> Result<BridgeEvent, AdapterError> {
    let mut obj = into_object(data, "parse_message_event")?;

    let thread_id = remove_string(&mut obj, "thread_id", "parse_message_event")?;
    let msg_value = obj.remove("message").ok_or(AdapterError {
        context: "parse_message_event",
        detail: "missing 'message' key".to_string(),
    })?;

    let message: ChatMessage = serde_json::from_value(msg_value).map_err(|e| AdapterError {
        context: "parse_message_event/ChatMessage",
        detail: e.to_string(),
    })?;

    debug!(
        "parse_message_event: thread={thread_id} role={} len={}",
        message.role,
        message.content.len()
    );
    Ok(BridgeEvent::StreamMessage { thread_id, message })
}

/// Parse a `"state"` event data payload into [`BridgeEvent::State`].
pub fn parse_state_event(data: Value) -> Result<BridgeEvent, AdapterError> {
    let state: AppStateUpdate = serde_json::from_value(data).map_err(|e| AdapterError {
        context: "parse_state_event",
        detail: e.to_string(),
    })?;

    debug!(
        "parse_state_event: thread={} title={:?}",
        state.thread_id, state.title
    );
    Ok(BridgeEvent::State { state })
}

/// Parse a `"suggestions"` event data payload into [`BridgeEvent::Suggestions`].
pub fn parse_suggestions_event(data: Value) -> Result<BridgeEvent, AdapterError> {
    let mut obj = into_object(data, "parse_suggestions_event")?;

    let thread_id = remove_string(&mut obj, "thread_id", "parse_suggestions_event")?;
    let suggestions_value = obj.remove("suggestions").ok_or(AdapterError {
        context: "parse_suggestions_event",
        detail: "missing 'suggestions' key".to_string(),
    })?;

    let suggestions: Vec<String> =
        serde_json::from_value(suggestions_value).map_err(|e| AdapterError {
            context: "parse_suggestions_event/Vec<String>",
            detail: e.to_string(),
        })?;

    debug!(
        "parse_suggestions_event: thread={thread_id} count={}",
        suggestions.len()
    );
    Ok(BridgeEvent::Suggestions {
        thread_id,
        suggestions,
    })
}

/// Parse a `"done"` event data payload into [`BridgeEvent::Done`].
pub fn parse_done_event(data: Value) -> Result<BridgeEvent, AdapterError> {
    let mut obj = into_object(data, "parse_done_event")?;

    let thread_id = remove_string(&mut obj, "thread_id", "parse_done_event")?;
    let usage: Option<Usage> = obj
        .remove("usage")
        .map(serde_json::from_value)
        .transpose()
        .map_err(|e| AdapterError {
            context: "parse_done_event/Usage",
            detail: e.to_string(),
        })?;

    debug!("parse_done_event: thread={thread_id} usage={usage:?}");
    Ok(BridgeEvent::Done { thread_id, usage })
}

/// Parse an `"error"` event data payload into [`BridgeEvent::Error`].
///
/// Best-effort: if the payload is missing or malformed we still produce
/// a reasonable error event rather than failing.
pub fn parse_error_event(data: Option<Value>) -> BridgeEvent {
    let message = data
        .and_then(|value| {
            let mut obj = match value {
                Value::Object(m) => m,
                _ => return None,
            };
            obj.remove("message")
                .and_then(|v| v.as_str().map(ToOwned::to_owned))
        })
        .unwrap_or_else(|| "Bridge stream error".to_string());

    error!("parse_error_event: {message}");
    BridgeEvent::Error { message }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Convert a `Value` into a `serde_json::Map`, returning an error if it
/// is not an object.
fn into_object(
    value: Value,
    context: &'static str,
) -> Result<serde_json::Map<String, Value>, AdapterError> {
    match value {
        Value::Object(map) => Ok(map),
        _ => Err(AdapterError {
            context,
            detail: "expected JSON object".to_string(),
        }),
    }
}

/// Remove a string field from a JSON object map, returning an error if
/// the key is missing or not a string.
fn remove_string(
    obj: &mut serde_json::Map<String, Value>,
    key: &'static str,
    context: &'static str,
) -> Result<String, AdapterError> {
    obj.remove(key)
        .and_then(|v| match v {
            Value::String(s) => Some(s),
            _ => None,
        })
        .ok_or(AdapterError {
            context,
            detail: format!("missing or non-string '{key}'"),
        })
}
