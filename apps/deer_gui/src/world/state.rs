//! World state resource — centralized runtime state for the world simulation.
//!
//! Tracks system health metrics and a registry mapping entity IDs to
//! Bevy `Entity` handles for fast lookup.

use std::collections::HashMap;

use bevy::prelude::*;

/// Top-level world state, accessible as a Bevy resource.
#[derive(Resource, Default)]
pub struct WorldState {
    /// Aggregate system health metrics.
    pub system_health: SystemHealth,
    /// Maps domain entity IDs (strings) to Bevy ECS entities.
    pub entity_registry: HashMap<String, Entity>,
}

impl WorldState {
    /// Convenience accessor: is the system currently online?
    pub fn system_online(&self) -> bool {
        self.system_health.system_online
    }

    /// Registers a domain entity ID → Bevy `Entity` mapping.
    ///
    /// Returns `None` if the ID was new, or `Some(old_entity)` if
    /// an existing mapping was replaced.
    pub fn register(&mut self, entity_id: String, entity: Entity) -> Option<Entity> {
        bevy::log::debug!("WorldState::register — id={entity_id} entity={entity:?}");
        self.entity_registry.insert(entity_id, entity)
    }

    /// Removes the mapping for `entity_id`.
    ///
    /// Returns `Some(entity)` if the ID was present, `None` otherwise.
    pub fn unregister(&mut self, entity_id: &str) -> Option<Entity> {
        bevy::log::debug!("WorldState::unregister — id={entity_id}");
        self.entity_registry.remove(entity_id)
    }

    /// Looks up the Bevy `Entity` for a given domain ID.
    pub fn lookup(&self, entity_id: &str) -> Option<Entity> {
        bevy::log::trace!("WorldState::lookup — id={entity_id}");
        self.entity_registry.get(entity_id).copied()
    }

    /// Returns the number of registered domain entities.
    pub fn entity_count(&self) -> usize {
        self.entity_registry.len()
    }
}

/// Aggregate health metrics for the running system.
#[derive(Debug, Clone, Default)]
pub struct SystemHealth {
    /// Number of agents currently working.
    pub active_agents: u32,
    /// Number of agents in retry backoff.
    pub retrying_agents: u32,
    /// Number of agents in a failed state.
    pub failed_agents: u32,
    /// Current token throughput (tokens/sec).
    pub tokens_per_sec: f32,
    /// Estimated cost rate (USD/hour).
    pub cost_per_hour: f32,
    /// Whether the bridge subprocess is connected and responsive.
    pub system_online: bool,
    /// Rolling average response latency (milliseconds).
    pub avg_latency_ms: f32,
    /// Fraction of requests that resulted in errors (0.0–1.0).
    pub error_rate: f32,
    /// Fraction of CPU capacity in use (0.0–1.0).
    pub cpu_load: f32,
}
