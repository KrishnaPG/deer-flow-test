use deer_foundation_domain::AnyRecord;

use crate::world_object::{WorldObject, WorldProjection};

pub fn project_world_objects(records: &[AnyRecord]) -> WorldProjection {
    let objects = records
        .iter()
        .filter_map(|record| match record {
            AnyRecord::Task(task) => Some(WorldObject::task_beacon(
                task.record_id().as_str(),
                "task_detail",
            )),
            AnyRecord::Artifact(artifact) => Some(WorldObject::artifact_unlock(
                artifact.record_id().as_str(),
                "artifact_detail",
            )),
            _ => None,
        })
        .collect();

    WorldProjection { objects }
}
