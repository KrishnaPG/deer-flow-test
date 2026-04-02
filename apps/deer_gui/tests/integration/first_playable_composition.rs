use deer_gui::composition::build_first_playable_shell;

#[test]
fn builds_first_playable_shell_from_proven_modules_only() {
    let shell = build_first_playable_shell();

    assert_eq!(shell.mode, "battle_command_thin");
    assert!(shell.panels.contains(&"world_viewport".to_string()));
    assert!(shell.panels.contains(&"chat_panel".to_string()));
    assert!(shell.panels.contains(&"inspector_panel".to_string()));
}
