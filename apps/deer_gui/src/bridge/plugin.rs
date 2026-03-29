//! Bevy plugin that wraps the bridge client as ECS resources.
//!
//! Spawns the bridge subprocess at startup and polls for events each frame,
//! making them available via the [`BridgeEventQueue`] resource.

use std::sync::Arc;

use bevy::log::{error, info, trace};
use bevy::prelude::*;
use crossbeam_channel::Receiver;

use super::client::BridgeClient;
use super::events::BridgeEvent;
use super::orchestrator::OrchestratorClient;
use crate::constants::perf::EVENT_QUEUE_CAPACITY;
use crate::constants::timing::BRIDGE_POLL_MAX_PER_FRAME;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Holds the orchestrator client handle (trait object).
///
/// Using `Arc<dyn OrchestratorClient>` allows the concrete transport to be
/// swapped without changing downstream system signatures.
#[derive(Resource)]
pub struct BridgeClientResource {
    pub client: Option<Arc<dyn OrchestratorClient>>,
}

impl BridgeClientResource {
    /// Convenience accessor — returns a reference to the client or `None`.
    ///
    /// Avoids the verbose `res.client.as_ref().map(Arc::as_ref)` pattern
    /// that downstream systems would otherwise repeat.
    pub fn client_ref(&self) -> Option<&dyn OrchestratorClient> {
        self.client.as_deref()
    }
}

/// Holds the event receiver channel for the bridge subprocess.
///
/// Separated from [`BridgeClientResource`] so that event polling is
/// independent of the orchestrator API surface.
#[derive(Resource)]
pub struct BridgeEventReceiver {
    pub rx: Option<Receiver<BridgeEvent>>,
}

/// Per-frame queue of bridge events for downstream systems to consume.
///
/// Pre-allocates with [`EVENT_QUEUE_CAPACITY`] so the vec never reallocates
/// during normal frame draining (up to [`BRIDGE_POLL_MAX_PER_FRAME`] events).
#[derive(Resource)]
pub struct BridgeEventQueue {
    pub events: Vec<BridgeEvent>,
}

impl Default for BridgeEventQueue {
    fn default() -> Self {
        Self {
            events: Vec::with_capacity(EVENT_QUEUE_CAPACITY),
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Registers bridge resources and systems.
pub struct BridgePlugin;

impl Plugin for BridgePlugin {
    fn build(&self, app: &mut App) {
        info!("BridgePlugin::build — registering bridge resources and systems");
        app.insert_resource(BridgeClientResource { client: None })
            .insert_resource(BridgeEventReceiver { rx: None })
            .init_resource::<BridgeEventQueue>()
            .add_systems(Startup, bridge_startup_system)
            .add_systems(Update, bridge_poll_system);
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns the Python bridge subprocess on startup.
fn bridge_startup_system(
    mut client_res: ResMut<BridgeClientResource>,
    mut event_rx_res: ResMut<BridgeEventReceiver>,
) {
    info!("bridge_startup_system — spawning bridge client");
    match BridgeClient::spawn() {
        Ok(client) => {
            info!("Bridge client spawned successfully");
            let rx = client.events().clone();
            let arc_client: Arc<dyn OrchestratorClient> = Arc::new(client);
            client_res.client = Some(arc_client);
            event_rx_res.rx = Some(rx);
        }
        Err(e) => {
            error!("Failed to spawn bridge client: {e}");
        }
    }
}

/// Drains up to [`BRIDGE_POLL_MAX_PER_FRAME`] events from the bridge channel
/// into the [`BridgeEventQueue`] resource each frame.
fn bridge_poll_system(event_rx_res: Res<BridgeEventReceiver>, mut queue: ResMut<BridgeEventQueue>) {
    queue.events.clear();
    if let Some(rx) = &event_rx_res.rx {
        for _ in 0..BRIDGE_POLL_MAX_PER_FRAME {
            match rx.try_recv() {
                Ok(event) => {
                    trace!("bridge_poll_system — received event: {:?}", event);
                    queue.events.push(event);
                }
                Err(_) => break,
            }
        }
    }
}
