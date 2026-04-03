use deer_runtime_read_models::{
    apply_policy_invalidation_to_linked_shell, reduce_linked_shell_state, reduce_policy_state,
    reduce_temporal_state, LinkedShellAction, LinkedShellState, PolicyAction, PolicyOverlayState,
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
    assert_eq!(next.stream_state.as_deref(), Some("degraded"));
    assert!(next.degraded);
    assert_eq!(next.world_overlay_freshness.status, "stale");
    assert_eq!(
        next.world_overlay_freshness.stale_reason,
        Some("late_event_inserted")
    );
    assert_eq!(
        next.world_overlay_freshness.source_event_id.as_deref(),
        Some("evt_9")
    );
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
        world_overlay_freshness: deer_runtime_read_models::WorldOverlayFreshness {
            status: "stale",
            stale_reason: Some("late_event_inserted"),
            source_event_id: Some("evt_9".into()),
        },
    };

    let next = reduce_temporal_state(state, TemporalAction::ReturnToLiveTail);

    assert_eq!(next.mode, "live_tail");
    assert_eq!(next.cursor_id, None);
    assert!(!next.is_stale);
    assert_eq!(next.stream_state.as_deref(), Some("live"));
    assert!(!next.degraded);
    assert_eq!(next.world_overlay_freshness.status, "fresh");
    assert_eq!(next.world_overlay_freshness.stale_reason, None);
    assert_eq!(next.world_overlay_freshness.source_event_id, None);
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
        world_overlay_freshness: deer_runtime_read_models::WorldOverlayFreshness {
            status: "stale",
            stale_reason: Some("late_event_inserted"),
            source_event_id: Some("evt_9".into()),
        },
    };

    let next = reduce_temporal_state(state, TemporalAction::LayoutRestored { layout_instance: 2 });

    assert_eq!(next.layout_instance, 2);
    assert_eq!(next.mode, "historical");
    assert_eq!(next.cursor_id.as_deref(), Some("checkpoint_7"));
    assert!(next.is_stale);
    assert_eq!(next.stream_state.as_deref(), Some("degraded"));
    assert!(next.degraded);
    assert_eq!(next.world_overlay_freshness.status, "stale");
    assert_eq!(
        next.world_overlay_freshness.stale_reason,
        Some("late_event_inserted")
    );
    assert_eq!(
        next.world_overlay_freshness.source_event_id.as_deref(),
        Some("evt_9")
    );
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

#[test]
fn policy_invalidation_clears_linked_shell_state_through_policy_api() {
    let linked = LinkedShellState {
        selected: Some("artifact_1".into()),
        focused: Some("artifact_1".into()),
        pinned: vec!["artifact_1".into(), "artifact_2".into()],
        ..Default::default()
    };

    let invalidated = apply_policy_invalidation_to_linked_shell(
        linked,
        &PolicyAction::RecordInvalidated {
            source_record_id: "artifact_1".into(),
            policy_epoch: 4,
            policy_reason: "access_revoked".into(),
            tombstone_visible: true,
        },
    );

    assert_eq!(invalidated.selected, None);
    assert_eq!(invalidated.focused, None);
    assert_eq!(invalidated.pinned, vec!["artifact_2".to_string()]);
}
