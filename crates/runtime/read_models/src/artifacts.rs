use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct ArtifactPanelState {
    pub selected_artifact_id: Option<String>,
    pub preview_request_state: Option<String>,
}
