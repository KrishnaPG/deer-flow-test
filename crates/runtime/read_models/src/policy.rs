use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct PolicyOverlayState {
    pub excluded_record_ids: Vec<String>,
}
