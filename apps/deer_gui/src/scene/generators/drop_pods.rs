//! Drop pods generator — vertical falling particle entities.

use bevy::asset::Assets;
use bevy::ecs::system::Commands;
use bevy::log::{debug, warn};
use bevy::math::Vec3;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{ChildOf, Component, Entity, Mesh, Mesh3d, MeshMaterial3d, Sphere, Transform};

use crate::scene::descriptor::GeneratorParams;

#[derive(Component, Debug, Clone)]
pub struct DropPod {
    pub t: f32,
    pub index: usize,
    pub speed: f32,
}

pub fn gen_drop_pods(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    root: Entity,
    params: &GeneratorParams,
) {
    let GeneratorParams::DropPods {
        count,
        speed,
        emissive,
    } = params
    else {
        warn!("gen_drop_pods: expected DropPods params");
        return;
    };

    let mesh = meshes.add(Mesh::from(Sphere::new(0.4)));
    let material = materials.add(StandardMaterial {
        emissive: bevy::color::LinearRgba::new(emissive[0], emissive[1], emissive[2], emissive[3]),
        ..Default::default()
    });

    for i in 0..*count {
        let t = i as f32 / *count as f32;
        let pos = pod_position(t, i);
        commands.spawn((
            DropPod {
                t,
                index: i,
                speed: *speed,
            },
            ChildOf(root),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(pos),
        ));
    }
    debug!("gen_drop_pods: count={count} speed={speed}");
}

pub fn pod_position(t: f32, index: usize) -> Vec3 {
    let phase = index as f32 * 1.618;
    let x = (phase * 0.7).cos() * 100.0;
    let z = (phase * 0.7).sin() * 100.0;
    let y = (1.0 - t) * 800.0 - 400.0; // falling downward
    Vec3::new(x, y, z)
}
