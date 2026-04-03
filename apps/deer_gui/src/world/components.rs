//! ECS components for world entities.
//!
//! These components are attached to Bevy entities to represent domain
//! objects such as agents, swarms, missions, and artifacts.

use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Core world entity component
// ---------------------------------------------------------------------------

/// Primary identity component for a domain entity in the world.
#[derive(Component, Debug)]
pub struct WorldEntity {
    /// Unique domain identifier (e.g. thread ID, agent ID).
    pub entity_id: String,
    /// The kind of entity and its associated state.
    pub entity_type: WorldEntityType,
}

/// Discriminated type of a world entity with inline state.
#[derive(Clone, Debug)]
pub enum WorldEntityType {
    /// A single AI agent.
    Agent(AgentState),
    /// A cluster of agents.
    Swarm { agent_count: u32 },
    /// A running mission / task.
    Mission { progress: f32 },
    /// A produced artifact.
    Artifact { artifact_type: ArtifactKind },
}

/// Lifecycle state of an agent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgentState {
    Idle,
    Working,
    Error,
    ApprovalNeeded,
}

/// Classification of an artifact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArtifactKind {
    Document,
    Index,
    Embedding,
    Raw,
}

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Marker: entity is an agent.
#[derive(Component)]
pub struct Agent;

/// Marker: entity is a swarm cluster.
#[derive(Component)]
pub struct Swarm;

/// Marker: entity is a mission / task.
#[derive(Component)]
pub struct Mission;

/// Marker: entity is an artifact.
#[derive(Component)]
pub struct Artifact;

/// Marker: entity can be selected by the player.
#[derive(Component)]
pub struct Selectable;

/// Marker: entity is currently selected.
#[derive(Component)]
pub struct Selected;

/// Pulsing beacon effect attached to an entity.
#[derive(Component)]
pub struct PulsingBeacon {
    /// Pulse frequency in Hz.
    pub frequency: f32,
}

use crate::shell::{CanonicalEntityRef, CanonicalRecordFamily};

impl WorldEntity {
    pub fn to_canonical_ref(&self) -> CanonicalEntityRef {
        let family = match self.entity_type {
            WorldEntityType::Agent(_) => CanonicalRecordFamily::Agent,
            WorldEntityType::Mission { .. } => CanonicalRecordFamily::Mission,
            WorldEntityType::Artifact { .. } => CanonicalRecordFamily::Artifact,
            WorldEntityType::Swarm { .. } => CanonicalRecordFamily::Mission,
        };

        CanonicalEntityRef {
            family,
            canonical_id: self.entity_id.clone(),
            correlation_id: None,
            lineage_id: None,
        }
    }
}
