//! Faction selector UI widget for changing faction themes.
//!
//! Provides an egui widget for selecting faction themes in settings.

use bevy_egui::egui;

use super::faction::{FactionId, FactionThemeManager};

/// Faction selector widget for egui.
pub fn faction_selector_ui(ui: &mut egui::Ui, manager: &mut FactionThemeManager) {
    ui.heading("Faction Selection");
    ui.separator();

    egui::Grid::new("faction_grid")
        .num_columns(3)
        .spacing([20.0, 10.0])
        .show(ui, |ui| {
            let factions = [
                (FactionId::English, "English", "Red Rose"),
                (FactionId::French, "French", "Fleur-de-lis"),
                (FactionId::Byzantine, "Byzantine", "Double Eagle"),
                (FactionId::Mongol, "Mongol", "Sky Wolf"),
            ];

            for (i, (faction_id, name, description)) in factions.iter().enumerate() {
                ui.vertical_centered(|ui| {
                    // Button with faction color
                    let theme = manager.get_theme(*faction_id).unwrap();
                    let primary = egui::Color32::from_rgb(
                        (theme.primary.0 * 255.0) as u8,
                        (theme.primary.1 * 255.0) as u8,
                        (theme.primary.2 * 255.0) as u8,
                    );

                    let button = egui::Button::new(
                        egui::RichText::new(*name)
                            .size(16.0)
                            .color(egui::Color32::WHITE),
                    )
                    .fill(primary)
                    .stroke(egui::Stroke::new(
                        2.0,
                        if manager.current.id == *faction_id {
                            egui::Color32::GOLD
                        } else {
                            egui::Color32::DARK_GRAY
                        },
                    ));

                    if ui.add(button).clicked() {
                        manager.transition_to(*faction_id);
                    }

                    ui.label(egui::RichText::new(*description).small());
                });

                if (i + 1) % 2 == 0 {
                    ui.end_row();
                }
            }
        });

    ui.separator();

    // Current faction display
    ui.horizontal(|ui| {
        ui.label("Current Faction:");
        ui.label(
            egui::RichText::new(manager.current.description.clone()).color(
                egui::Color32::from_rgb(
                    (manager.current.primary.0 * 255.0) as u8,
                    (manager.current.primary.1 * 255.0) as u8,
                    (manager.current.primary.2 * 255.0) as u8,
                ),
            ),
        );
    });

    // Transition progress
    if manager.target.is_some() {
        ui.horizontal(|ui| {
            ui.label("Transition:");
            let progress = manager.transition_progress;
            ui.add(egui::ProgressBar::new(progress).show_percentage());
        });
    }
}

/// Window-based faction selector.
pub fn faction_selector_window(
    egui_context: &mut bevy_egui::EguiContexts,
    manager: &mut FactionThemeManager,
    open: &mut bool,
) {
    if let Ok(ctx) = egui_context.ctx_mut() {
        egui::Window::new("Faction Selector")
            .open(open)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                faction_selector_ui(ui, manager);
            });
    }
}
