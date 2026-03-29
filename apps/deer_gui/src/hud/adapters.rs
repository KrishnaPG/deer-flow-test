//! Pure mapping functions from domain/world models to HUD view models.
//!
//! These functions contain no side effects and no ECS dependencies.
//! They transform domain types into presentation-ready view models,
//! keeping the HUD decoupled from the world representation.

use bevy::log::trace;

use crate::world::components::{WorldEntity, WorldEntityType};

use super::{EntityInspectorData, InspectorDetails};

// ---------------------------------------------------------------------------
// Mapping functions
// ---------------------------------------------------------------------------

/// Maps a [`WorldEntity`] to [`EntityInspectorData`] for the HUD inspector.
///
/// Delegates to [`entity_type_to_details`] for the variant-specific mapping
/// and wraps the result with entity identity fields.
pub fn world_entity_to_inspector(world_entity: &WorldEntity) -> EntityInspectorData {
    trace!(
        "world_entity_to_inspector — mapping entity '{}'",
        world_entity.entity_id
    );

    EntityInspectorData {
        entity_id: world_entity.entity_id.clone(),
        display_name: world_entity.entity_id.clone(),
        details: entity_type_to_details(&world_entity.entity_type),
    }
}

/// Maps a [`WorldEntityType`] to [`InspectorDetails`].
///
/// Each variant is mapped with sensible defaults for fields
/// not directly available on the world component (e.g. `tokens_used`).
pub fn entity_type_to_details(entity_type: &WorldEntityType) -> InspectorDetails {
    trace!("entity_type_to_details — mapping {:?}", entity_type);

    match entity_type {
        WorldEntityType::Agent(state) => InspectorDetails::Agent {
            state: format!("{:?}", state),
            tokens_used: 0,
            context_size: 0,
            pending_actions: Vec::new(),
        },
        WorldEntityType::Mission { progress } => InspectorDetails::Mission {
            progress: *progress,
            assigned_agents: 0,
            description: String::new(),
        },
        WorldEntityType::Artifact { artifact_type } => InspectorDetails::Artifact {
            artifact_type: format!("{:?}", artifact_type),
            size_bytes: 0,
            path: String::new(),
        },
        WorldEntityType::Swarm { agent_count } => InspectorDetails::Agent {
            state: format!("Swarm({})", agent_count),
            tokens_used: 0,
            context_size: 0,
            pending_actions: Vec::new(),
        },
    }
}
