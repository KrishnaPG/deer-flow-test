//! glTF subscene generator — loads a glTF/GLB asset and spawns it as a child of the scene root.

use bevy::asset::Assets;
use bevy::ecs::system::Commands;
use bevy::ecs::world::World;
use bevy::log::{debug, trace, warn};
use bevy::math::Vec3;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{
    AssetServer, ChildOf, Entity, Handle, InheritedVisibility, Mesh, Scene, SceneRoot, Transform,
    Visibility,
};

use crate::scene::descriptor::GeneratorParams;

/// Spawn a glTF/GLB subscene as a child of the scene root.
///
/// Uses `commands.queue` to defer AssetServer access until command flush.
pub fn gen_gltf_subscene(
    commands: &mut Commands,
    _meshes: &mut Assets<Mesh>,
    _materials: &mut Assets<StandardMaterial>,
    root: Entity,
    params: &GeneratorParams,
) {
    let GeneratorParams::GltfSubscene {
        path,
        transform,
        scale,
    } = params
    else {
        warn!("gen_gltf_subscene: expected GltfSubscene params, got {params:?}");
        return;
    };

    trace!("gen_gltf_subscene: path='{path}' transform={transform:?} scale={scale:?}");

    let path = path.clone();
    let transform_arr = *transform;
    let scale_val = *scale;

    commands.queue(move |world: &mut World| {
        let asset_server = world.resource::<AssetServer>();
        let scene_handle: Handle<Scene> = asset_server.load(format!("{path}#Scene0"));
        let t = build_transform(transform_arr, scale_val);
        world.commands().spawn((
            ChildOf(root),
            SceneRoot(scene_handle),
            t,
            Visibility::default(),
            InheritedVisibility::default(),
        ));
        debug!("gen_gltf_subscene: queued glTF load path='{path}' root={root:?}");
    });
}

/// Build a Transform from optional position and scale.
fn build_transform(position: Option<[f32; 3]>, scale: Option<f32>) -> Transform {
    let mut t = Transform::IDENTITY;
    if let Some(pos) = position {
        t = Transform::from_translation(Vec3::new(pos[0], pos[1], pos[2]));
    }
    if let Some(s) = scale {
        t = t.with_scale(Vec3::splat(s));
    }
    t
}
