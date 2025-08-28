//! Egui-based UI for the App Uninstaller.
//!
//! This module defines the application state, the eframe App implementation,
//! and wires UI actions to background tasks defined in ui::tasks.

use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

use eframe::{egui, App};

use crate::core::reveal_in_finder;
use crate::style::set_appkit_style;
use crate::types::{AppInfo, ProgressUpdate, TaskKind};
use crate::osx::open_full_disk_access_settings;

/// Shared UI state synchronized across UI thread and worker threads.
pub struct GuiState {
    pub apps: Vec<AppInfo>,
    pub selected_index: Option<usize>,
    pub related_paths: Vec<PathBuf>,
    pub related_selected: Vec<bool>,

    // progress channel
    pub progress_tx: mpsc::Sender<ProgressUpdate>,
    pub progress_rx: mpsc::Receiver<ProgressUpdate>,
    pub current_task: TaskKind,
    pub current_progress: f32,
    pub current_message: String,
    pub task_running: bool,

    // status log
    pub status_msgs: Vec<String>,
}

impl GuiState {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            apps: Vec::new(),
            selected_index: None,
            related_paths: Vec::new(),
            related_selected: Vec::new(),
            progress_tx: tx,
            progress_rx: rx,
            current_task: TaskKind::Idle,
            current_progress: 0.0,
            current_message: String::new(),
            task_running: false,
            status_msgs: Vec::new(),
        }
    }
}

/// Main eframe application that renders and controls the UI.
pub struct MacUninstallerApp {
    pub state: Arc<Mutex<GuiState>>,
}

/// Start with an empty state and immediately trigger an apps refresh.
impl Default for MacUninstallerApp {
    fn default() -> Self {
        let state = Arc::new(Mutex::new(GuiState::new()));
        // kick off initial refresh in background
        {
            let st = state.clone();
            super::ui::tasks::spawn_refresh_apps(st.clone());
        }
        Self { state }
    }
}

