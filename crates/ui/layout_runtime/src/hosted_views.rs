#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostedViewRegistration {
    view_id: &'static str,
}

impl HostedViewRegistration {
    const fn new(view_id: &'static str) -> Self {
        Self { view_id }
    }

    pub fn view_id(self) -> &'static str {
        self.view_id
    }
}

pub const CHAT_THREAD: HostedViewRegistration = HostedViewRegistration::new("chat_thread_view");
pub const ARTIFACT_SHELF: HostedViewRegistration =
    HostedViewRegistration::new("artifact_shelf_view");
pub const INSPECTOR: HostedViewRegistration = HostedViewRegistration::new("inspector_view");
pub const WORLD_SCENE: HostedViewRegistration = HostedViewRegistration::new("world_scene_view");
pub const MINIMAP: HostedViewRegistration = HostedViewRegistration::new("minimap_view");

pub fn hosted_view_registration(view_id: &str) -> Option<HostedViewRegistration> {
    match view_id {
        "chat_thread_view" => Some(CHAT_THREAD),
        "artifact_shelf_view" => Some(ARTIFACT_SHELF),
        "inspector_view" => Some(INSPECTOR),
        "world_scene_view" => Some(WORLD_SCENE),
        "minimap_view" => Some(MINIMAP),
        _ => None,
    }
}
