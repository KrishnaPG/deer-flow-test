use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VmBacklink {
    pub source_record_id: String,
    pub level: String,
    pub plane: String,
    pub panel_target: &'static str,
}
