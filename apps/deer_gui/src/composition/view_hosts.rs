use super::panel_catalog::{
    ARTIFACT_PANEL, CHAT_PANEL, INSPECTOR_PANEL, MINIMAP_PANEL, WORLD_VIEWPORT_PANEL,
};

pub const CHAT_HOST: &str = "chat_thread_view";
pub const ARTIFACT_HOST: &str = "artifact_shelf_view";
pub const WORLD_HOST: &str = "world_scene_view";
pub const MINIMAP_HOST: &str = "minimap_view";
pub const INSPECTOR_HOST: &str = "inspector_view";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewHostBinding {
    pub panel: &'static str,
    pub host: &'static str,
}

pub const FIRST_PLAYABLE_VIEW_HOSTS: [ViewHostBinding; 5] = [
    ViewHostBinding {
        panel: WORLD_VIEWPORT_PANEL,
        host: WORLD_HOST,
    },
    ViewHostBinding {
        panel: MINIMAP_PANEL,
        host: MINIMAP_HOST,
    },
    ViewHostBinding {
        panel: CHAT_PANEL,
        host: CHAT_HOST,
    },
    ViewHostBinding {
        panel: ARTIFACT_PANEL,
        host: ARTIFACT_HOST,
    },
    ViewHostBinding {
        panel: INSPECTOR_PANEL,
        host: INSPECTOR_HOST,
    },
];
