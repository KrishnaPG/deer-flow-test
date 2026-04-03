use deer_runtime_world_projection::WorldProjection;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Position3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SceneAnchor {
    pub source_record_id: String,
    pub kind: &'static str,
    pub drill_down_target: &'static str,
    pub pickable: bool,
    pub position: Position3,
}

impl SceneAnchor {
    pub fn pickable(
        source_record_id: &str,
        kind: &'static str,
        drill_down_target: &'static str,
        x: f32,
        y: f32,
        z: f32,
    ) -> Self {
        Self {
            source_record_id: source_record_id.to_owned(),
            kind,
            drill_down_target,
            pickable: true,
            position: Position3::new(x, y, z),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WorldSceneVm {
    pub anchors: Vec<SceneAnchor>,
}

pub fn build_world_scene_vm(projection: &WorldProjection) -> WorldSceneVm {
    let anchors = projection
        .objects
        .iter()
        .enumerate()
        .map(|(index, object)| {
            SceneAnchor::pickable(
                &object.source_record_id,
                object.kind,
                object.drill_down_target,
                index as f32 * 2.5,
                0.0,
                0.0,
            )
        })
        .collect();

    WorldSceneVm { anchors }
}