/// Egui frame update: handles theme, progress messages, and UI layout.
impl App for MacUninstallerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Ensure Dock icon is set after eframe/winit initialization (macOS only)
        #[cfg(target_os = "macos")]
        {
            use std::sync::Once;
            static SET_ICON_ONCE: Once = Once::new();
            SET_ICON_ONCE.call_once(|| {
                crate::osx::try_set_dock_icon_from_icns();
            });
        }
        // apply theme
        set_appkit_style(ctx);

        // pull updates from progress channel (non-blocking)
        {
            let mut s = self.state.lock().unwrap();
            while let Ok(update) = s.progress_rx.try_recv() {
                s.current_task = update.kind.clone();
                s.current_progress = update.progress;
                s.current_message = update.message.clone();
                s.task_running = !update.finished;
                if let Some(err) = update.error {
                    s.status_msgs.push(format!("Error: {}", err));
                }
                if update.finished {
                    // append summary to status
                    match update.kind {
                        TaskKind::RefreshApps => s.status_msgs.push("Refreshed app list".to_string()),
                        TaskKind::RefreshRelated(_) => s.status_msgs.push("Refreshed related files".to_string()),
                        TaskKind::Uninstall(_) => s.status_msgs.push("Uninstall finished".to_string()),
                        _ => {}
                    }
                }
            }
        }

        let scale = ctx.pixels_per_point();

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.add_space(8.0 * scale);
            ui.horizontal(|ui| {
                ui.heading(format!("📦 App Uninstaller v{}", env!("CARGO_PKG_VERSION")));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("annguyen.apps@gmail.com");
                });
            });
            ui.add_space(6.0 * scale);
        });

        // Bottom status panel: compact status bar. Prevent expansion and resizing.
        egui::TopBottomPanel::bottom("bottom_status")
            .resizable(false)
            .default_height(24.0)
            .show(ctx, |ui| {
                let (apps_len, related_paths) = {
                    let s = self.state.lock().unwrap();
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
                    ui.centered_and_justified(|ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "Applications: {}  •  Related: {} (Prefs {}, Receipts {}, Caches {}, Support {}, Containers {}, Logs {}, Agents {}, Other {})",
                                apps_len, total_related, prefs, receipts, caches, app_support, containers, logs, launch_agents, other
                            ))
                            .color(egui::Color32::BLACK).monospace(),
                        );
                    });
                });
            });

        // Sidebar
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .default_width(260.0)
            .show(ctx, |ui| {
                ui.add_space(4.0 * scale);
                // Header row: Applications label on left, Refresh button on right
                let disabled = {
                    let s = self.state.lock().unwrap();
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
                            let st = self.state.clone();
                            super::ui::tasks::spawn_refresh_apps(st);
                        }
                    });
                });
                ui.separator();
                ui.add_space(2.0 * scale);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let apps_clone = { self.state.lock().unwrap().apps.clone() };
                    for (i, app) in apps_clone.iter().enumerate() {
                        let mut label = app.name.clone();
                        if app.running {
                            label = format!("{} • running", label);
                        }
                        let selected = { self.state.lock().unwrap().selected_index == Some(i) };
                        let full_width = ui.available_width();
                        let resp = ui.add_sized(
                            [full_width, 0.0],
                            egui::Button::selectable(selected, label),
                        );
                        if resp.clicked() {
                            // update selection and load related in background
                            {
                                let mut s = self.state.lock().unwrap();
                                s.selected_index = Some(i);
                                s.related_paths.clear();
                                s.related_selected.clear();
                            }
                            let st = self.state.clone();
                            super::ui::tasks::spawn_refresh_related_for_selected(st, i);
                        }
                    }
                });
            });

        // Main panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(8.0 * scale);

            let (selected_opt, related_clone, related_selected_clone, task_running, progress, message, _status_msgs, current_task) =
                {
                    let s = self.state.lock().unwrap();
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
                let apps_snapshot = { self.state.lock().unwrap().apps.clone() };
                if let Some(app) = apps_snapshot.get(idx) {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.heading(egui::RichText::new(&app.name).size(20.0 * scale));
                            ui.label(format!("Bundle ID: {}", app.bundle_id.clone().unwrap_or_default()));
                            ui.label(format!("Path: {}", app.path.display()));
                            if app.running {
                                ui.colored_label(egui::Color32::from_rgb(200, 70, 70), "⚠ Application is running");
                            }
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            if ui.button("Show in Finder").clicked() {
                                if let Err(e) = reveal_in_finder(&app.path) {
                                    let mut s = self.state.lock().unwrap();
                                    s.status_msgs.push(format!("Cannot reveal in Finder: {:?}", e));
                                }
                            }
                        });
                    });

                    ui.add_space(8.0 * scale);
                    ui.separator();
                    ui.add_space(8.0 * scale);

                    // Actions: placed above Related section
                    ui.horizontal(|ui| {
                        // Uninstall button (disabled if a task running or app running)
                        let s = self.state.lock().unwrap();
                        let uninstall_disabled = s.task_running || app.running;
                        drop(s);
                        if ui.add_enabled(
                            !uninstall_disabled,
                            egui::Button::new(
                                egui::RichText::new("🗑 Uninstall").color(egui::Color32::WHITE),
                            )
                            .fill(egui::Color32::from_rgb(220, 68, 68)),
                        ).clicked() {
                            // before uninstall, check if related contains receipts -> show Full Disk Access warning
                            let rel = { self.state.lock().unwrap().related_paths.clone() };
                            let has_receipts = rel.iter().any(|p| p.starts_with("/private/var/db/receipts"));
                            if has_receipts {
                                // append status and open settings prompt
                                let mut s = self.state.lock().unwrap();
                                s.status_msgs.push("Operation touches system receipts; Full Disk Access may be required.".into());
                                // Optionally open system prefs for Full Disk Access:
                                open_full_disk_access_settings();
                            }
                            // spawn uninstall
                            let st = self.state.clone();
                            super::ui::tasks::spawn_uninstall_selected(st, idx);
                        }

                        if ui.button("Scan Related Resources").clicked() {
                            let st = self.state.clone();
                            super::ui::tasks::spawn_refresh_related_for_selected(st, idx);
                        }

                        // Show select all/none toggle if there are related items
                        let (has_related, all_selected) = {
                            let s = self.state.lock().unwrap();
                            let n = s.related_paths.len();
                            let all_sel = n > 0 && s.related_selected.iter().take(n).all(|b| *b);
                            (n > 0, all_sel)
                        };
                        if has_related {
                            let label = if all_selected { "Select None" } else { "Select All" };
                            if ui.button(label).on_hover_text("Select/Deselect all related items to be deleted").clicked() {
                                let mut s = self.state.lock().unwrap();
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

                    ui.add_space(8.0 * scale);

                    // Show progress for "Finding related files..." between buttons and the Related section
                    if matches!(current_task, TaskKind::RefreshRelated(_)) && task_running {
                        ui.label(message.clone());
                        ui.add(egui::ProgressBar::new(progress).show_percentage());
                        ui.add_space(6.0 * scale);
                    }

                    let label = if related_clone.is_empty() { "Related Files & Folders" } else { "Related Files & Folders To Be Deleted (REVIEW CAREFULLY)" };
                    ui.label(egui::RichText::new(label).strong().size(16.0).color(egui::Color32::DARK_RED));
                    ui.add_space(6.0 * scale);

                    if related_clone.is_empty() {
                        ui.label("No related data found.");
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
                                        let checkbox = ui.checkbox(&mut checked, p.display().to_string());
                                        if checkbox.clicked() {
                                            // update real state
                                            let mut s = self.state.lock().unwrap();
                                            if i < s.related_selected.len() {
                                                s.related_selected[i] = checked;
                                            }
                                        }
                                        if ui.small_button("Reveal").clicked() {
                                            if let Err(e) = reveal_in_finder(p) {
                                                let mut s = self.state.lock().unwrap();
                                                s.status_msgs.push(format!("Cannot reveal {}: {:?}", p.display(), e));
                                            }
                                        }
                                    });
                                }
                            });
                    }

                    ui.add_space(8.0 * scale);
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

        // request repaint for smooth progress updates
        ctx.request_repaint_after(Duration::from_millis(16));
    }
}

pub mod tasks;
