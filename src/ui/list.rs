use crate::types::StateColors;
use eframe::emath::{Align2, Vec2};
use eframe::epaint::{FontId, StrokeKind};
use egui::{Response, Sense, Ui};

pub fn list_item(
    ui: &mut Ui,
    text: &str,
    size: Vec2,
    selected: bool,
    colors: StateColors,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    if ui.is_rect_visible(rect) {
        let mut visuals = ui.style().interact_selectable(&response, selected);

        // Override background color based on state
        let bg_color = if selected {
            colors.selected.unwrap_or(visuals.bg_fill)
        } else if response.hovered() {
            colors.hover
        } else {
            colors.default
        };

        visuals.bg_fill = bg_color;

        // Draw button background
        let border_radius = 2.0;
        ui.painter()
            .rect_filled(rect, border_radius, visuals.bg_fill);
        ui.painter()
            .rect_stroke(rect, border_radius, visuals.bg_stroke, StrokeKind::Middle);

        // Draw left-aligned text
        let text_pos = rect.left_center() + Vec2::new(10.0, 0.0);
        ui.painter().text(
            text_pos,
            Align2::LEFT_CENTER,
            text,
            FontId::default(),
            visuals.text_color(),
        );
    }

    response
}
