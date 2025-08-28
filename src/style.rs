//! AppKit-inspired light theme for egui components.

use eframe::egui;
use egui::{Color32, Stroke};

/// Apply a macOS-like light theme to the current egui Context.
pub fn set_appkit_style(ctx: &egui::Context) {
    use egui::Visuals;

    let mut visuals = Visuals::light();

    visuals.window_fill = Color32::from_rgb(252, 250, 244);
    // Finder-like gray (left side)
    // visuals.panel_fill = Color32::from_rgb(233, 231, 225);
    // Finder-like gray (right side)
    // visuals.panel_fill = Color32::from_rgb(252, 250, 244);
    // JetBrains-like gray
    visuals.panel_fill = Color32::from_rgb(247, 248, 250);

    // macOS Blue accents and hover
    let accent = Color32::from_rgb(58, 128, 246);
    let hover_bg = Color32::from_rgb(245, 245, 247);

    // visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(255, 255, 255);
    visuals.widgets.noninteractive.bg_stroke.color = Color32::from_rgb(235, 236, 240);
    // visuals.widgets.inactive.bg_fill = Color32::WHITE;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(30, 30, 30));

    // Hover button background color
    visuals.widgets.hovered.bg_fill = hover_bg;
    // Hover button border color
    visuals.widgets.hovered.bg_stroke.color = Color32::TRANSPARENT;
    // Hover button text color
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::from_rgb(20, 20, 20));

    // Clicked button text color
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::from_rgb(20, 20, 20));
    // Clicked button border color
    visuals.widgets.active.bg_stroke.color = Color32::TRANSPARENT;

    // visuals.selection.bg_fill = Color32::TRANSPARENT;
    // Background color of selected text and progress bar
    visuals.selection.bg_fill = accent;
    // Text color of selected text and progress bar
    visuals.selection.stroke.color = Color32::WHITE;

    visuals.hyperlink_color = accent;

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();

    // Set font weights to normal

    // Tighter layout akin to AppKit
    // Disable text selection
    style.interaction.selectable_labels = false;
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
    style.spacing.window_margin = egui::Margin::symmetric(8, 8);

    // Slightly larger heading, readable body
    style
        .text_styles
        .insert(egui::TextStyle::Heading, egui::FontId::proportional(20.0));
    style
        .text_styles
        .insert(egui::TextStyle::Body, egui::FontId::proportional(14.0));
    style
        .text_styles
        .insert(egui::TextStyle::Monospace, egui::FontId::monospace(13.0));

    ctx.set_style(style);
}
