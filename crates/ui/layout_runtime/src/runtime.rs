use std::collections::BTreeSet;

use thiserror::Error;

use crate::linked_brokers::LinkedBrokerState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutRuntimeState {
    brokers: Vec<LinkedBrokerState>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LayoutRuntimeError {
    #[error("duplicate broker interaction type '{interaction_type}'")]
    DuplicateInteractionType { interaction_type: String },
}

impl LayoutRuntimeState {
    pub fn with_brokers(brokers: Vec<LinkedBrokerState>) -> Result<Self, LayoutRuntimeError> {
        let mut interaction_types = BTreeSet::new();

        for broker in &brokers {
            if !interaction_types.insert(broker.interaction_type.clone()) {
                return Err(LayoutRuntimeError::DuplicateInteractionType {
                    interaction_type: broker.interaction_type.clone(),
                });
            }
        }

        Ok(Self { brokers })
    }

    pub fn brokers(&self) -> &[LinkedBrokerState] {
        &self.brokers
    }

    pub fn broker_for(&self, interaction_type: &str) -> Option<&LinkedBrokerState> {
        self.brokers
            .iter()
            .find(|broker| broker.interaction_type == interaction_type)
    }
}
