use super::layout_presets::FIRST_PLAYABLE_PRESET;
use super::panel_catalog::{
    ARTIFACT_PANEL, CHAT_PANEL, COMMAND_PANEL, EVENT_PANEL, INSPECTOR_PANEL, MINIMAP_PANEL,
    WORLD_VIEWPORT_PANEL,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirstPlayableShell {
    pub mode: String,
    pub panels: Vec<String>,
}

pub fn build_first_playable_shell() -> FirstPlayableShell {
    FirstPlayableShell {
        mode: FIRST_PLAYABLE_PRESET.into(),
        panels: vec![
            WORLD_VIEWPORT_PANEL.into(),
            MINIMAP_PANEL.into(),
            CHAT_PANEL.into(),
            ARTIFACT_PANEL.into(),
            INSPECTOR_PANEL.into(),
            EVENT_PANEL.into(),
            COMMAND_PANEL.into(),
        ],
    }
}
