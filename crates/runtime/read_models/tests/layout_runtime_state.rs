use deer_runtime_read_models::{
    reduce_layout_runtime_state, LayoutRuntimeAction, LayoutRuntimeReadModel,
};

#[test]
fn layout_runtime_state_tracks_preset_restore_and_broker_epoch_changes() {
    let state = LayoutRuntimeReadModel::default();

    let state = reduce_layout_runtime_state(
        state,
        LayoutRuntimeAction::PresetLoaded {
            mode: "live_meeting".into(),
        },
    );
    let state = reduce_layout_runtime_state(
        state,
        LayoutRuntimeAction::BrokerEpochChanged {
            interaction_type: "selection".into(),
            epoch: 2,
        },
    );

    assert_eq!(state.active_mode.as_deref(), Some("live_meeting"));
    assert_eq!(state.broker_epochs.get("selection"), Some(&2));
}
