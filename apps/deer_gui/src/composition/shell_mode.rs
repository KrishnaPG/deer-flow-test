pub const SELECTION_BROKER: &str = "world_viewport";
pub const FOCUS_BROKER: &str = "inspector_panel";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShellBrokers {
    pub selection: &'static str,
    pub focus: &'static str,
}

pub const FIRST_PLAYABLE_BROKERS: ShellBrokers = ShellBrokers {
    selection: SELECTION_BROKER,
    focus: FOCUS_BROKER,
};
