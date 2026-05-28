//! Tauri command handlers.
//!
//! Each command runs the corresponding `core` function on a blocking task
//! and emits typed `progress` events while it runs.

use std::path::PathBuf;
use tauri::AppHandle;

use crate::core;
use crate::models::{AppInfo, UninstallFailure, UninstallReport};
use crate::progress::{self, ProgressEvent};

#[tauri::command]
pub async fn list_apps(app: AppHandle) -> Result<Vec<AppInfo>, String> {
    progress::emit(
        &app,
        ProgressEvent::RefreshApps {
            progress: 0.0,
            message: "Scanning /Applications and ~/Applications...".into(),
            finished: false,
            error: None,
        },
    );

    let app_for_progress = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        core::scan_apps_with_progress(|p, msg| {
            progress::emit(
                &app_for_progress,
                ProgressEvent::RefreshApps {
                    progress: p,
                    message: msg.to_string(),
                    finished: false,
                    error: None,
                },
            );
        })
    })
    .await
    .map_err(|e| e.to_string())?;

    match result {
        Ok(apps) => {
            progress::emit(
                &app,
                ProgressEvent::RefreshApps {
                    progress: 1.0,
                    message: "Done.".into(),
                    finished: true,
                    error: None,
                },
            );
            Ok(apps)
        }
        Err(e) => {
            let msg = format!("{e:?}");
            progress::emit(
                &app,
                ProgressEvent::RefreshApps {
                    progress: 1.0,
                    message: "Failed.".into(),
                    finished: true,
                    error: Some(msg.clone()),
                },
            );
            Err(msg)
        }
    }
}

#[tauri::command]
pub async fn find_related(
    app: AppHandle,
    bundle_id: Option<String>,
    app_name: String,
) -> Result<Vec<PathBuf>, String> {
    progress::emit(
        &app,
        ProgressEvent::FindRelated {
            progress: 0.0,
            message: format!("Finding related files for {app_name}..."),
            finished: false,
            error: None,
        },
    );

    let result = tauri::async_runtime::spawn_blocking(move || {
        core::find_related_paths(bundle_id.as_deref(), Some(&app_name))
    })
    .await
    .map_err(|e| e.to_string())?;

    progress::emit(
        &app,
        ProgressEvent::FindRelated {
            progress: 1.0,
            message: format!("Found {} related item(s).", result.len()),
            finished: true,
            error: None,
        },
    );

    Ok(result)
}

#[tauri::command]
pub async fn is_app_running(
    app_path: Option<PathBuf>,
    bundle_id: Option<String>,
    app_name: Option<String>,
) -> Result<bool, String> {
    let result = tauri::async_runtime::spawn_blocking(move || {
        core::is_app_running_simple(
            app_path.as_deref(),
            bundle_id.as_deref(),
            app_name.as_deref(),
        )
    })
    .await
    .map_err(|e| e.to_string())?;
    Ok(result)
}

/// Send SIGKILL to every process that matches the given app. Returns the
/// number of processes that were killed.
#[tauri::command]
pub async fn kill_app(
    app_path: Option<PathBuf>,
    bundle_id: Option<String>,
    app_name: Option<String>,
) -> Result<u32, String> {
    let killed = tauri::async_runtime::spawn_blocking(move || {
        core::kill_app(
            app_path.as_deref(),
            bundle_id.as_deref(),
            app_name.as_deref(),
        )
    })
    .await
    .map_err(|e| e.to_string())?;
    Ok(killed)
}

