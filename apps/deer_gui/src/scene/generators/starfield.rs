//! Starfield generator — spawns stars on a Fibonacci sphere.

use bevy::asset::Assets;
use bevy::ecs::system::Commands;
use bevy::log::{debug, warn};
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Entity, Mesh};

use crate::scene::descriptor::GeneratorParams;
use crate::scene::primitives;

/// Spawn a starfield using shared primitives.
pub fn gen_starfield(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    root: Entity,
    params: &GeneratorParams,
) {
    let GeneratorParams::Starfield {
        count,
        radius,
        emissive,
    } = params
    else {
        warn!("gen_starfield: expected Starfield params, got {:?}", params);
        return;
    };

    let emissive_color =
        bevy::color::LinearRgba::new(emissive[0], emissive[1], emissive[2], emissive[3]);

    primitives::spawn_starfield(
        commands,
        meshes,
        materials,
        root,
        emissive_color,
        *count,
        *radius,
    );
    debug!("gen_starfield: count={count} radius={radius}");
}
