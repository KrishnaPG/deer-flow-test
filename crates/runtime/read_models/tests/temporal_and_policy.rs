use deer_runtime_read_models::{
    reduce_linked_shell_state, reduce_temporal_state, LinkedShellAction, LinkedShellState,
    TemporalAction, TemporalState,
};

#[test]
fn temporal_state_keeps_historical_cursor_stable_on_late_event() {
    let state = TemporalState::historical("checkpoint_7");
    let next = reduce_temporal_state(
        state,
        TemporalAction::LateEventInserted {
            event_id: "evt_9".into(),
        },
    );

    assert_eq!(next.cursor_id.as_deref(), Some("checkpoint_7"));
    assert!(next.is_stale);
}

#[test]
fn policy_exclusion_clears_ghost_selection_and_pins() {
    let state = LinkedShellState {
        selected: Some("artifact_1".into()),
        pinned: vec!["artifact_1".into()],
        ..Default::default()
    };

    let cleared = reduce_linked_shell_state(
        state,
        LinkedShellAction::Exclude {
            source_record_id: "artifact_1".into(),
        },
    );

    assert_eq!(cleared.selected, None);
    assert!(cleared.pinned.is_empty());
}