/// Recursively sum the size of every file under `path`. Expensive for large
/// bundles, so this runs on demand and is not part of `list_apps`.
#[tauri::command]
pub async fn get_app_size(path: PathBuf) -> Result<Option<u64>, String> {
    tauri::async_runtime::spawn_blocking(move || core::compute_size(&path))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn uninstall(
    app: AppHandle,
    app_path: PathBuf,
    app_name: String,
    bundle_id: Option<String>,
    related_paths: Vec<PathBuf>,
) -> Result<UninstallReport, String> {
    let app_for_task = app.clone();

    tauri::async_runtime::spawn_blocking(move || {
        run_uninstall(&app_for_task, app_path, app_name, bundle_id, related_paths)
    })
    .await
    .map_err(|e| e.to_string())?
}

fn run_uninstall(
    app: &AppHandle,
    app_path: PathBuf,
    app_name: String,
    bundle_id: Option<String>,
    related_paths: Vec<PathBuf>,
) -> Result<UninstallReport, String> {
    let emit_progress = |progress: f32, message: String, finished: bool, error: Option<String>| {
        progress::emit(
            app,
            ProgressEvent::Uninstall {
                progress,
                message,
                finished,
                error,
            },
        );
    };

    emit_progress(
        0.0,
        format!("Starting uninstall of {app_name}..."),
        false,
        None,
    );

    if core::is_app_running_simple(Some(&app_path), bundle_id.as_deref(), Some(&app_name)) {
        let err = "App is running. Abort uninstall.".to_string();
        emit_progress(0.0, err.clone(), true, Some(err.clone()));
        return Err(err);
    }

    let total_steps = 1 + related_paths.len();
    let mut step = 0usize;
    let mut report = UninstallReport {
        app_path: app_path.clone(),
        removed: Vec::new(),
        failed: Vec::new(),
        aborted: false,
    };

    // Step 1: bundle itself.
    if let Err(e) = core::move_to_trash_or_remove(&app_path) {
        let msg = format!("Failed to remove bundle: {e:?}");
        emit_progress(0.0, msg.clone(), true, Some(msg.clone()));
        report.aborted = true;
        report.failed.push(UninstallFailure {
            path: app_path.clone(),
            error: msg.clone(),
        });
        return Err(msg);
    }
    step += 1;
    report.removed.push(app_path.clone());
    emit_progress(
        step as f32 / total_steps as f32,
        format!("Moved {} to Trash", app_path.display()),
        false,
        None,
    );

    // Step 2: split into protected vs unprotected.
    let (protected, unprotected): (Vec<PathBuf>, Vec<PathBuf>) = related_paths
        .into_iter()
        .partition(|p| core::is_protected_path(p));

    // Phase 2a: protected — abort on first failure.
    for p in protected {
        match core::move_to_trash_or_remove(&p) {
            Ok(()) => {
                step += 1;
                report.removed.push(p.clone());
                emit_progress(
                    step as f32 / total_steps as f32,
                    format!("Removed {}", p.display()),
                    false,
                    None,
                );
            }
            Err(e) => {
                let msg = format!("Aborting on {}: {:?}", p.display(), e);
                report.aborted = true;
                report.failed.push(UninstallFailure {
                    path: p,
                    error: format!("{e:?}"),
                });
                emit_progress(
                    step as f32 / total_steps as f32,
                    msg.clone(),
                    true,
                    Some(msg.clone()),
                );
                return Err(msg);
            }
        }
    }

    // Phase 2b: unprotected — continue past per-item errors.
    for p in unprotected {
        match core::move_to_trash_or_remove(&p) {
            Ok(()) => {
                step += 1;
                report.removed.push(p.clone());
                emit_progress(
                    step as f32 / total_steps as f32,
                    format!("Removed {}", p.display()),
                    false,
                    None,
                );
            }
            Err(e) => {
                let msg = format!("Failed to remove {}: {:?}", p.display(), e);
                report.failed.push(UninstallFailure {
                    path: p,
                    error: format!("{e:?}"),
                });
                emit_progress(
                    step as f32 / total_steps as f32,
                    msg,
                    false,
                    Some(format!("{e:?}")),
                );
            }
        }
    }

    emit_progress(1.0, "Uninstall complete".into(), true, None);
    Ok(report)
}

#[tauri::command]
pub async fn reveal_in_finder(path: PathBuf) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || core::reveal_in_finder(&path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| format!("{e:?}"))
}
