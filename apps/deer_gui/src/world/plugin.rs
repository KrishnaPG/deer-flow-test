//! Bevy plugin for the world simulation layer.
//!
//! Registers the [`WorldState`] resource and the bridge event handler system.

use bevy::log::info;
use bevy::prelude::*;

use super::state::WorldState;
use super::systems::bridge_event_handler_system;

/// Registers world state resources and systems.
pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        info!("WorldPlugin::build — registering world resources and systems");
        app.init_resource::<WorldState>()
            .add_systems(Update, bridge_event_handler_system);
    }
}
