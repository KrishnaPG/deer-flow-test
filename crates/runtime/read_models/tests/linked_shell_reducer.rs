use deer_runtime_read_models::{
    reduce_linked_shell_state, LinkedShellAction, LinkedShellPanelRole, LinkedShellState,
};

#[test]
fn linked_shell_state_tracks_selection_pins_and_drill_down_by_canonical_ref() {
    let state = LinkedShellState::default();

    let state = reduce_linked_shell_state(
        state,
        LinkedShellAction::Select {
            source_record_id: "artifact_1".into(),
        },
    );
    let state = reduce_linked_shell_state(
        state,
        LinkedShellAction::Pin {
            source_record_id: "artifact_1".into(),
        },
    );
    let state = reduce_linked_shell_state(
        state,
        LinkedShellAction::OpenDrillDown {
            panel_target: "artifact_detail",
        },
    );

    assert_eq!(state.selected.as_deref(), Some("artifact_1"));
    assert_eq!(state.pinned, vec!["artifact_1".to_string()]);
    assert_eq!(state.drill_down_target.as_deref(), Some("artifact_detail"));
}

#[test]
fn linked_shell_state_preserves_panel_roles_for_restored_panels() {
    let state = LinkedShellState::default();

    let state = reduce_linked_shell_state(
        state,
        LinkedShellAction::PanelParticipationDeclared {
            panel_id: "event_rail".into(),
            roles: vec![LinkedShellPanelRole::Source, LinkedShellPanelRole::Sink],
        },
    );
    let state = reduce_linked_shell_state(
        state,
        LinkedShellAction::PanelParticipationDeclared {
            panel_id: "minimap".into(),
            roles: vec![LinkedShellPanelRole::Mirror, LinkedShellPanelRole::Sink],
        },
    );

    let state = reduce_linked_shell_state(
        state,
        LinkedShellAction::LayoutPanelsRestored {
            panel_ids: vec!["event_rail".into()],
        },
    );

    assert_eq!(
        state.panel_roles.get("event_rail"),
        Some(&vec![
            LinkedShellPanelRole::Source,
            LinkedShellPanelRole::Sink
        ])
    );
    assert_eq!(state.panel_roles.get("minimap"), None);
}
