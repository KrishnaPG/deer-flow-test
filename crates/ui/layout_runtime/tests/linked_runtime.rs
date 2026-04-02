use deer_ui_layout_runtime::{LayoutRuntimeState, LinkedBrokerState};

#[test]
fn linked_runtime_keeps_one_broker_per_interaction_type() {
    let runtime = LayoutRuntimeState::with_brokers(vec![
        LinkedBrokerState::new("selection", "chat_panel"),
        LinkedBrokerState::new("focus", "inspector_panel"),
    ]);

    assert_eq!(runtime.brokers.len(), 2);
    assert_eq!(runtime.brokers[0].interaction_type, "selection");
}
