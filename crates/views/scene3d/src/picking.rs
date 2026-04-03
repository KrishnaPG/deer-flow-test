use crate::{SpatialIndex, SpatialRay};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldPickEvent {
    pub selection_id: String,
    pub focus_target: &'static str,
    pub command_armed: bool,
}

pub fn emit_world_pick(index: &SpatialIndex, ray: SpatialRay) -> Option<WorldPickEvent> {
    index.first_hit(&ray).map(|hit| WorldPickEvent {
        selection_id: hit.selection_id,
        focus_target: hit.focus_target,
        command_armed: false,
    })
}
