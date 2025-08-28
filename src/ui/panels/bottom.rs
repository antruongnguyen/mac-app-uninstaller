use crate::ui::GuiState;
use eframe::egui;
use eframe::epaint::Color32;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Render the bottom status bar.
pub fn show(ctx: &egui::Context, state: &Arc<Mutex<GuiState>>) {
    egui::TopBottomPanel::bottom("bottom_status")
        .resizable(false)
        .show(ctx, |ui| {
            let (apps_len, related_paths): (usize, Vec<PathBuf>) = {
                let s = state.lock().unwrap();
                (s.apps.len(), s.related_paths.clone())
            };

            // categorize related paths
            let mut prefs = 0usize;
            let mut receipts = 0usize;
            let mut caches = 0usize;
            let mut app_support = 0usize;
            let mut containers = 0usize;
            let mut logs = 0usize;
            let mut launch_agents = 0usize;
            let mut other = 0usize;

            for p in &related_paths {
                let ps = p.to_string_lossy();
                let lower = ps.to_lowercase();
                let mut counted = false;
                if lower.contains("/library/preferences") || ps.ends_with(".plist") {
                    prefs += 1;
                    counted = true;
                }
                if lower.starts_with("/private/var/db/receipts") || lower.contains("/library/receipts") {
                    receipts += 1;
                    counted = true;
                }
                if lower.contains("/library/caches") {
                    caches += 1;
                    counted = true;
                }
                if lower.contains("/library/application support") {
                    app_support += 1;
                    counted = true;
                }
                if lower.contains("/library/containers") {
                    containers += 1;
                    counted = true;
                }
                if lower.contains("/library/logs") {
                    logs += 1;
                    counted = true;
                }
                if lower.contains("/library/launchagents") {
                    launch_agents += 1;
                    counted = true;
                }
                if !counted {
                    other += 1;
                }
            }
            let total_related = related_paths.len();

            ui.horizontal(|ui| {
                ui.set_height(32.0);
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "Applications: {}  •  Related: {} (Prefs {}, Receipts {}, Caches {}, Support {}, Containers {}, Logs {}, Agents {}, Other {})",
                            apps_len, total_related, prefs, receipts, caches, app_support, containers, logs, launch_agents, other
                        ))
                        .color(Color32::from_rgb(110, 112, 124))
                        .monospace(),
                    );
                });
            });
        });
}
