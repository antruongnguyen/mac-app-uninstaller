use eframe::egui;
use std::sync::{Arc, Mutex};

use crate::ui::tasks;
use crate::ui::GuiState;

/// Render the left sidebar with apps list and refresh button.
pub fn show(ctx: &egui::Context, state: &Arc<Mutex<GuiState>>) {
    let scale = ctx.pixels_per_point();

    egui::SidePanel::left("sidebar")
        .resizable(false)
        .default_width(260.0)
        .show(ctx, |ui| {
            ui.add_space(4.0 * scale);
            // Header row: Applications label on left, Refresh button on right
            let disabled = {
                let s = state.lock().unwrap();
                s.task_running
            };
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Applications")
                        .strong()
                        .size(16.0)
                        .color(egui::Color32::BLACK),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let tx = ui.add_enabled(!disabled, egui::Button::new("Refresh"));
                    if tx.clicked() {
                        let st = state.clone();
                        tasks::spawn_refresh_apps(st);
                    }
                });
            });
            ui.separator();
            ui.add_space(2.0 * scale);
            egui::ScrollArea::vertical().show(ui, |ui| {
                let apps_clone = { state.lock().unwrap().apps.clone() };
                for (i, app) in apps_clone.iter().enumerate() {
                    let mut label = app.name.clone();
                    if app.running {
                        label = format!("{} • running", label);
                    }
                    let selected = { state.lock().unwrap().selected_index == Some(i) };
                    let full_width = ui.available_width();
                    let resp =
                        ui.add_sized([full_width, 0.0], egui::Button::selectable(selected, label));
                    if resp.clicked() {
                        // update selection and load related in background
                        {
                            let mut s = state.lock().unwrap();
                            s.selected_index = Some(i);
                            s.related_paths.clear();
                            s.related_selected.clear();
                        }
                        let st = state.clone();
                        tasks::spawn_refresh_related_for_selected(st, i);
                    }
                }
            });
        });
}
