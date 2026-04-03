use deer_ui_layout_runtime::{minimap_panel_descriptor, world_panel_descriptor, PanelDescriptor};
use deer_ui_panel_shells::{PanelContract, PanelRole};

pub const CHAT_PANEL: &str = "chat_panel";
pub const ARTIFACT_PANEL: &str = "artifact_panel";
pub const INSPECTOR_PANEL: &str = "inspector_panel";
const THREAD_JOIN_KEY: &str = "thread_id";

pub fn panel_descriptors() -> Vec<PanelDescriptor> {
    let mut descriptors = vec![
        descriptor(
            CHAT_PANEL,
            "chat_thread_view",
            vec![PanelRole::Source, PanelRole::Broker],
        ),
        descriptor(ARTIFACT_PANEL, "artifact_shelf_view", vec![PanelRole::Sink]),
        descriptor(INSPECTOR_PANEL, "inspector_view", vec![PanelRole::Mirror]),
    ];
    descriptors.push(world_panel_descriptor().expect("world panel should remain valid"));
    descriptors.push(minimap_panel_descriptor().expect("minimap panel should remain valid"));
    descriptors
}

pub fn spatial_panel_ids() -> (String, String) {
    let world = world_panel_descriptor().expect("world panel should remain valid");
    let minimap = minimap_panel_descriptor().expect("minimap panel should remain valid");

    (world.panel_id().to_string(), minimap.panel_id().to_string())
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
