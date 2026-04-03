use bevy::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonicalRecordFamily {
    Agent,
    Mission,
    Artifact,
    Session,
    ReplayCheckpoint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalEntityRef {
    pub family: CanonicalRecordFamily,
    pub canonical_id: String,
    pub correlation_id: Option<String>,
    pub lineage_id: Option<String>,
}

#[derive(Debug, Default)]
pub struct SelectionState {
    pub primary: Option<CanonicalEntityRef>,
    pub ordered: Vec<CanonicalEntityRef>,
}

#[derive(Debug, Default)]
pub struct FocusState {
    pub target: Option<CanonicalEntityRef>,
}

#[derive(Debug, Default)]
pub struct FilterState {
    pub global: Vec<String>,
    pub panel: Vec<String>,
}

#[derive(Debug, Default)]
pub struct PinState {
    pub items: Vec<CanonicalEntityRef>,
}

#[derive(Debug, Default)]
pub struct CompareState {
    pub items: Vec<CanonicalEntityRef>,
}

#[derive(Debug, Default)]
pub struct IntentPrefillState {
    pub target: Option<CanonicalEntityRef>,
}

#[derive(Resource, Debug, Default)]
pub struct ShellState {
    pub selection: SelectionState,
    pub focus: FocusState,
    pub filters: FilterState,
    pub pins: PinState,
    pub compare: CompareState,
    pub intent_prefill: IntentPrefillState,
}
