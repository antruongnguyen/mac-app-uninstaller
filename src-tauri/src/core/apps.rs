//! Scan installed application bundles.

use anyhow::{Context, Result};
use home::home_dir;
use std::{fs, path::PathBuf};
use sysinfo::{ProcessRefreshKind, RefreshKind, System, UpdateKind};
use walkdir::WalkDir;

use super::{plist_info::read_info_from_app, running::is_app_running};
use crate::models::AppInfo;

pub fn candidate_app_dirs() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/Applications"),
        home_dir()
            .map(|h| h.join("Applications"))
            .unwrap_or_default(),
    ]
}

pub fn scan_apps() -> Result<Vec<AppInfo>> {
    scan_apps_with_progress(|_, _| {})
}

pub fn scan_apps_with_progress<F>(mut on_progress: F) -> Result<Vec<AppInfo>>
where
    F: FnMut(f32, &str),
{
    let candidates = candidate_app_dirs();
    let total = candidates.len().max(1);

    // Collect process metadata once for the whole scan. Restrict the snapshot
    // to the fields `is_app_running` reads (exe path + cmdline) so we don't
    // pull in CPU/memory/disks/networks/users for every process on the box.
    let kind = RefreshKind::nothing().with_processes(
        ProcessRefreshKind::nothing()
            .with_exe(UpdateKind::OnlyIfNotSet)
            .with_cmd(UpdateKind::OnlyIfNotSet),
    );
    let sys = System::new_with_specifics(kind);

    let mut res = Vec::new();
    for (idx, dir) in candidates.into_iter().enumerate() {
        on_progress(
            (idx as f32) / (total as f32),
            &format!("Scanning {}", dir.display()),
        );

        if dir.exists() && dir.is_dir() {
            let mut v = scan_one_dir(&sys, &dir)?;
            res.append(&mut v);
        }
    }

    res.sort_by(|a, b| a.name.cmp(&b.name));
    on_progress(0.95, "Finalizing");
    Ok(res)
}

fn scan_one_dir(sys: &System, dir: &std::path::Path) -> Result<Vec<AppInfo>> {
    let mut res = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("Read dir {}", dir.display()))? {
        let e = entry?;
        let p = e.path();
        if p.extension().and_then(|s| s.to_str()) == Some("app") {
            let info = read_info_from_app(&p).unwrap_or_default();
            let running = is_app_running(
                sys,
                Some(&p),
                info.bundle_id.as_deref(),
                info.bundle_name.as_deref(),
                info.executable.as_deref(),
            );
            let modified_at = e
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64);
            res.push(AppInfo {
                name: info
                    .bundle_name
                    .clone()
                    .unwrap_or_else(|| p.file_name().unwrap().to_string_lossy().to_string()),
                bundle_id: info.bundle_id,
                version: info.version,
                executable: info.executable,
                modified_at,
                running,
                path: p,
            });
        }
    }
    Ok(res)
}

/// Sum of regular-file sizes under `path`. Best-effort: skips entries that
/// can't be read (symlinks pointing into protected dirs, etc.). Expensive for
/// large bundles (e.g. Xcode), so it's exposed as its own command and called
/// lazily by the frontend when an app is selected — never during the scan.
pub fn compute_size(path: &std::path::Path) -> Option<u64> {
    let mut total: u64 = 0;
    for entry in WalkDir::new(path).follow_links(false) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if let Ok(meta) = entry.metadata() {
            if meta.is_file() {
                total = total.saturating_add(meta.len());
            }
        }
    }
    if total == 0 { None } else { Some(total) }
}
