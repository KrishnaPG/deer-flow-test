//! World systems — process bridge events and update world state.

use bevy::log::{debug, info, trace, warn};
use bevy::prelude::*;

use crate::bridge::{BridgeEvent, BridgeEventQueue};

use super::state::WorldState;

/// Consumes bridge events from [`BridgeEventQueue`] and updates [`WorldState`].
///
/// This system runs every frame during `Update` and translates bridge-level
/// events into world-state mutations and entity commands.
pub fn bridge_event_handler_system(
    queue: Res<BridgeEventQueue>,
    mut world_state: ResMut<WorldState>,
    _commands: Commands,
) {
    if queue.events.is_empty() {
        return;
    }

    trace!(
        "bridge_event_handler_system — processing {} events",
        queue.events.len()
    );

    for event in &queue.events {
        match event {
            BridgeEvent::Ready => {
                info!("Bridge ready — marking system online");
                world_state.system_health.system_online = true;
            }
            BridgeEvent::StreamMessage { thread_id, message } => {
                debug!(
                    "Stream message: thread={thread_id} role={} len={}",
                    message.role,
                    message.content.len()
                );
            }
            BridgeEvent::State { state } => {
                debug!(
                    "State update: thread={} title={:?}",
                    state.thread_id, state.title
                );
            }
            BridgeEvent::Suggestions {
                thread_id,
                suggestions,
            } => {
                debug!(
                    "Suggestions: thread={thread_id} count={}",
                    suggestions.len()
                );
            }
            BridgeEvent::Done { thread_id, usage } => {
                debug!("Stream done: thread={thread_id} usage={usage:?}");
            }
            BridgeEvent::Error { message } => {
                warn!("Bridge error: {message}");
            }
            BridgeEvent::BridgeExited => {
                warn!("Bridge process exited — marking system offline");
                world_state.system_health.system_online = false;
            }
            BridgeEvent::Response {
                request_id,
                response,
            } => {
                debug!("Bridge response: request_id={request_id:?} response={response:?}");
            }
        }
    }
}
