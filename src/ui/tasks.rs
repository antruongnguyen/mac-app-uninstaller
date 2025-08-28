//! Background tasks used by the UI for refreshing app lists, scanning related files,
//! and performing uninstalls without blocking the UI thread.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};

use crate::core::{find_app_bundles_progress, find_related_paths, is_app_running_simple, move_to_trash_or_remove};
use crate::types::{ProgressUpdate, TaskKind};

use super::GuiState;

/// Spawn a background task to refresh the list of installed applications.
pub fn spawn_refresh_apps(state_arc: Arc<Mutex<GuiState>>) {
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

/// Spawn a background task to compute related files for the selected app.
pub fn spawn_refresh_related_for_selected(state_arc: Arc<Mutex<GuiState>>, idx: usize) {
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

/// Spawn a background uninstall task for the selected app using the user-selected related paths.
pub fn spawn_uninstall_selected(state_arc: Arc<Mutex<GuiState>>, idx: usize) {
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
