//! Bottom console HUD panel — command input, mode selector, execute.
//!
//! Provides a text input field for sending commands to agents,
//! a mode selector (Direct / Brainstorm / Query / Halt), and
//! an execute button.
//!
//! Commands are placed into [`HudState::pending_command`] as intents;
//! actual dispatch happens in [`super::state_systems::command_dispatch_system`].

use bevy::log::{debug, trace};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use super::battle_command::{BattleCommandHudState, ShellVisibilityTier};
use super::styles::glass_panel_frame;
use super::{CommandMode, HudState, PendingCommand};

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Renders the bottom console panel with command input and mode selector.
pub fn bottom_console_system(mut contexts: EguiContexts, mut hud: ResMut<HudState>) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    trace!("bottom_console_system — rendering");

    let frame = glass_panel_frame(ctx);

    egui::TopBottomPanel::bottom("hud_bottom_console")
        .frame(frame)
        .min_height(48.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                render_mode_selector(ui, &mut hud);
                ui.separator();
                render_command_input(ui, &mut hud);
                ui.separator();
                render_execute_button(ui, &mut hud);
            });
        });
}

// ---------------------------------------------------------------------------
// Render helpers
// ---------------------------------------------------------------------------

/// Renders the command mode selector as a row of clickable labels.
fn render_mode_selector(ui: &mut egui::Ui, hud: &mut HudState) {
    let modes = [
        (CommandMode::Direct, "Direct"),
        (CommandMode::Brainstorm, "Brainstorm"),
        (CommandMode::Query, "Query"),
        (CommandMode::Halt, "Halt"),
    ];

    for (mode, label) in &modes {
        let is_active = hud.command_mode == *mode;
        let color = if is_active {
            mode_color(*mode)
        } else {
            egui::Color32::from_rgb(128, 128, 128)
        };
        let text = egui::RichText::new(*label).color(color).strong();
        if ui.selectable_label(is_active, text).clicked() {
            debug!("bottom_console — mode changed to {:?}", mode);
            hud.command_mode = *mode;
        }
    }
}

/// Renders the command text input field.
fn render_command_input(ui: &mut egui::Ui, hud: &mut HudState) {
    let response = ui.add(
        egui::TextEdit::singleline(&mut hud.command_input)
            .desired_width(ui.available_width() - 80.0)
            .hint_text("Enter command...")
            .text_color(egui::Color32::from_rgb(230, 235, 242)),
    );

    // Submit on Enter key
    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        submit_command(hud);
    }
}

/// Renders the execute button.
fn render_execute_button(ui: &mut egui::Ui, hud: &mut HudState) {
    let button_color = mode_color(hud.command_mode);
    let button = egui::Button::new(egui::RichText::new("Execute").color(egui::Color32::WHITE))
        .fill(button_color.gamma_multiply(0.6));

    if ui.add(button).clicked() {
        debug!("bottom_console — execute button clicked");
        submit_command(hud);
    }
}

/// Places a command intent into [`HudState::pending_command`] and clears the input.
fn submit_command(hud: &mut HudState) {
    let text = hud.command_input.trim().to_string();
    if text.is_empty() {
        return;
    }
    debug!(
        "bottom_console::submit_command — mode={:?} text='{}'",
        hud.command_mode, text
    );
    hud.pending_command = Some(PendingCommand {
        text,
        mode: hud.command_mode,
    });
    hud.command_input.clear();
}

/// Returns the accent colour for a command mode.
fn mode_color(mode: CommandMode) -> egui::Color32 {
    match mode {
        CommandMode::Direct => egui::Color32::from_rgb(0, 204, 255), // cyan
        CommandMode::Brainstorm => egui::Color32::from_rgb(153, 102, 255), // purple
        CommandMode::Query => egui::Color32::from_rgb(51, 230, 102), // green
        CommandMode::Halt => egui::Color32::from_rgb(255, 77, 77),   // red
    }
}
