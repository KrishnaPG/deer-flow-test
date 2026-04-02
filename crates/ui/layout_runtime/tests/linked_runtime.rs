use deer_ui_layout_runtime::{
    LayoutRuntimeError, LayoutRuntimeState, LinkedBrokerState, LinkedInteractionUpdate,
};

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

#[test]
fn linked_runtime_routes_updates_through_declared_broker() {
    let runtime = LayoutRuntimeState::with_brokers(vec![
        LinkedBrokerState::new("selection", "chat_panel"),
        LinkedBrokerState::new("focus", "inspector_panel"),
    ])
    .unwrap();

    let propagated = runtime
        .propagate(LinkedInteractionUpdate::new(
            "selection",
            "artifact-42",
            "artifact_panel",
        ))
        .unwrap();

    assert_eq!(propagated.interaction_type, "selection");
    assert_eq!(propagated.payload, "artifact-42");
    assert_eq!(propagated.broker_panel_id, "chat_panel");
    assert_eq!(propagated.origin_panel_id, "artifact_panel");
}

#[test]
fn linked_runtime_rejects_updates_for_unknown_interaction_types() {
    let runtime =
        LayoutRuntimeState::with_brokers(vec![LinkedBrokerState::new("selection", "chat_panel")])
            .unwrap();

    let error = runtime
        .propagate(LinkedInteractionUpdate::new(
            "focus",
            "artifact-42",
            "artifact_panel",
        ))
        .unwrap_err();

    assert_eq!(
        error,
        LayoutRuntimeError::UnknownInteractionType {
            interaction_type: "focus".into(),
        }
    );
}
