pub fn reopen_safe_target(panel_target: &str) -> &'static str {
    match panel_target {
        "task_detail" => "task_detail",
        "artifact_detail" => "artifact_detail",
        _ => "inspector",
    }
}
