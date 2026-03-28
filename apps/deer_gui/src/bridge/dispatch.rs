//! Envelope dispatch and response parsing for the bridge protocol.
//!
//! These functions are called from the reader thread to convert raw
//! JSON envelopes into typed [`BridgeEvent`] values.

use bevy::log::{debug, error, info, warn};
use crossbeam_channel::Sender;
use serde_json::Value;

use super::commands::InboundEnvelope;
use super::events::{BridgeEvent, BridgeResponse};
use crate::models::{AppStateUpdate, ChatMessage, Usage};

/// Route a deserialized envelope to the appropriate [`BridgeEvent`].
pub(super) fn dispatch_envelope(env: InboundEnvelope, events_tx: &Sender<BridgeEvent>) {
    debug!(
        "Dispatch envelope: kind={}, id={:?}, event={:?}, ok={:?}",
        env.kind, env.id, env.event, env.ok
    );

    match env.kind.as_str() {
        "ready" => {
            info!("Python bridge ready");
            let _ = events_tx.send(BridgeEvent::Ready);
        }
        "response" => dispatch_response(env, events_tx),
        "event" => dispatch_event(env, events_tx),
        other => {
            warn!("Unknown envelope kind: {other}");
            let _ = events_tx.send(BridgeEvent::Error {
                message: format!("Unknown envelope kind: {other}"),
            });
        }
    }
}

/// Handle a `"response"` envelope.
fn dispatch_response(env: InboundEnvelope, events_tx: &Sender<BridgeEvent>) {
    let response = match (env.id, env.ok.unwrap_or(false), env.result, env.error) {
        (Some(id), true, Some(result), _) => {
            debug!("Response ok id={id}: {result}");
            match parse_response(&result) {
                Ok(parsed) => BridgeEvent::Response {
                    request_id: Some(id),
                    response: parsed,
                },
                Err(err) => {
                    error!("Failed to parse response payload: {err}");
                    BridgeEvent::Error { message: err }
                }
            }
        }
        (Some(id), true, None, _) => {
            debug!("Response ack id={id}");
            BridgeEvent::Response {
                request_id: Some(id),
                response: BridgeResponse::Ack,
            }
        }
        (id, false, _, err) => {
            let msg = err.unwrap_or_else(|| "Bridge command failed".to_string());
            error!("Response error id={id:?}: {msg}");
            BridgeEvent::Error { message: msg }
        }
        _ => {
            error!("Malformed bridge response");
            BridgeEvent::Error {
                message: "Malformed bridge response".to_string(),
            }
        }
    };
    let _ = events_tx.send(response);
}

/// Handle an `"event"` envelope.
fn dispatch_event(env: InboundEnvelope, events_tx: &Sender<BridgeEvent>) {
    let Some(event_name) = env.event else {
        error!("Event envelope missing event name");
        let _ = events_tx.send(BridgeEvent::Error {
            message: "Missing event name".to_string(),
        });
        return;
    };

    debug!("Event: {event_name} id={:?}", env.id);

    let event = match event_name.as_str() {
        "message" => dispatch_message_event(env.data),
        "state" => dispatch_state_event(env.data),
        "suggestions" => dispatch_suggestions_event(env.data),
        "done" => dispatch_done_event(env.data),
        "error" => {
            let msg = env
                .data
                .and_then(|value| {
                    value
                        .get("message")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned)
                })
                .unwrap_or_else(|| "Bridge stream error".to_string());
            error!("Bridge stream error event: {msg}");
            BridgeEvent::Error { message: msg }
        }
        _ => {
            warn!("Unknown bridge event type: {event_name}");
            BridgeEvent::Error {
                message: format!("Unknown bridge event: {event_name}"),
            }
        }
    };
    let _ = events_tx.send(event);
}

fn dispatch_message_event(data: Option<Value>) -> BridgeEvent {
    data.and_then(|value| {
        let thread_id = value.get("thread_id")?.as_str()?.to_string();
        let msg_value = value.get("message")?.clone();
        match serde_json::from_value::<ChatMessage>(msg_value.clone()) {
            Ok(message) => {
                debug!(
                    "Stream message: thread={thread_id} role={} len={}",
                    message.role,
                    message.content.len()
                );
                Some(BridgeEvent::StreamMessage { thread_id, message })
            }
            Err(err) => {
                error!("Failed to deserialize ChatMessage: {err} — raw: {msg_value}");
                None
            }
        }
    })
    .unwrap_or_else(|| BridgeEvent::Error {
        message: "Invalid message event payload".to_string(),
    })
}

