#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PanelFrameState {
    pub title: String,
    pub status_banner: Option<String>,
}
