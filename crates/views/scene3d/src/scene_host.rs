use deer_runtime_world_projection::WorldProjection;

use crate::{world_scene_vm::build_world_scene_vm, SpatialIndex};

#[derive(Debug, Clone, PartialEq)]
pub struct SceneHost {
    pub host_id: String,
    spatial_index: SpatialIndex,
}

impl SceneHost {
    pub fn from_projection(host_id: &str, projection: &WorldProjection) -> Self {
        let scene = build_world_scene_vm(projection);
        let mut spatial_index = SpatialIndex::new();

        for anchor in scene.anchors {
            spatial_index.insert_anchor(anchor);
        }

        Self {
            host_id: host_id.to_owned(),
            spatial_index,
        }
    }

    pub fn spatial_index(&self) -> &SpatialIndex {
        &self.spatial_index
    }
}
