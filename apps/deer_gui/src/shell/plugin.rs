use bevy::prelude::*;

use super::state::ShellState;
use super::systems::{selection_broker_system, ShellSelectionRequest};

pub struct ShellPlugin;

impl Plugin for ShellPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShellState>();
        app.add_message::<ShellSelectionRequest>()
            .add_systems(Update, selection_broker_system);
    }
}
