use deer_runtime_read_models::{
    reduce_linked_shell_state, reduce_policy_state, reduce_temporal_state, LinkedShellAction,
    LinkedShellState, PolicyAction, PolicyOverlayState, TemporalAction, TemporalState,
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
    assert_eq!(next.stream_state.as_deref(), Some("degraded"));
    assert!(next.degraded);
}

#[test]
fn temporal_state_returns_to_live_tail_and_clears_degradation() {
    let state = TemporalState {
        layout_instance: 0,
        mode: "historical",
        cursor_id: Some("checkpoint_7".into()),
        is_stale: true,
        stream_state: Some("degraded".into()),
        degraded: true,
    };

    let next = reduce_temporal_state(state, TemporalAction::ReturnToLiveTail);

    assert_eq!(next.mode, "live_tail");
    assert_eq!(next.cursor_id, None);
    assert!(!next.is_stale);
    assert_eq!(next.stream_state.as_deref(), Some("live"));
    assert!(!next.degraded);
}

#[test]
fn temporal_state_preserves_staleness_metadata_across_layout_restore() {
    let state = TemporalState {
        mode: "historical",
        cursor_id: Some("checkpoint_7".into()),
        is_stale: true,
        stream_state: Some("degraded".into()),
        degraded: true,
        layout_instance: 1,
    };

    let next = reduce_temporal_state(state, TemporalAction::LayoutRestored { layout_instance: 2 });

    assert_eq!(next.layout_instance, 2);
    assert_eq!(next.mode, "historical");
    assert_eq!(next.cursor_id.as_deref(), Some("checkpoint_7"));
    assert!(next.is_stale);
    assert_eq!(next.stream_state.as_deref(), Some("degraded"));
    assert!(next.degraded);
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

#[test]
fn policy_overlay_tracks_explicit_exclusion_and_tombstone_invalidation() {
    let state = PolicyOverlayState::default();

    let next = reduce_policy_state(
        state,
        PolicyAction::RecordInvalidated {
            source_record_id: "artifact_1".into(),
            policy_epoch: 4,
            policy_reason: "access_revoked".into(),
            tombstone_visible: true,
        },
    );

    assert_eq!(next.policy_epoch, 4);
    assert_eq!(next.policy_reason.as_deref(), Some("access_revoked"));
    assert_eq!(next.excluded_record_ids, vec!["artifact_1".to_string()]);
    assert_eq!(next.tombstoned_record_ids, vec!["artifact_1".to_string()]);
}
