pub const ARTIFACT_DETAIL_TARGET: &str = "artifact_detail";
pub const INSPECTOR_TARGET: &str = "inspector";

pub fn route_to_detail(selection_id: &str) -> &'static str {
    match selection_id {
        "artifact_1" => ARTIFACT_DETAIL_TARGET,
        _ => INSPECTOR_TARGET,
    }
}
