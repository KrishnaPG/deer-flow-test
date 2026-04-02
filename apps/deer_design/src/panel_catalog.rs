use deer_ui_layout_runtime::PanelDescriptor;
use deer_ui_panel_shells::{PanelContract, PanelRole};

pub const CHAT_PANEL: &str = "chat_panel";
pub const ARTIFACT_PANEL: &str = "artifact_panel";
pub const INSPECTOR_PANEL: &str = "inspector_panel";
const THREAD_JOIN_KEY: &str = "thread_id";

pub fn panel_descriptors() -> Vec<PanelDescriptor> {
    vec![
        descriptor(
            CHAT_PANEL,
            "chat_thread_view",
            vec![PanelRole::Source, PanelRole::Broker],
        ),
        descriptor(ARTIFACT_PANEL, "artifact_shelf_view", vec![PanelRole::Sink]),
        descriptor(INSPECTOR_PANEL, "inspector_view", vec![PanelRole::Mirror]),
    ]
}

fn descriptor(panel_id: &str, hosted_view: &str, roles: Vec<PanelRole>) -> PanelDescriptor {
    PanelDescriptor::new(PanelContract {
        panel_id: panel_id.into(),
        required_hosted_views: vec![hosted_view.into()],
        roles,
        join_keys: vec![THREAD_JOIN_KEY.into()],
    })
    .expect("panel contract should remain valid")
}
