//! [`ThemePlugin`] — registers the theme resource and the apply system.
//!
//! On startup the plugin inserts a [`ThemeManager`] pre-loaded with the
//! TET Orchestrator theme. Each frame, a system checks whether the active
//! theme has changed (via the generation counter) and applies the
//! corresponding [`egui::Visuals`] to the primary egui context.

use bevy::log::{debug, info};
use bevy::prelude::*;
use bevy_egui::EguiContexts;

use super::tet_theme::{apply_theme_to_egui, tet_theme};
use super::theme::ThemeManager;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Registers the theme subsystem.
///
/// * Inserts a [`ThemeManager`] resource seeded with the default theme(s).
/// * Adds an `Update` system that reacts to theme switches by applying
///   the new [`egui::Visuals`] to the primary egui context.
pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        info!("ThemePlugin::build — registering theme systems");

        let manager = ThemeManager::new(vec![tet_theme()]);

        app.insert_resource(manager)
            .add_systems(Update, theme_apply_system);
    }
}

// ---------------------------------------------------------------------------
// Apply system
// ---------------------------------------------------------------------------

/// Detects theme changes via the [`ThemeManager::generation`] counter and
/// pushes updated visuals into the primary egui context.
///
/// Uses a [`Local<u64>`] to remember the last applied generation so that
/// the (relatively expensive) visuals rebuild only runs on actual switches.
fn theme_apply_system(
    manager: Res<ThemeManager>,
    mut last_gen: Local<u64>,
    mut contexts: EguiContexts,
) {
    if *last_gen == manager.generation {
        return;
    }
    *last_gen = manager.generation;

    let theme = manager.current();
    let visuals = apply_theme_to_egui(theme);

    if let Ok(ctx) = contexts.ctx_mut() {
        ctx.set_visuals(visuals);
        info!(
            "theme_apply_system — applied '{}' (gen={})",
            theme.name, manager.generation,
        );
    } else {
        debug!("theme_apply_system — egui context not available yet, skipping");
    }
}
