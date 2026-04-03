use bevy::log::debug;

use super::view_hosts::{MINIMAP_HOST, WORLD_HOST};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldSelectionBinding {
    pub selection_id: &'static str,
    pub source_hosts: [&'static str; 2],
}

pub fn bind_world_selection(selection_id: &'static str) -> WorldSelectionBinding {
    debug!("composition::world_projection_binding::bind_world_selection");

    WorldSelectionBinding {
        selection_id,
        source_hosts: [WORLD_HOST, MINIMAP_HOST],
    }
}
