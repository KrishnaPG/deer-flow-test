//! HUD state maintenance systems — mutations that run before rendering.
//!
//! These systems update [`HudState`] fields (aging, pruning, dispatching)
//! and are scheduled in `Update`, before the `EguiPrimaryContextPass`
//! render systems touch the same resource as read-only.

use bevy::log::{debug, trace};
use bevy::prelude::*;

use crate::constants::timing::{EVENT_TICKER_FADE_SECS, EVENT_TICKER_MAX_ENTRIES};

use super::HudState;

// ---------------------------------------------------------------------------
// Event ticker maintenance
// ---------------------------------------------------------------------------

/// Ages event log entries, prunes fully-faded ones, and caps list length.
///
/// Runs during `Update`, before the render pass reads the log.
pub fn event_ticker_maintenance_system(mut hud: ResMut<HudState>, time: Res<Time>) {
    let dt = time.delta_secs();

    // Age all entries
    for entry in &mut hud.event_log {
        entry.age_secs += dt;
    }

    // Prune entries that have fully faded (2x the fade window)
    let before = hud.event_log.len();
    hud.event_log
        .retain(|e| e.age_secs < EVENT_TICKER_FADE_SECS * 2.0);
    let pruned = before - hud.event_log.len();

    // Cap the list to prevent unbounded growth
    let mut drained = 0;
    if hud.event_log.len() > EVENT_TICKER_MAX_ENTRIES {
        let drain_count = hud.event_log.len() - EVENT_TICKER_MAX_ENTRIES;
        hud.event_log.drain(0..drain_count);
        drained = drain_count;
    }

    if pruned > 0 || drained > 0 {
        trace!(
            "event_ticker_maintenance — pruned={}, drained={}, remaining={}",
            pruned,
            drained,
            hud.event_log.len()
        );
    }
}

// ---------------------------------------------------------------------------
// Command dispatch
// ---------------------------------------------------------------------------

/// Processes pending command intents from the console.
///
/// Consumes [`HudState::pending_command`] and logs it.
/// Future: dispatches to `OrchestratorClient` or a command event bus.
pub fn command_dispatch_system(mut hud: ResMut<HudState>) {
    let Some(cmd) = hud.pending_command.take() else {
        return;
    };

    debug!(
        "command_dispatch_system — mode={:?} text='{}'",
        cmd.mode, cmd.text
    );

    // TODO: dispatch to OrchestratorClient or emit a Bevy event.
    // For now, we just consume the intent so the console is responsive.
}
