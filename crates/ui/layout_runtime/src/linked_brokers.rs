#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkedBrokerState {
    pub interaction_type: String,
    pub broker_panel_id: String,
}

impl LinkedBrokerState {
    pub fn new(interaction_type: &str, broker_panel_id: &str) -> Self {
        Self {
            interaction_type: interaction_type.into(),
            broker_panel_id: broker_panel_id.into(),
        }
    }
}
