use crate::linked_brokers::LinkedBrokerState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutRuntimeState {
    pub brokers: Vec<LinkedBrokerState>,
}

impl LayoutRuntimeState {
    pub fn with_brokers(brokers: Vec<LinkedBrokerState>) -> Self {
        let mut unique_brokers: Vec<LinkedBrokerState> = Vec::with_capacity(brokers.len());

        for broker in brokers {
            if let Some(existing) = unique_brokers
                .iter_mut()
                .find(|existing| existing.interaction_type == broker.interaction_type)
            {
                *existing = broker;
            } else {
                unique_brokers.push(broker);
            }
        }

        Self {
            brokers: unique_brokers,
        }
    }

    pub fn broker_for(&self, interaction_type: &str) -> Option<&LinkedBrokerState> {
        self.brokers
            .iter()
            .find(|broker| broker.interaction_type == interaction_type)
    }
}
