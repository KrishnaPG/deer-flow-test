use crate::layout_model::LayoutSnapshot;

pub fn serialize_layout(snapshot: &LayoutSnapshot) -> Result<String, serde_json::Error> {
    serde_json::to_string(snapshot)
}

pub fn deserialize_layout(encoded: &str) -> Result<LayoutSnapshot, serde_json::Error> {
    serde_json::from_str(encoded)
}
