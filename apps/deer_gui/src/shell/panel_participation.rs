use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanelId {
    MissionRailPanel,
    InspectorPanel,
    CommandDeckPanel,
    CenterCanvasPanel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionKind {
    Selection,
    Focus,
    Filter,
    Pin,
    Compare,
    IntentPrefill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanelRole {
    Source,
    Sink,
    Broker,
}

#[derive(Debug, Clone)]
pub struct PanelParticipation {
    pub roles: Vec<PanelRole>,
    pub interactions: Vec<InteractionKind>,
}

#[derive(Debug, Default)]
pub struct PanelParticipationRegistry {
    panels: HashMap<PanelId, PanelParticipation>,
}

impl PanelParticipationRegistry {
    pub fn get(&self, panel_id: &PanelId) -> Option<&PanelParticipation> {
        self.panels.get(panel_id)
    }
}
