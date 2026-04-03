use deer_runtime_world_projection::WorldProjection;
use serde::Serialize;

use crate::world_scene_vm::{build_world_scene_vm, SceneAnchor};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ActorCloudView {
    pub object_count: usize,
    pub anchors: Vec<SceneAnchor>,
}

pub fn render_actor_cloud(projection: &WorldProjection) -> ActorCloudView {
    let scene = build_world_scene_vm(projection);

    ActorCloudView {
        object_count: scene.anchors.len(),
        anchors: scene.anchors,
    }
}
