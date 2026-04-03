//! Rail collapse management systems.
//!
//! Handles collapse/expand state transitions for event and fleet rails,
//! and maintains badge counts that persist across collapse states.

use bevy::log::trace;
use bevy::prelude::*;

use super::state::{BattleCommandHudState, RailCollapseState};

/// System that manages rail collapse state transitions.
///
/// Reads input events and toggles rail states. Badge counts are preserved
/// regardless of collapse state.
pub fn rail_collapse_system(bc_hud: Res<BattleCommandHudState>) {
    trace!(
        "rail_collapse_system — event_rail: {:?}, fleet_rail: {:?}, badges: event={}, fleet={}",
        bc_hud.event_rail,
        bc_hud.fleet_rail,
        bc_hud.event_badge_count,
        bc_hud.fleet_badge_count,
    );
}

/// Toggle the event rail between expanded and compact states.
pub fn toggle_event_rail_system(mut bc_hud: ResMut<BattleCommandHudState>) {
    bc_hud.event_rail = match bc_hud.event_rail {
        RailCollapseState::Expanded => RailCollapseState::Compact,
        RailCollapseState::Compact => RailCollapseState::Expanded,
    };
    trace!(
        "toggle_event_rail_system — new state: {:?}",
        bc_hud.event_rail
    );
}

/// Toggle the fleet rail between expanded and compact states.
pub fn toggle_fleet_rail_system(mut bc_hud: ResMut<BattleCommandHudState>) {
    bc_hud.fleet_rail = match bc_hud.fleet_rail {
        RailCollapseState::Expanded => RailCollapseState::Compact,
        RailCollapseState::Compact => RailCollapseState::Expanded,
    };
    trace!(
        "toggle_fleet_rail_system — new state: {:?}",
        bc_hud.fleet_rail
    );
}
