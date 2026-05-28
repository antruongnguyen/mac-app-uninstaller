//! Serde DTOs shared with the frontend.
//!
//! These types are mirrored verbatim in `src/types/models.ts`.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub path: PathBuf,
    pub name: String,
    pub bundle_id: Option<String>,
    pub version: Option<String>,
    pub executable: Option<String>,
    /// Last-modified time as a Unix timestamp (seconds). `None` if unreadable.
    pub modified_at: Option<i64>,
    pub running: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UninstallReport {
    pub app_path: PathBuf,
    pub removed: Vec<PathBuf>,
    pub failed: Vec<UninstallFailure>,
    pub aborted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UninstallFailure {
    pub path: PathBuf,
    pub error: String,
}
