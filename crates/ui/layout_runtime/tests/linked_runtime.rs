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

#[test]
fn linked_runtime_exposes_explicit_viewport_and_camera_broker_declarations() {
    let runtime = LayoutRuntimeState::with_brokers(vec![
        deer_ui_layout_runtime::linked_brokers::viewport_broker("world_viewport"),
        deer_ui_layout_runtime::linked_brokers::camera_broker("minimap_panel"),
    ])
    .unwrap();

    assert_eq!(
        runtime.broker_for("viewport"),
        Some(&LinkedBrokerState::new("viewport", "world_viewport"))
    );
    assert_eq!(
        runtime.broker_for("camera"),
        Some(&LinkedBrokerState::new("camera", "minimap_panel"))
    );
}

#[test]
fn linked_runtime_routes_viewport_and_camera_updates_only_through_declared_brokers() {
    let runtime = LayoutRuntimeState::with_brokers(vec![
        deer_ui_layout_runtime::linked_brokers::viewport_broker("world_viewport"),
        deer_ui_layout_runtime::linked_brokers::camera_broker("minimap_panel"),
    ])
    .unwrap();

    let viewport = runtime
        .propagate(LinkedInteractionUpdate::new(
            "viewport",
            "world",
            "minimap_panel",
        ))
        .unwrap();
    let camera = runtime
        .propagate(LinkedInteractionUpdate::new(
            "camera",
            "minimap",
            "world_viewport",
        ))
        .unwrap();

    assert_eq!(viewport.interaction_type, "viewport");
    assert_eq!(viewport.payload, "world");
    assert_eq!(viewport.broker_panel_id, "world_viewport");
    assert_eq!(viewport.origin_panel_id, "minimap_panel");

    assert_eq!(camera.interaction_type, "camera");
    assert_eq!(camera.payload, "minimap");
    assert_eq!(camera.broker_panel_id, "minimap_panel");
    assert_eq!(camera.origin_panel_id, "world_viewport");
}
