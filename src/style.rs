//! AppKit-inspired light theme for egui components.

use crate::ui::color::roles as colors;
use eframe::egui;
use egui::{Color32, Stroke};
use std::sync::Arc;

fn load_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Load multiple fonts
    fonts.font_data.insert(
        "Heading".to_owned(),
        Arc::from(egui::FontData::from_static(include_bytes!(
            "../resources/fonts/BaiJamjuree-SemiBold.ttf"
        ))),
    );

    fonts.font_data.insert(
        "Body".to_owned(),
        Arc::from(egui::FontData::from_static(include_bytes!(
            "../resources/fonts/BaiJamjuree-Regular.ttf"
        ))),
    );

    // Configure font families
    fonts.families.insert(
        egui::FontFamily::Proportional,
        vec!["Body".to_owned(), "Heading".to_owned()],
    );

    // You can also create custom font families
    fonts.families.insert(
        egui::FontFamily::Name("Heading".into()),
        vec!["Heading".to_owned()],
    );

    ctx.set_fonts(fonts);
}

/// Apply a macOS-like light theme to the current egui Context.
pub fn set_appkit_style(ctx: &egui::Context) {
    load_fonts(ctx);

    use egui::Visuals;

    let mut visuals = Visuals::light();

    visuals.window_fill = colors::window_bg();
    // Finder-like gray (left side)
    // visuals.panel_fill = Color32::from_rgb(233, 231, 225);
    // Finder-like gray (right side)
    // visuals.panel_fill = Color32::from_rgb(252, 250, 244);
    // JetBrains-like gray
    visuals.panel_fill = colors::panel_bg();

    // macOS Blue accents and hover
    let accent = colors::accent();
    let hover_bg = colors::hover_bg();

    // visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(255, 255, 255);
    visuals.widgets.noninteractive.bg_stroke.color = colors::border();
    // visuals.widgets.inactive.bg_fill = Color32::WHITE;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, colors::text());

    // Hover button background color
    visuals.widgets.hovered.bg_fill = hover_bg;
    // Hover button border color
    visuals.widgets.hovered.bg_stroke.color = Color32::TRANSPARENT;
    // Hover button text color
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, colors::text());

    // Clicked button text color
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, colors::text());
    // Clicked button border color
    visuals.widgets.active.bg_stroke.color = Color32::TRANSPARENT;

    // visuals.selection.bg_fill = Color32::TRANSPARENT;
    // Background color of selected text and progress bar
    visuals.selection.bg_fill = accent;
    // Text color of selected text and progress bar
    visuals.selection.stroke.color = colors::text_inverse();

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
