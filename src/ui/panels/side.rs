use eframe::egui;
use std::sync::{Arc, Mutex};

use crate::types::StateColors;
use crate::ui::GuiState;
use crate::ui::{list, tasks};

use crate::ui::color::roles as colors;
use egui::Vec2;

/// Render the left sidebar with apps list and refresh button.
pub fn show(ctx: &egui::Context, state: &Arc<Mutex<GuiState>>) {
    egui::SidePanel::left("sidebar")
        .resizable(false)
        .exact_width(260.0)
        .show(ctx, |ui| {
            ui.add_space(4.0);
            // Header row: Applications label on left, Refresh button on right
            let disabled = {
                let s = state.lock().unwrap();
                s.task_running
            };
            ui.horizontal(|ui| {
                ui.set_height(32.0);
                ui.label(egui::RichText::new("APPLICATIONS").strong().size(16.0));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let tx = ui.add_enabled(!disabled, egui::Button::new("Refresh"));
                    if tx.clicked() {
                        let st = state.clone();
                        tasks::spawn_refresh_apps(st);
                    }
                });
            });
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                let apps_clone = { state.lock().unwrap().apps.clone() };
                for (i, app) in apps_clone.iter().enumerate() {
                    let mut label = app.name.clone();
                    if app.running {
                        label = format!("{} • running", label);
                    }
                    let selected = { state.lock().unwrap().selected_index == Some(i) };
                    let full_width = ui.available_width();
                    let resp = list::list_item(
                        ui,
                        &label,
                        Vec2::new(full_width, 24.0),
                        selected,
                        app.running,
                        StateColors {
                            default: colors::list_bg_default(),
                            hover: colors::list_bg_hover(),
                            selected: Some(colors::list_bg_selected()),
                        },
                    );
                    // let resp = default_list_item(ui, &label, Vec2::new(full_width, 24.0), selected);
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
