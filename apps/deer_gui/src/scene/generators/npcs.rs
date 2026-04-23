//! NPC generator — spawns non-player characters for a scene.
//!
//! Creates NPC entities with behavior configuration.

use bevy::ecs::system::Commands;
use bevy::log::{debug, info, warn};
use bevy::math::Vec3;
use bevy::prelude::{Entity, InheritedVisibility, Name, Transform, Visibility};
use rand::thread_rng;

use crate::scene::descriptor::GeneratorParams;
use crate::world::npc::{
    AnimationState, Npc, NpcGlobalConfig, NpcHealth, NpcMovement,
};

/// Generate NPCs from scene descriptor parameters.
pub fn gen_npcs(
    commands: &mut Commands,
    _meshes: &mut bevy::asset::Assets<bevy::prelude::Mesh>,
    _materials: &mut bevy::asset::Assets<bevy::pbr::StandardMaterial>,
    root: Entity,
    params: &GeneratorParams,
) {
    let GeneratorParams::Npcs { spawns } = params else {
        warn!("gen_npcs: expected Npcs params");
        return;
    };

    let total_count: u32 = spawns.iter().map(|s| s.count).sum();
    info!(
        "gen_npcs: spawning {} NPCs in {} groups",
        total_count,
        spawns.len()
    );

    commands.insert_resource(NpcGlobalConfig {
        spawns: spawns.clone(),
        max_npcs: total_count.max(100),
        ai_frequency: 10.0,
    });

    let _rng = thread_rng();

    for spawn in spawns {
        let center = Vec3::from(spawn.center);
        let count = spawn.count;
        let npc_type = spawn.npc_type;
        let faction_id = spawn.faction_id.clone();

        for i in 0..count {
            let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
            let radius_variance = spawn.radius * (0.5 + (i as f32 * 0.1) % 0.5);
            let x = center.x + radius_variance * angle.cos();
            let z = center.z + radius_variance * angle.sin();
            let position = Vec3::new(x, center.y, z);

            let entity = commands
                .spawn((
                    Name::new(format!("{:?}_{}", npc_type, i)),
                    Transform::from_translation(position),
                    Npc {
                        npc_type,
                        animation_state: AnimationState::Idle,
                        faction_id: faction_id.clone(),
                    },
                    NpcHealth::for_type(npc_type),
                    NpcMovement::for_type(npc_type),
                    Visibility::default(),
                    InheritedVisibility::default(),
                ))
                .id();

            commands.entity(root).add_child(entity);
            debug!("gen_npcs: spawned {:?} at {:?}", npc_type, position);
        }
    }
}
