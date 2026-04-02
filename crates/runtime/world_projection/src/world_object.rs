use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorldObject {
    pub kind: &'static str,
    pub source_record_id: String,
    pub drill_down_target: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supersedes_record_id: Option<String>,
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
            supersedes_record_id: None,
            level: "L2",
            plane: "AsIs",
        }
    }

    pub fn artifact_unlock(source_record_id: &str, drill_down_target: &'static str) -> Self {
        Self {
            kind: "WorldArtifactUnlock",
            source_record_id: source_record_id.to_owned(),
            drill_down_target,
            supersedes_record_id: None,
            level: "L2",
            plane: "AsIs",
        }
    }

    pub fn with_supersession(mut self, supersedes_record_id: Option<String>) -> Self {
        self.supersedes_record_id = supersedes_record_id;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorldProjection {
    pub objects: Vec<WorldObject>,
}
