use crate::linked_brokers::LinkedBrokerState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutRuntimeState {
    pub brokers: Vec<LinkedBrokerState>,
}

impl LayoutRuntimeState {
    pub fn with_brokers(brokers: Vec<LinkedBrokerState>) -> Self {
        Self { brokers }
    }
}
