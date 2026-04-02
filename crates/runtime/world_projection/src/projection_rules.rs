use deer_foundation_domain::AnyRecord;

use crate::world_object::{WorldObject, WorldProjection};

pub fn project_world_objects(records: &[AnyRecord]) -> WorldProjection {
    let objects = records
        .iter()
        .filter_map(|record| match record {
            AnyRecord::Task(task) => Some(
                WorldObject::task_beacon(task.record_id().as_str(), "task_detail")
                    .with_supersession(
                        task.header
                            .lineage
                            .supersedes
                            .as_ref()
                            .map(|record_ref| record_ref.record_id.to_string()),
                    ),
            ),
            AnyRecord::Artifact(artifact) => Some(
                WorldObject::artifact_unlock(artifact.record_id().as_str(), "artifact_detail")
                    .with_supersession(
                        artifact
                            .header
                            .lineage
                            .supersedes
                            .as_ref()
                            .map(|record_ref| record_ref.record_id.to_string()),
                    ),
            ),
            _ => None,
        })
        .collect();

    WorldProjection { objects }
}
