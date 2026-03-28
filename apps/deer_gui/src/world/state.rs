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
