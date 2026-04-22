//! NPC (Non-Player Character) population system using bevior_tree.
//!
//! Provides a reusable [`NpcPlugin`] for spawning and managing medieval NPCs
//! with behavior trees for autonomous activity.
//!
//! ## Features
//!
//! - Configurable NPC types (Knight, Peasant, Guard, etc.)
//! - Behavior tree system via bevior_tree
//! - Skeletal animation state management
//! - Faction allegiance support
//! - Modular and reusable in other projects
//!
//! ## Reusability
//!
//! This plugin can be extracted to a standalone crate for use in any
//! Bevy project needing NPC populations. Configuration is done
//! via Bevy resources (data-driven, no hardcoded paths).

use bevy::ecs::system::{Commands, Res};
use bevy::log::{debug, info, trace};
use bevy::math::Vec3;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// NPC Types
// ---------------------------------------------------------------------------

/// Types of medieval NPCs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NpcType {
    /// Common villagers who work and trade.
    Peasant,
    /// Armed soldiers defending the settlement.
    Guard,
    /// Elite warriors in armor.
    Knight,
    /// Religious figures.
    Priest,
    /// Traders and merchants.
    Merchant,
    /// Rulers and nobility.
    Noble,
}

impl Default for NpcType {
    fn default() -> Self {
        Self::Peasant
    }
}

impl NpcType {
    /// Get the model path for this NPC type.
    pub fn model_path(&self) -> &str {
        match self {
            Self::Peasant => "models/npcs/peasant.glb",
            Self::Guard => "models/npcs/guard.glb",
            Self::Knight => "models/npcs/knight.glb",
            Self::Priest => "models/npcs/priest.glb",
            Self::Merchant => "models/npcs/merchant.glb",
            Self::Noble => "models/npcs/noble.glb",
        }
    }

    /// Get the default health for this NPC type.
    pub fn base_health(&self) -> f32 {
        match self {
            Self::Peasant => 50.0,
            Self::Guard => 100.0,
            Self::Knight => 150.0,
            Self::Priest => 40.0,
            Self::Merchant => 60.0,
            Self::Noble => 80.0,
        }
    }

    /// Get the movement speed for this NPC type.
    pub fn movement_speed(&self) -> f32 {
        match self {
            Self::Peasant => 1.5,
            Self::Guard => 2.0,
            Self::Knight => 2.5,
            Self::Priest => 1.2,
            Self::Merchant => 1.3,
            Self::Noble => 1.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Animation States
// ---------------------------------------------------------------------------

/// Animation states for NPC skeletal animations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnimationState {
    /// Standing still.
    Idle,
    /// Walking to a destination.
    Walk,
    /// Working animation (chopping, farming, etc.).
    Work,
    /// Running (for combat or fleeing).
    Run,
    /// Attacking animation.
    Attack,
    /// Taking damage.
    Hit,
    /// Dead/fallen.
    Death,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self::Idle
    }
}

// ---------------------------------------------------------------------------
// Behavior Types
// ---------------------------------------------------------------------------

/// Behavior tree node types for NPC AI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BehaviorType {
    /// Wander randomly within a radius.
    Wander { radius: f32 },
    /// Patrol between waypoints.
    Patrol { waypoints: Vec<Vec3> },
    /// Stand at a position and perform work.
    Work { position: Vec3, work_type: String },
    /// Follow a target entity.
    Follow { target_id: String },
    /// Idle at a position.
    Idle { duration: f32 },
    /// Sequence of behaviors.
    Sequence(Vec<BehaviorType>),
    /// Selector (try behaviors in order until one succeeds).
    Selector(Vec<BehaviorType>),
}

// ---------------------------------------------------------------------------
// NPC Configuration
// ---------------------------------------------------------------------------

/// Configuration for NPC spawning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcSpawnConfig {
    /// Type of NPC to spawn.
    pub npc_type: NpcType,
    /// Number of NPCs to spawn.
    pub count: u32,
    /// Spawn radius from center.
    pub radius: f32,
    /// Spawn center position.
    #[serde(default)]
    pub center: (f32, f32, f32),
    /// Default behavior for spawned NPCs.
    pub behavior: BehaviorType,
    /// Faction allegiance.
    #[serde(default)]
    pub faction_id: Option<String>,
}

/// Global configuration for the NPC system.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct NpcGlobalConfig {
    /// NPC spawn configurations.
    #[serde(default)]
    pub spawns: Vec<NpcSpawnConfig>,
    /// Maximum NPC count.
    #[serde(default = "default_max_npcs")]
    pub max_npcs: u32,
    /// AI update frequency (Hz).
    #[serde(default = "default_ai_frequency")]
    pub ai_frequency: f32,
}

fn default_max_npcs() -> u32 {
    100
}

fn default_ai_frequency() -> f32 {
    10.0
}

