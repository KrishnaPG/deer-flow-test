use deer_ui_layout_runtime::{LayoutRuntimeError, LayoutRuntimeState, LinkedBrokerState};

#[test]
fn linked_runtime_rejects_duplicate_brokers_for_same_interaction_type() {
    let error = LayoutRuntimeState::with_brokers(vec![
        LinkedBrokerState::new("selection", "chat_panel"),
        LinkedBrokerState::new("selection", "artifact_panel"),
    ])
    .unwrap_err();

    assert_eq!(
        error,
        LayoutRuntimeError::DuplicateInteractionType {
            interaction_type: "selection".into(),
        }
    );
}

#[test]
fn linked_runtime_tracks_unique_brokers_by_interaction_type() {
    let runtime = LayoutRuntimeState::with_brokers(vec![
        LinkedBrokerState::new("selection", "chat_panel"),
        LinkedBrokerState::new("focus", "inspector_panel"),
    ])
    .unwrap();

    assert_eq!(runtime.brokers().len(), 2);
    assert_eq!(
        runtime.broker_for("selection"),
        Some(&LinkedBrokerState::new("selection", "chat_panel"))
    );
    assert_eq!(
        runtime.broker_for("focus"),
        Some(&LinkedBrokerState::new("focus", "inspector_panel"))
    );
}
