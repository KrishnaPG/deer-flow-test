use deer_ui_panel_shells::{PanelParticipation, PanelRole};

fn main() {
    let _ = PanelParticipation {
        required_hosted_views: vec!["artifact_shelf_view".to_string()],
        roles: vec![PanelRole::Source],
        join_keys: vec!["artifact_id".to_string()],
    };
}