fn dispatch_state_event(data: Option<Value>) -> BridgeEvent {
    data.and_then(
        |value| match serde_json::from_value::<AppStateUpdate>(value.clone()) {
            Ok(state) => {
                debug!(
                    "State update: thread={} title={:?}",
                    state.thread_id, state.title
                );
                Some(BridgeEvent::State { state })
            }
            Err(err) => {
                error!("Failed to deserialize AppStateUpdate: {err} — raw: {value}");
                None
            }
        },
    )
    .unwrap_or_else(|| BridgeEvent::Error {
        message: "Invalid state event payload".to_string(),
    })
}

fn dispatch_suggestions_event(data: Option<Value>) -> BridgeEvent {
    data.and_then(|value| {
        let thread_id = value.get("thread_id")?.as_str()?.to_string();
        let suggestions =
            serde_json::from_value::<Vec<String>>(value.get("suggestions")?.clone()).ok()?;
        debug!(
            "Suggestions: thread={thread_id} count={}",
            suggestions.len()
        );
        Some(BridgeEvent::Suggestions {
            thread_id,
            suggestions,
        })
    })
    .unwrap_or_else(|| BridgeEvent::Error {
        message: "Invalid suggestions event payload".to_string(),
    })
}

fn dispatch_done_event(data: Option<Value>) -> BridgeEvent {
    data.and_then(|value| {
        let thread_id = value.get("thread_id")?.as_str()?.to_string();
        let usage = value
            .get("usage")
            .cloned()
            .and_then(|u| serde_json::from_value::<Usage>(u).ok());
        info!("Stream done: thread={thread_id} usage={usage:?}");
        Some(BridgeEvent::Done { thread_id, usage })
    })
    .unwrap_or_else(|| BridgeEvent::Error {
        message: "Invalid done event payload".to_string(),
    })
}

// ---------------------------------------------------------------------------
// Response parsing
// ---------------------------------------------------------------------------

/// Parse a JSON `result` value into a typed [`BridgeResponse`].
fn parse_response(value: &Value) -> Result<BridgeResponse, String> {
    if let Some(threads) = value.get("threads") {
        debug!("Parsed response: Threads");
        return serde_json::from_value(threads.clone())
            .map(BridgeResponse::Threads)
            .map_err(|err| err.to_string());
    }
    if let Some(thread) = value.get("thread") {
        debug!("Parsed response: Thread");
        return serde_json::from_value(thread.clone())
            .map(BridgeResponse::Thread)
            .map_err(|err| err.to_string());
    }
    if let Some(thread) = value.get("created_thread") {
        debug!("Parsed response: CreatedThread");
        return serde_json::from_value(thread.clone())
            .map(BridgeResponse::CreatedThread)
            .map_err(|err| err.to_string());
    }
    if let Some(models) = value.get("models") {
        debug!("Parsed response: Models");
        return serde_json::from_value(models.clone())
            .map(BridgeResponse::Models)
            .map_err(|err| err.to_string());
    }
    if let Some(thread) = value.get("renamed_thread") {
        debug!("Parsed response: Renamed");
        return serde_json::from_value(thread.clone())
            .map(BridgeResponse::Renamed)
            .map_err(|err| err.to_string());
    }
    if let Some(thread_id) = value.get("deleted_thread_id").and_then(Value::as_str) {
        debug!("Parsed response: Deleted({thread_id})");
        return Ok(BridgeResponse::Deleted(thread_id.to_string()));
    }
    if let Some(artifact) = value.get("artifact") {
        debug!("Parsed response: Artifact");
        return serde_json::from_value(artifact.clone())
            .map(BridgeResponse::Artifact)
            .map_err(|err| err.to_string());
    }

    debug!(
        "Parsed response: Raw — keys={:?}",
        value.as_object().map(|o| o.keys().collect::<Vec<_>>())
    );
    Ok(BridgeResponse::Raw(value.clone()))
}
