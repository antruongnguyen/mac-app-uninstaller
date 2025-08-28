//! Egui-based UI for the App Uninstaller.
//!
//! This module defines the application state, the eframe App implementation,
//! and wires UI actions to background tasks defined in ui::tasks.

use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

use eframe::{egui, App};

use crate::style::set_appkit_style;
use crate::types::{AppInfo, ProgressUpdate, TaskKind};

pub mod panels;

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
            tasks::spawn_refresh_apps(st.clone());
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

        panels::top::show(ctx);

        // Bottom status panel: compact status bar. Prevent expansion and resizing.
        panels::bottom::show(ctx, &self.state);

        // Sidebar
        panels::side::show(ctx, &self.state);

        // Main panel
        panels::central::show(ctx, &self.state);

        // request repaint for smooth progress updates
        ctx.request_repaint_after(Duration::from_millis(16));
    }
}

mod list;
pub mod tasks;
