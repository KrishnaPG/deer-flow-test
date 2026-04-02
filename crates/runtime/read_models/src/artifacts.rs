use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub enum ArtifactRequestState {
    #[default]
    Idle,
    Requested {
        artifact_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct ArtifactPanelState {
    pub selected_artifact_id: Option<String>,
    pub preview_request_state: ArtifactRequestState,
    pub open_request_state: ArtifactRequestState,
}
