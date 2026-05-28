//! Typed progress events emitted to the frontend over the `progress` channel.

use serde::Serialize;
use tauri::{AppHandle, Emitter};

pub const EVENT_NAME: &str = "progress";

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProgressEvent {
    RefreshApps {
        progress: f32,
        message: String,
        finished: bool,
        error: Option<String>,
    },
    FindRelated {
        progress: f32,
        message: String,
        finished: bool,
        error: Option<String>,
    },
    Uninstall {
        progress: f32,
        message: String,
        finished: bool,
        error: Option<String>,
    },
}

pub fn emit(app: &AppHandle, event: ProgressEvent) {
    let _ = app.emit(EVENT_NAME, event);
}
