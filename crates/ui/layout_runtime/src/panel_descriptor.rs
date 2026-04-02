#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelDescriptor {
    pub panel_id: String,
    pub hosted_views: Vec<String>,
}

impl PanelDescriptor {
    pub fn new(panel_id: &str, hosted_views: Vec<String>) -> Self {
        Self {
            panel_id: panel_id.into(),
            hosted_views,
        }
    }
}