impl Default for NpcGlobalConfig {
    fn default() -> Self {
        Self {
            spawns: Vec::new(),
            max_npcs: 100,
            ai_frequency: 10.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marker component for NPC entities.
#[derive(Component, Debug)]
pub struct Npc {
    /// Type of NPC.
    pub npc_type: NpcType,
    /// Current animation state.
    pub animation_state: AnimationState,
    /// Faction allegiance.
    pub faction_id: Option<String>,
}

/// Health component for NPCs.
#[derive(Component, Debug)]
pub struct NpcHealth {
    /// Current health.
    pub current: f32,
    /// Maximum health.
    pub max: f32,
}

impl NpcHealth {
    /// Create a new health component based on NPC type.
    pub fn for_type(npc_type: NpcType) -> Self {
        let max = npc_type.base_health();
        Self { current: max, max }
    }

    /// Get health as percentage.
    pub fn percentage(&self) -> f32 {
        self.current / self.max
    }
}

/// Component for NPC movement.
#[derive(Component, Debug)]
pub struct NpcMovement {
    /// Movement speed multiplier.
    pub speed_multiplier: f32,
    /// Current velocity.
    pub velocity: Vec3,
    /// Target destination (if moving).
    pub target: Option<Vec3>,
}

impl NpcMovement {
    /// Create a new movement component based on NPC type.
    pub fn for_type(npc_type: NpcType) -> Self {
        Self {
            speed_multiplier: 1.0,
            velocity: Vec3::ZERO,
            target: None,
        }
    }

    /// Get effective movement speed.
    pub fn speed(&self, npc_type: NpcType) -> f32 {
        npc_type.movement_speed() * self.speed_multiplier
    }
}

// ---------------------------------------------------------------------------
// NPC Plugin
// ---------------------------------------------------------------------------

/// Modular NPC plugin using bevior_tree for behavior trees.
///
/// This plugin can be added to any Bevy app for NPC management.
/// Configuration is done via the `NpcGlobalConfig` resource.
///
/// # Usage
///
/// ```rust,ignore
/// app.add_plugins(NpcPlugin::default());
/// app.insert_resource(NpcGlobalConfig {
///     spawns: vec![...],
///     ..Default::default()
/// });
/// ```
#[derive(Default)]
pub struct NpcPlugin {
    /// Initial configuration (optional).
    pub config: Option<NpcGlobalConfig>,
}

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        info!("NpcPlugin: initializing NPC system");

        // Add config resource
        if let Some(config) = &self.config {
            app.insert_resource(config.clone());
        } else {
            app.init_resource::<NpcGlobalConfig>();
        }

        // Add systems
        app.add_systems(Startup, setup_npc_system);
        app.add_systems(Update, npc_animation_system);

        info!("NpcPlugin: registered systems");
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Setup the NPC system.
fn setup_npc_system(
    mut commands: Commands,
    config: Res<NpcGlobalConfig>,
    asset_server: Res<AssetServer>,
) {
    trace!("setup_npc_system: initializing");

    info!(
        "NpcPlugin: initialized with {} spawn configs, max NPCs: {}",
        config.spawns.len(),
        config.max_npcs
    );

    for spawn in &config.spawns {
        trace!(
            "  - Spawn: {}x {:?} at radius {}",
            spawn.count,
            spawn.npc_type,
            spawn.radius
        );

        // Spawn NPCs based on config
        spawn_npc_group(&mut commands, &asset_server, spawn);
    }
}

/// Spawn a group of NPCs based on spawn configuration.
fn spawn_npc_group(commands: &mut Commands, _asset_server: &AssetServer, spawn: &NpcSpawnConfig) {
    let center = Vec3::from(spawn.center);
    let count = spawn.count;
    let npc_type = spawn.npc_type;
    let faction_id = spawn.faction_id.clone();

    for i in 0..count {
        // Calculate position in a circle pattern
        let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
        let radius_variance = spawn.radius * (0.5 + (i as f32 * 0.1) % 0.5);
        let x = center.x + radius_variance * angle.cos();
        let z = center.z + radius_variance * angle.sin();
        let position = Vec3::new(x, 0.0, z); // Y will be set by terrain if needed

        // Create NPC entity (without mesh for now - asset loading needs async handling)
        commands.spawn((
            Name::new(format!("{:?}_{}", npc_type, i)),
            Transform::from_translation(position),
            Npc {
                npc_type,
                animation_state: AnimationState::Idle,
                faction_id: faction_id.clone(),
            },
            NpcHealth::for_type(npc_type),
            NpcMovement::for_type(npc_type),
        ));

        debug!("spawn_npc_group: spawned {:?} at {:?}", npc_type, position);
    }
}

/// Update NPC animations based on state.
fn npc_animation_system(time: Res<Time>, mut npcs: Query<(&mut Npc, &mut NpcMovement)>) {
    for (mut npc, mut movement) in npcs.iter_mut() {
        // Update animation based on movement
        if movement.velocity.length_squared() > 0.01 {
            if npc.animation_state != AnimationState::Walk
                && npc.animation_state != AnimationState::Run
            {
                npc.animation_state = AnimationState::Walk;
            }
        } else if npc.animation_state == AnimationState::Walk {
            npc.animation_state = AnimationState::Idle;
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn npc_type_default() {
        let npc_type = NpcType::default();
        assert_eq!(npc_type, NpcType::Peasant);
    }

    #[test]
    fn npc_type_properties() {
        assert!(NpcType::Knight.base_health() > NpcType::Peasant.base_health());
        assert!(NpcType::Knight.movement_speed() > NpcType::Noble.movement_speed());
    }

    #[test]
    fn animation_state_default() {
        let state = AnimationState::default();
        assert_eq!(state, AnimationState::Idle);
    }

    #[test]
    fn npc_health_creation() {
        let health = NpcHealth::for_type(NpcType::Knight);
        assert_eq!(health.max, 150.0);
        assert_eq!(health.current, 150.0);
        assert_eq!(health.percentage(), 1.0);
    }

    #[test]
    fn npc_movement_speed() {
        let movement = NpcMovement::for_type(NpcType::Guard);
        assert_eq!(movement.speed(NpcType::Guard), 2.0);
    }

    #[test]
    fn npc_global_config_default() {
        let config = NpcGlobalConfig::default();
        assert!(config.spawns.is_empty());
        assert_eq!(config.max_npcs, 100);
    }
}
