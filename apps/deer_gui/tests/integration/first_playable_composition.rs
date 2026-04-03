use deer_gui::composition::build_first_playable_shell;
use deer_gui::composition::layout_presets::FIRST_PLAYABLE_PRESET;
use deer_gui::composition::panel_catalog::{
    ARTIFACT_PANEL, CHAT_PANEL, INSPECTOR_PANEL, MINIMAP_PANEL, WORLD_VIEWPORT_PANEL,
};
use deer_gui::composition::shell_mode::{ShellBrokers, FOCUS_BROKER, SELECTION_BROKER};
use deer_gui::composition::view_hosts::{
    ARTIFACT_HOST, CHAT_HOST, INSPECTOR_HOST, MINIMAP_HOST, WORLD_HOST,
};

#[test]
fn builds_first_playable_shell_from_proven_modules_only() {
    let shell = build_first_playable_shell();

    assert_eq!(shell.mode, FIRST_PLAYABLE_PRESET);
    assert_eq!(
        shell.panels,
        [
            WORLD_VIEWPORT_PANEL,
            MINIMAP_PANEL,
            CHAT_PANEL,
            ARTIFACT_PANEL,
            INSPECTOR_PANEL,
        ]
    );
    assert_eq!(INSPECTOR_HOST, "inspector_view");
    assert_eq!(WORLD_HOST, "world_scene_view");
    assert_eq!(MINIMAP_HOST, "minimap_view");
    assert_eq!(
        shell.brokers,
        ShellBrokers {
            selection: SELECTION_BROKER,
            focus: FOCUS_BROKER,
        }
    );
    assert_eq!(
        shell
            .view_hosts
            .iter()
            .map(|binding| (binding.panel, binding.host))
            .collect::<Vec<_>>(),
        vec![
            (WORLD_VIEWPORT_PANEL, WORLD_HOST),
            (MINIMAP_PANEL, MINIMAP_HOST),
            (CHAT_PANEL, CHAT_HOST),
            (ARTIFACT_PANEL, ARTIFACT_HOST),
            (INSPECTOR_PANEL, INSPECTOR_HOST),
        ]
    );
}
