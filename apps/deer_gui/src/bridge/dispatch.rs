//! Envelope dispatch — routes raw inbound envelopes to the appropriate
//! [`BridgeEvent`] via the [`adapter`](super::adapter) module.
//!
//! This module is intentionally thin: it owns the routing logic only,
//! delegating all JSON→Domain parsing to `adapter`.

use bevy::log::{debug, error, info, warn};
use crossbeam_channel::Sender;

use super::adapter;
use super::commands::InboundEnvelope;
use super::events::{BridgeEvent, BridgeResponse};

/// Route a deserialized envelope to the appropriate [`BridgeEvent`].
pub fn dispatch_envelope(env: InboundEnvelope, events_tx: &Sender<BridgeEvent>) {
    debug!(
        "dispatch_envelope: kind={}, id={:?}, event={:?}, ok={:?}",
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
            debug!("dispatch_response: ok id={id}");
            match adapter::parse_response(result) {
                Ok(parsed) => BridgeEvent::Response {
                    request_id: Some(id),
                    response: parsed,
                },
                Err(err) => {
                    error!("Failed to parse response payload: {err}");
                    BridgeEvent::Error {
                        message: err.to_string(),
                    }
                }
            }
        }
        (Some(id), true, None, _) => {
            debug!("dispatch_response: ack id={id}");
            BridgeEvent::Response {
                request_id: Some(id),
                response: BridgeResponse::Ack,
            }
        }
        (id, false, _, err) => {
            let msg = err.unwrap_or_else(|| "Bridge command failed".to_string());
            error!("dispatch_response: error id={id:?}: {msg}");
            BridgeEvent::Error { message: msg }
        }
        _ => {
            error!("dispatch_response: malformed envelope");
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
        error!("dispatch_event: missing event name");
        let _ = events_tx.send(BridgeEvent::Error {
            message: "Missing event name".to_string(),
        });
        return;
    };

    debug!("dispatch_event: {event_name} id={:?}", env.id);

    let event = match event_name.as_str() {
        "message" => parse_or_error(env.data, "message", adapter::parse_message_event),
        "state" => parse_or_error(env.data, "state", adapter::parse_state_event),
        "suggestions" => parse_or_error(env.data, "suggestions", adapter::parse_suggestions_event),
        "done" => parse_or_error(env.data, "done", adapter::parse_done_event),
        "error" => adapter::parse_error_event(env.data),
        _ => {
            warn!("Unknown bridge event type: {event_name}");
            BridgeEvent::Error {
                message: format!("Unknown bridge event: {event_name}"),
            }
        }
    };
    let _ = events_tx.send(event);
}

/// Unwrap `Option<Value>` and call an adapter parse function, converting
/// `None` or adapter errors into [`BridgeEvent::Error`].
fn parse_or_error(
    data: Option<serde_json::Value>,
    label: &str,
    parse_fn: fn(serde_json::Value) -> Result<BridgeEvent, adapter::AdapterError>,
) -> BridgeEvent {
    let Some(value) = data else {
        error!("dispatch_event/{label}: missing data payload");
        return BridgeEvent::Error {
            message: format!("Invalid {label} event payload"),
        };
    };
    match parse_fn(value) {
        Ok(event) => event,
        Err(err) => {
            error!("dispatch_event/{label}: {err}");
            BridgeEvent::Error {
                message: format!("Invalid {label} event payload: {err}"),
            }
        }
    }
}
