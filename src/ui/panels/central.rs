use crate::core::reveal_in_finder;
use crate::osx::open_full_disk_access_settings;
use crate::types::TaskKind;
use crate::ui::tasks;
use crate::ui::GuiState;
use eframe::egui;
use std::sync::{Arc, Mutex};

/// Render the central panel with app details and actions.
pub fn show(ctx: &egui::Context, state: &Arc<Mutex<GuiState>>) {
    egui::CentralPanel::default().show(ctx, |ui| {
        let (selected_opt, related_clone, related_selected_clone, task_running, progress, message, _status_msgs, current_task) = {
            let s = state.lock().unwrap();
            (
                s.selected_index,
                s.related_paths.clone(),
                s.related_selected.clone(),
                s.task_running,
                s.current_progress,
                s.current_message.clone(),
                s.status_msgs.clone(),
                s.current_task.clone(),
            )
        };

        if let Some(idx) = selected_opt {
            // show details for selected app
            let apps_snapshot = { state.lock().unwrap().apps.clone() };
            if let Some(app) = apps_snapshot.get(idx) {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.heading(egui::RichText::new(&app.name).strong().size(20.0));
                        ui.label(format!("Bundle ID: {}", app.bundle_id.clone().unwrap_or_default()));
                        ui.label(format!("Path: {}", app.path.display()));
                        if app.running {
                            ui.colored_label(egui::Color32::from_rgb(200, 70, 70), "⚠ Application is running");
                        }
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        if ui.button("Show in Finder").clicked() {
                            if let Err(e) = reveal_in_finder(&app.path) {
                                let mut s = state.lock().unwrap();
                                s.status_msgs.push(format!("Cannot reveal in Finder: {:?}", e));
                            }
                        }
                    });
                });

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                // Actions: placed above Related section
                ui.horizontal(|ui| {
                    // Uninstall button (disabled if a task running or app running)
                    let s = state.lock().unwrap();
                    let uninstall_disabled = s.task_running || app.running;
                    drop(s);
                    if ui
                        .add_enabled(
                            !uninstall_disabled,
                            egui::Button::new(egui::RichText::new("🗑 Uninstall").color(egui::Color32::WHITE))
                                .fill(egui::Color32::from_rgb(220, 68, 68)),
                        )
                        .clicked()
                    {
                        // before uninstall, check if related contains receipts -> show Full Disk Access warning
                        let rel = { state.lock().unwrap().related_paths.clone() };
                        let has_receipts = rel.iter().any(|p| p.starts_with("/private/var/db/receipts"));
                        if has_receipts {
                            // append status and open settings prompt
                            let mut s = state.lock().unwrap();
                            s.status_msgs.push(
                                "Operation touches system receipts; Full Disk Access may be required.".into(),
                            );
                            // Optionally open system prefs for Full Disk Access:
                            open_full_disk_access_settings();
                        }
                        // spawn uninstall
                        let st = state.clone();
                        tasks::spawn_uninstall_selected(st, idx);
                    }

                    if ui.button("Scan Related Resources").clicked() {
                        let st = state.clone();
                        tasks::spawn_refresh_related_for_selected(st, idx);
                    }

                    // Show select all/none toggle if there are related items
                    let (has_related, all_selected) = {
                        let s = state.lock().unwrap();
                        let n = s.related_paths.len();
                        let all_sel = n > 0 && s.related_selected.iter().take(n).all(|b| *b);
                        (n > 0, all_sel)
                    };
                    if has_related {
                        let label = if all_selected { "Select None" } else { "Select All" };
                        if ui
                            .button(label)
                            .on_hover_text("Select/Deselect all related items to be deleted")
                            .clicked()
                        {
                            let mut s = state.lock().unwrap();
                            let n = s.related_paths.len();
                            let new_val = !all_selected;
                            if s.related_selected.len() < n {
                                s.related_selected.resize(n, new_val);
                            }
                            for i in 0..n {
                                s.related_selected[i] = new_val;
                            }
                        }
                    }
                });

                ui.add_space(8.0);

                let label = if related_clone.is_empty() {
                    "Related Files & Folders"
                } else {
                    "Related Files & Folders To Be Deleted (REVIEW CAREFULLY)"
                };
                ui.label(
                    egui::RichText::new(label)
                        .strong()
                        .size(16.0)
                        .color(egui::Color32::DARK_RED),
                );
                ui.add_space(6.0);

                if related_clone.is_empty() {
                    // Show progress for "Finding related files..." between buttons and the Related section
                    if matches!(current_task, TaskKind::RefreshRelated(_)) && task_running {
                        ui.label(message.clone());
                        ui.add(egui::ProgressBar::new(progress).desired_height(6.0));
                        ui.add_space(6.0);
                    } else {
                        ui.label("No related data found.");
                    }
                } else {
                    egui::ScrollArea::both()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            // Make the list expand to the width of the panel and avoid line-wrapping
                            ui.set_width(ui.available_width());
                            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);

                            for (i, p) in related_clone.iter().enumerate() {
                                let mut checked = related_selected_clone.get(i).cloned().unwrap_or(true);
                                ui.horizontal(|ui| {
                                    if ui.small_button("Reveal").clicked() {
                                        if let Err(e) = reveal_in_finder(p) {
                                            let mut s = state.lock().unwrap();
                                            s.status_msgs.push(format!("Cannot reveal {}: {:?}", p.display(), e));
                                        }
                                    }
                                    let checkbox = ui.checkbox(&mut checked, p.display().to_string());
                                    if checkbox.clicked() {
                                        // update real state
                                        let mut s = state.lock().unwrap();
                                        if i < s.related_selected.len() {
                                            s.related_selected[i] = checked;
                                        }
                                    }
                                });
                            }
                        });
                }

                ui.add_space(8.0);
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Selected index out of range");
                });
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Select an application from the left to see details.");
            });
        }
    });
}
