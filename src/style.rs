//! AppKit-inspired light theme for egui components.

use eframe::{egui, epaint};

/// Apply a macOS-like light theme to the current egui Context.
pub fn set_appkit_style(ctx: &egui::Context) {
    use egui::Visuals;

    let mut visuals = Visuals::light();
    visuals.window_fill = epaint::Color32::from_rgb(236, 236, 236); // like System Preferences
    visuals.panel_fill = epaint::Color32::from_rgb(255, 255, 255);
    visuals.widgets.active.bg_fill = epaint::Color32::from_rgb(0, 122, 255);
    visuals.widgets.active.fg_stroke = epaint::Stroke::new(1.0, epaint::Color32::WHITE);
    visuals.widgets.hovered.bg_fill = epaint::Color32::from_rgb(245, 245, 247);
    visuals.widgets.noninteractive.bg_fill = epaint::Color32::from_rgb(255, 255, 255);
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    ctx.set_style(style);
}
