#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkedBrokerState {
    pub interaction_type: String,
    pub broker_panel_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkedInteractionUpdate {
    pub interaction_type: String,
    pub payload: String,
    pub origin_panel_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkedInteractionPropagation {
    pub interaction_type: String,
    pub payload: String,
    pub broker_panel_id: String,
    pub origin_panel_id: String,
}

impl LinkedBrokerState {
    pub fn new(interaction_type: &str, broker_panel_id: &str) -> Self {
        Self {
            interaction_type: interaction_type.into(),
            broker_panel_id: broker_panel_id.into(),
        }
    }
}

impl LinkedInteractionUpdate {
    pub fn new(interaction_type: &str, payload: &str, origin_panel_id: &str) -> Self {
        Self {
            interaction_type: interaction_type.into(),
            payload: payload.into(),
            origin_panel_id: origin_panel_id.into(),
        }
    }
}

impl LinkedInteractionPropagation {
    pub fn from_update(update: LinkedInteractionUpdate, broker_panel_id: &str) -> Self {
        Self {
            interaction_type: update.interaction_type,
            payload: update.payload,
            broker_panel_id: broker_panel_id.into(),
            origin_panel_id: update.origin_panel_id,
        }
    }
}
