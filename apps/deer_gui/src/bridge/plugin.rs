//! Bevy plugin that wraps the bridge client as ECS resources.
//!
//! Spawns the bridge subprocess at startup and polls for events each frame,
//! making them available via the [`BridgeEventQueue`] resource.

use bevy::log::{error, info, trace};
use bevy::prelude::*;

use super::client::BridgeClient;
use super::events::BridgeEvent;
use crate::constants::timing::BRIDGE_POLL_MAX_PER_FRAME;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Holds the bridge subprocess handle.
#[derive(Resource)]
pub struct BridgeClientResource {
    pub client: Option<BridgeClient>,
}

/// Per-frame queue of bridge events for downstream systems to consume.
#[derive(Resource, Default)]
pub struct BridgeEventQueue {
    pub events: Vec<BridgeEvent>,
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
            .init_resource::<BridgeEventQueue>()
            .add_systems(Startup, bridge_startup_system)
            .add_systems(Update, bridge_poll_system);
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns the Python bridge subprocess on startup.
fn bridge_startup_system(mut res: ResMut<BridgeClientResource>) {
    info!("bridge_startup_system — spawning bridge client");
    match BridgeClient::spawn() {
        Ok(client) => {
            info!("Bridge client spawned successfully");
            res.client = Some(client);
        }
        Err(e) => {
            error!("Failed to spawn bridge client: {e}");
        }
    }
}

/// Drains up to [`BRIDGE_POLL_MAX_PER_FRAME`] events from the bridge channel
/// into the [`BridgeEventQueue`] resource each frame.
fn bridge_poll_system(client_res: Res<BridgeClientResource>, mut queue: ResMut<BridgeEventQueue>) {
    queue.events.clear();
    if let Some(client) = &client_res.client {
        for _ in 0..BRIDGE_POLL_MAX_PER_FRAME {
            match client.events().try_recv() {
                Ok(event) => {
                    trace!("bridge_poll_system — received event: {:?}", event);
                    queue.events.push(event);
                }
                Err(_) => break,
            }
        }
    }
}
