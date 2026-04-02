use deer_ui_layout_runtime::{LayoutRuntimeState, LinkedBrokerState};

#[test]
fn linked_runtime_keeps_one_broker_per_interaction_type() {
    let runtime = LayoutRuntimeState::with_brokers(vec![
        LinkedBrokerState::new("selection", "chat_panel"),
        LinkedBrokerState::new("selection", "artifact_panel"),
        LinkedBrokerState::new("focus", "inspector_panel"),
    ]);

    assert_eq!(runtime.brokers.len(), 2);
    assert_eq!(
        runtime.broker_for("selection"),
        Some(&LinkedBrokerState::new("selection", "artifact_panel"))
    );
    assert_eq!(
        runtime.broker_for("focus"),
        Some(&LinkedBrokerState::new("focus", "inspector_panel"))
    );
}
