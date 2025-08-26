use anyhow::{Context, Result};
use home::home_dir;
use eframe::{egui, epaint};
use plist::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::mpsc,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use sysinfo::System;
use walkdir::WalkDir;

// ---------- Data structures ----------
#[derive(Clone, Debug)]
struct AppInfo {
    path: PathBuf,
    name: String,
    bundle_id: Option<String>,
    running: bool,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
enum TaskKind {
    Idle,
    RefreshApps,
    RefreshRelated(usize), // index in apps
    Uninstall(usize),      // index in apps
}

#[derive(Clone, Debug)]
struct ProgressUpdate {
    kind: TaskKind,
    progress: f32,   // 0.0 ..= 1.0
    message: String, // human friendly
    finished: bool,  // whether task finished
    error: Option<String>,
}

// ---------- GUI state ----------
struct GuiState {
    apps: Vec<AppInfo>,
    selected_index: Option<usize>,
    related_paths: Vec<PathBuf>,
    related_selected: Vec<bool>,

    // progress channel
    progress_tx: mpsc::Sender<ProgressUpdate>,
    progress_rx: mpsc::Receiver<ProgressUpdate>,
    current_task: TaskKind,
    current_progress: f32,
    current_message: String,
    task_running: bool,

    // status log
    status_msgs: Vec<String>,
}

impl GuiState {
    fn new() -> Self {
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

// ---------- Main App ----------
struct MacUninstallerApp {
    state: Arc<Mutex<GuiState>>,
}

impl Default for MacUninstallerApp {
    fn default() -> Self {
        let state = Arc::new(Mutex::new(GuiState::new()));
        // kick off initial refresh in background
        {
            let st = state.clone();
            spawn_refresh_apps(st.clone());
        }
        Self { state }
    }
}

impl eframe::App for MacUninstallerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Ensure Dock icon is set after eframe/winit initialization (macOS only)
        #[cfg(target_os = "macos")]
        {
            use std::sync::Once;
            static SET_ICON_ONCE: Once = Once::new();
            SET_ICON_ONCE.call_once(|| {
                try_set_dock_icon_from_icns();
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
                        TaskKind::RefreshApps => {
                            s.status_msgs.push("Refreshed app list".to_string())
                        }
                        TaskKind::RefreshRelated(_) => {
                            s.status_msgs.push("Refreshed related files".to_string())
                        }
                        TaskKind::Uninstall(_) => {
                            s.status_msgs.push("Uninstall finished".to_string())
                        }
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
                            spawn_refresh_apps(st);
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
                            spawn_refresh_related_for_selected(st, i);
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
                            spawn_uninstall_selected(st, idx);
                        }

                        if ui.button("Scan Related Resources").clicked() {
                            let st = self.state.clone();
                            spawn_refresh_related_for_selected(st, idx);
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

                    // show warning if any related path under /private/var/db/receipts
                    let _needs_fda = related_clone.iter().any(|p| p.starts_with("/private/var/db/receipts"));

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

// ---------- Helpers: spawn background tasks ----------

fn spawn_refresh_apps(state_arc: Arc<Mutex<GuiState>>) {
    let tx;
    {
        let s = state_arc.lock().unwrap();
        tx = s.progress_tx.clone();
    }
    thread::spawn(move || {
        // send start
        let _ = tx.send(ProgressUpdate {
            kind: TaskKind::RefreshApps,
            progress: 0.0,
            message: "Scanning /Applications and ~/Applications...".into(),
            finished: false,
            error: None,
        });

        // call the real function but we can send coarse progress
        match find_app_bundles_progress(&tx) {
            Ok(list) => {
                // update apps in state
                let mut s = state_arc.lock().unwrap();
                s.apps = list;
                s.selected_index = None;
                s.related_paths.clear();
                s.related_selected.clear();
                s.status_msgs.push("App list refreshed.".into());
                let _ = tx.send(ProgressUpdate {
                    kind: TaskKind::RefreshApps,
                    progress: 1.0,
                    message: "Done.".into(),
                    finished: true,
                    error: None,
                });
            }
            Err(e) => {
                let mut s = state_arc.lock().unwrap();
                s.status_msgs.push(format!("Refresh apps failed: {:?}", e));
                let _ = tx.send(ProgressUpdate {
                    kind: TaskKind::RefreshApps,
                    progress: 1.0,
                    message: "Failed.".into(),
                    finished: true,
                    error: Some(format!("{:?}", e)),
                });
            }
        }
    });
}

fn spawn_refresh_related_for_selected(state_arc: Arc<Mutex<GuiState>>, idx: usize) {
    let tx;
    let app_opt = {
        let s = state_arc.lock().unwrap();
        tx = s.progress_tx.clone();
        s.apps.get(idx).cloned()
    };
    if app_opt.is_none() {
        let mut s = state_arc.lock().unwrap();
        s.status_msgs.push("Selected app not found.".into());
        return;
    }
    let app = app_opt.unwrap();

    thread::spawn(move || {
        let _ = tx.send(ProgressUpdate {
            kind: TaskKind::RefreshRelated(idx),
            progress: 0.0,
            message: format!("Finding related files for {}...", app.name),
            finished: false,
            error: None,
        });

        // We call find_related_paths (non-progressive) but simulate progress increments
        let maybe_paths = find_related_paths(app.bundle_id.as_deref(), Some(&app.name));
        // simulate progress quickly to show activity
        let steps = 4usize.max(maybe_paths.len());
        for i in 0..=steps {
            let p = (i as f32) / (steps as f32);
            let _ = tx.send(ProgressUpdate {
                kind: TaskKind::RefreshRelated(idx),
                progress: p,
                message: format!("Finding related files... {:.0}%", p * 100.0),
                finished: false,
                error: None,
            });
            thread::sleep(Duration::from_millis(80));
        }

        // push results to state
        {
            let mut s = state_arc.lock().unwrap();
            s.related_paths = maybe_paths;
            s.related_selected = vec![true; s.related_paths.len()];
        }

        let _ = tx.send(ProgressUpdate {
            kind: TaskKind::RefreshRelated(idx),
            progress: 1.0,
            message: "Related files loaded".into(),
            finished: true,
            error: None,
        });
    });
}

fn spawn_uninstall_selected(state_arc: Arc<Mutex<GuiState>>, idx: usize) {
    let (tx, app_opt, related_paths, related_selected) = {
        let s = state_arc.lock().unwrap();
        let tx = s.progress_tx.clone();
        let app = s.apps.get(idx).cloned();
        let related_paths = s.related_paths.clone();
        let related_selected = s.related_selected.clone();
        (tx, app, related_paths, related_selected)
    };

    if app_opt.is_none() {
        let mut s = state_arc.lock().unwrap();
        s.status_msgs.push("Selected app not found.".into());
        return;
    }

    let app = app_opt.unwrap();

    // Filter related items based on user selection
    let paths_to_remove: Vec<PathBuf> = related_paths
        .into_iter()
        .zip(related_selected.into_iter())
        .filter_map(|(p, sel)| if sel { Some(p) } else { None })
        .collect();

    // check for receipts path and warn (UI also appends a status)
    let needs_fda = paths_to_remove
        .iter()
        .any(|p| p.starts_with("/private/var/db/receipts"));
    if needs_fda {
        let mut s = state_arc.lock().unwrap();
        s.status_msgs.push(
            "This uninstall touches system receipts. Full Disk Access may be required.".into(),
        );
        // We don't stop execution; we let the OS enforce permissions and report errors.
    }

    let state_for_refresh = state_arc.clone();

    thread::spawn(move || {
        let _ = tx.send(ProgressUpdate {
            kind: TaskKind::Uninstall(idx),
            progress: 0.0,
            message: format!("Starting uninstall of {}...", app.name),
            finished: false,
            error: None,
        });

        // Step 1: move bundle to trash
        let total_steps = 1 + paths_to_remove.len();
        let mut step = 0usize;

        // Check running - if running, abort
        if is_app_running_simple(app.bundle_id.as_deref(), Some(&app.name)) {
            let _ = tx.send(ProgressUpdate {
                kind: TaskKind::Uninstall(idx),
                progress: 0.0,
                message: "App is running. Abort uninstall.".into(),
                finished: true,
                error: Some("App is running".into()),
            });
            return;
        }

        match move_to_trash_or_remove(&app.path) {
            Ok(_) => {
                step += 1;
                let _ = tx.send(ProgressUpdate {
                    kind: TaskKind::Uninstall(idx),
                    progress: (step as f32) / (total_steps as f32),
                    message: format!("Moved {} to Trash", app.path.display()),
                    finished: false,
                    error: None,
                });
            }
            Err(e) => {
                let _ = tx.send(ProgressUpdate {
                    kind: TaskKind::Uninstall(idx),
                    progress: 0.0,
                    message: format!("Failed to remove bundle: {:?}", e),
                    finished: true,
                    error: Some(format!("{:?}", e)),
                });
                return;
            }
        }

        // Step 2: related paths (only those the user checked)
        for p in paths_to_remove.iter() {
            match move_to_trash_or_remove(p) {
                Ok(_) => {
                    step += 1;
                    let _ = tx.send(ProgressUpdate {
                        kind: TaskKind::Uninstall(idx),
                        progress: (step as f32) / (total_steps as f32),
                        message: format!("Removed {}", p.display()),
                        finished: false,
                        error: None,
                    });
                }
                Err(e) => {
                    let _ = tx.send(ProgressUpdate {
                        kind: TaskKind::Uninstall(idx),
                        progress: (step as f32) / (total_steps as f32),
                        message: format!("Failed to remove {}: {:?}", p.display(), e),
                        finished: false,
                        error: Some(format!("{:?}", e)),
                    });
                }
            }
            // small pause so progress updates are visible
            thread::sleep(Duration::from_millis(120));
        }

        // finalization: send finished and trigger refresh
        let _ = tx.send(ProgressUpdate {
            kind: TaskKind::Uninstall(idx),
            progress: 1.0,
            message: "Uninstall complete; refreshing app list".into(),
            finished: true,
            error: None,
        });

        // Trigger automatic refresh of the apps list
        spawn_refresh_apps(state_for_refresh);
    });
}

// ---------- AppKit style ----------
fn set_appkit_style(ctx: &egui::Context) {
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

// ---------- Keep original core logic functions (mostly verbatim) ----------

/// tìm .app trong /Applications và ~/Applications
/// wrapper that gives coarse progress updates while scanning
fn find_app_bundles_progress(tx: &mpsc::Sender<ProgressUpdate>) -> Result<Vec<AppInfo>> {
    let candidates = candidate_app_dirs();

    // total candidate directories (for coarse progress)
    let total_dirs = candidates.len().max(1);
    let mut dir_idx = 0usize;

    let mut sys = System::new_all();
    sys.refresh_all();

    let mut res = Vec::new();
    for d in candidates.into_iter() {
        dir_idx += 1;
        let _ = tx.send(ProgressUpdate {
            kind: TaskKind::RefreshApps,
            progress: (dir_idx as f32 - 1.0) / (total_dirs as f32),
            message: format!("Scanning {:?}", d),
            finished: false,
            error: None,
        });

        if d.exists() && d.is_dir() {
            let mut v = scan_apps_in_dir(&sys, &d)?;
            res.append(&mut v);
        }

        // send intermediate progress
        let _ = tx.send(ProgressUpdate {
            kind: TaskKind::RefreshApps,
            progress: (dir_idx as f32) / (total_dirs as f32) * 0.9,
            message: format!("Scanned {:?}", d),
            finished: false,
            error: None,
        });
    }

    res.sort_by(|a, b| a.name.cmp(&b.name));
    // little delay for UX
    thread::sleep(Duration::from_millis(120));
    let _ = tx.send(ProgressUpdate {
        kind: TaskKind::RefreshApps,
        progress: 0.95,
        message: "Finalizing...".into(),
        finished: false,
        error: None,
    });

    Ok(res)
}

/// đọc CFBundleIdentifier và CFBundleName từ Contents/Info.plist
fn read_info_from_app(path: &Path) -> Result<(Option<String>, Option<String>)> {
    let info = path.join("Contents").join("Info.plist");
    if !info.exists() {
        return Ok((None, None));
    }
    let v = Value::from_file(&info).context("Read plist")?;
    let bundle_id = v
        .as_dictionary()
        .and_then(|dict| dict.get("CFBundleIdentifier"))
        .and_then(|v| v.as_string())
        .map(|s| s.to_string());
    let bundle_name = v
        .as_dictionary()
        .and_then(|dict| {
            dict.get("CFBundleName")
                .or_else(|| dict.get("CFBundleDisplayName"))
        })
        .and_then(|v| v.as_string())
        .map(|s| s.to_string());
    Ok((bundle_id, bundle_name))
}

/// minimal running check using sysinfo snapshot
fn is_app_running(sys: &System, bundle_id: Option<&str>, app_name: Option<&str>) -> bool {
    for (_pid, proc_) in sys.processes() {
        let name = proc_.name().to_string_lossy().to_lowercase();
        let cmdline = proc_
            .cmd()
            .iter()
            .map(|s| s.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        if let Some(bid) = bundle_id {
            let bid_l = bid.to_lowercase();
            if cmdline.contains(&bid_l) || name.contains(&bid_l) {
                return true;
            }
        }
        if let Some(an) = app_name {
            let an_l = an.to_lowercase();
            if name.contains(&an_l) || cmdline.contains(&an_l) {
                return true;
            }
        }
    }
    false
}

/// simpler runtime check (fresh System inside)
fn is_app_running_simple(bundle_id: Option<&str>, app_name: Option<&str>) -> bool {
    let mut sys = System::new_all();
    sys.refresh_all();
    is_app_running(&sys, bundle_id, app_name)
}

/// tìm các path liên quan (theo bundle id và app name)
fn find_related_paths(bundle_id: Option<&str>, app_name: Option<&str>) -> Vec<PathBuf> {
    let mut res: Vec<PathBuf> = Vec::new();
    let home = home_dir().unwrap_or_else(|| PathBuf::from("/Users/unknown"));

    if let Some(bid) = bundle_id {
        res.extend(common_paths_for_bundle_id(bid));
    }

    if let Some(name) = app_name {
        let libs = vec![
            home.join("Library").join("Application Support"),
            home.join("Library").join("Caches"),
            home.join("Library").join("Preferences"),
            home.join("Library").join("Containers"),
            home.join("Library").join("Logs"),
            home.join("Library").join("LaunchAgents"),
            PathBuf::from("/Library/Receipts"),
            PathBuf::from("/private/var/db/receipts"),
        ];
        for lib in libs {
            if lib.exists() && lib.is_dir() {
                for entry in WalkDir::new(&lib).max_depth(2).min_depth(1) {
                    if let Ok(ent) = entry {
                        if let Some(fname) = ent.file_name().to_str() {
                            if let Some(bid) = bundle_id {
                                if fname.to_lowercase().contains(&bid.to_lowercase()) {
                                    res.push(ent.path().to_path_buf());
                                }
                            }
                            if fname.to_lowercase().contains(&name.to_lowercase()) {
                                res.push(ent.path().to_path_buf());
                            }
                        }
                    }
                }
            }
        }
    }

    res.sort();
    res.dedup();
    res.retain(|p| p.exists());
    res
}

fn common_paths_for_bundle_id(bid: &str) -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(h) = home_dir() {
        v.push(h.join("Library").join("Application Support").join(bid));
        v.push(h.join("Library").join("Caches").join(bid));
        v.push(
            h.join("Library")
                .join("Preferences")
                .join(format!("{}.plist", bid)),
        );
        v.push(h.join("Library").join("Containers").join(bid));
    }
    v.push(
        PathBuf::from("/Library")
            .join("Application Support")
            .join(bid),
    );
    v.push(
        PathBuf::from("/Library")
            .join("Preferences")
            .join(format!("{}.plist", bid)),
    );
    v
}

/// move to trash (preferred) else remove directly
fn move_to_trash_or_remove(path: &Path) -> Result<()> {
    match trash::delete(path) {
        Ok(_) => Ok(()),
        Err(_trash_err) => {
            if path.is_dir() {
                fs::remove_dir_all(path)
                    .with_context(|| format!("Failed to remove dir {:?}", path))?;
            } else if path.is_file() {
                fs::remove_file(path)
                    .with_context(|| format!("Failed to remove file {:?}", path))?;
            } else {
                return Err(anyhow::anyhow!("Unknown path type: {:?}", path));
            }
            Ok(())
        }
    }
}

/// reveal path in Finder (macOS)
fn reveal_in_finder(path: &Path) -> Result<()> {
    if cfg!(target_os = "macos") {
        let p = path
            .canonicalize()
            .with_context(|| format!("Canon {:?}", path))?;
        std::process::Command::new("open")
            .arg("-R")
            .arg(p)
            .status()
            .with_context(|| "Failed to run open -R")?;
    } else {
        return Err(anyhow::anyhow!(
            "Reveal in Finder is supported only on macOS"
        ));
    }
    Ok(())
}

// ---------- Utilities ----------
#[allow(dead_code)]
fn open_full_disk_access_settings() {
    // Open System Settings → Privacy & Security → Full Disk Access
    if cfg!(target_os = "macos") {
        // Newer macOS may support x-apple.systempreferences url
        let _ = std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles")
            .spawn();
    }
}

// macOS: set Dock icon from our .icns if possible (bundle or dev path)
#[cfg(target_os = "macos")]
#[allow(deprecated)]
fn try_set_dock_icon_from_icns() {
    use cocoa::appkit::{NSApp, NSApplication, NSImage};
    use cocoa::base::{id, nil};
    use cocoa::foundation::{NSAutoreleasePool, NSString};
    use std::path::PathBuf;

    unsafe {
        let _pool = NSAutoreleasePool::new(nil);
        // Ensure NSApplication exists
        let _app = NSApplication::sharedApplication(nil);

        // Candidate locations
        let mut candidates: Vec<PathBuf> = Vec::new();
        if let Ok(exe) = std::env::current_exe() {
            // .../My.app/Contents/MacOS/exe -> .../My.app/Contents/Resources/icon.icns
            if let Some(contents) = exe.parent().and_then(|p| p.parent()) {
                candidates.push(contents.join("Resources").join("icon.icns"));
            }
        }
        if let Ok(cwd) = std::env::current_dir() {
            candidates.push(cwd.join("resources").join("icon.icns"));
            candidates.push(cwd.join("../resources").join("icon.icns"));
            candidates.push(cwd.join("../../resources").join("icon.icns"));
        }

        for p in candidates {
            if p.exists() {
                let ns_path = NSString::alloc(nil).init_str(&p.to_string_lossy());
                let img: id = NSImage::alloc(nil).initByReferencingFile_(ns_path);
                if img != nil {
                    let app = NSApp();
                    app.setApplicationIconImage_(img);
                    break;
                }
            }
        }
    }
}

// ---------- main ----------
fn main() -> eframe::Result<()> {
    // On macOS, proactively set the Dock icon from our bundle/dev resources
    #[cfg(target_os = "macos")]
    {
        try_set_dock_icon_from_icns();
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([1000.0, 700.0]),
        ..Default::default()
    };
    eframe::run_native(
        "App Uninstaller",
        native_options,
        Box::new(|_cc| Ok(Box::new(MacUninstallerApp::default()))),
    )
}

// ---------- Duplication reducers: helper functions ----------
fn candidate_app_dirs() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/Applications"),
        home_dir()
            .map(|h| h.join("Applications"))
            .unwrap_or_default(),
    ]
}

fn scan_apps_in_dir(sys: &System, dir: &Path) -> Result<Vec<AppInfo>> {
    let mut res = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("Read dir {:?}", dir))? {
        let e = entry?;
        let p = e.path();
        if p.extension().and_then(|s| s.to_str()) == Some("app") {
            let (bid, name) = read_info_from_app(&p).unwrap_or((None, None));
            let running = is_app_running(sys, bid.as_deref(), name.as_deref());
            res.push(AppInfo {
                path: p.clone(),
                name: name
                    .clone()
                    .unwrap_or_else(|| p.file_name().unwrap().to_string_lossy().to_string()),
                bundle_id: bid,
                running,
            });
        }
    }
    Ok(res)
}
