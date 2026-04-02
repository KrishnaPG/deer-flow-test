use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorldObject {
    pub kind: &'static str,
    pub source_record_id: String,
    pub drill_down_target: &'static str,
    #[serde(skip_serializing)]
    pub level: &'static str,
    #[serde(skip_serializing)]
    pub plane: &'static str,
}

impl WorldObject {
    pub fn task_beacon(source_record_id: &str, drill_down_target: &'static str) -> Self {
        Self {
            kind: "WorldTaskBeacon",
            source_record_id: source_record_id.to_owned(),
            drill_down_target,
            level: "L2",
            plane: "AsIs",
        }
    }

    pub fn artifact_unlock(source_record_id: &str, drill_down_target: &'static str) -> Self {
        Self {
            kind: "WorldArtifactUnlock",
            source_record_id: source_record_id.to_owned(),
            drill_down_target,
            level: "L2",
            plane: "AsIs",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorldProjection {
    pub objects: Vec<WorldObject>,
}
