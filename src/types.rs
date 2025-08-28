//! Core data types shared across the application.

use egui::Color32;
use std::path::PathBuf;

/// Discovered application bundle with basic metadata.
#[derive(Clone, Debug)]
pub struct AppInfo {
    pub path: PathBuf,
    pub name: String,
    pub bundle_id: Option<String>,
    pub running: bool,
}

/// Kind of background task currently running.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum TaskKind {
    Idle,
    RefreshApps,
    RefreshRelated(usize), // index in apps
    Uninstall(usize),      // index in apps
}

/// Progress update message sent from background tasks to the UI.
#[derive(Clone, Debug)]
pub struct ProgressUpdate {
    pub kind: TaskKind,
    pub progress: f32,   // 0.0 ..= 1.0
    pub message: String, // human friendly
    pub finished: bool,  // whether task finished
    pub error: Option<String>,
}

pub struct StateColors {
    pub default: Color32,
    pub hover: Color32,
    pub selected: Option<Color32>, // None = use default theme color
}
