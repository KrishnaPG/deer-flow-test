use bevy::prelude::*;

/// Visibility tier for the battle command shell overlay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellVisibilityTier {
    Tier1,
    Tier2,
    Tier3,
}

/// Collapse state for rails (event rail, fleet rail, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RailCollapseState {
    Expanded,
    Compact,
}

/// Which section of the bottom deck is currently active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BottomDeckSection {
    SelectionSummary,
    ActionDeck,
    QueueStatus,
}

/// Central state resource for the battle command HUD.
#[derive(Debug, Resource)]
pub struct BattleCommandHudState {
    /// Current shell visibility tier.
    pub visibility_tier: ShellVisibilityTier,
    /// Event rail collapse state.
    pub event_rail: RailCollapseState,
    /// Fleet rail collapse state.
    pub fleet_rail: RailCollapseState,
    /// Active bottom deck section.
    pub active_bottom_section: BottomDeckSection,
    /// Number of event badges to display.
    pub event_badge_count: u32,
    /// Number of fleet badges to display.
    pub fleet_badge_count: u32,
    /// Whether the overlay blocks world navigation.
    pub overlay_blocks_world_navigation: bool,
}

impl Default for BattleCommandHudState {
    fn default() -> Self {
        Self {
            visibility_tier: ShellVisibilityTier::Tier1,
            event_rail: RailCollapseState::Expanded,
            fleet_rail: RailCollapseState::Expanded,
            active_bottom_section: BottomDeckSection::SelectionSummary,
            event_badge_count: 0,
            fleet_badge_count: 0,
            overlay_blocks_world_navigation: false,
        }
    }
}
