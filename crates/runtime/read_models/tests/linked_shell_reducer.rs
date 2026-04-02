use deer_runtime_read_models::{reduce_linked_shell_state, LinkedShellAction, LinkedShellState};

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
