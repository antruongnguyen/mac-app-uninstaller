use anyhow::{Context, Result};
use home::home_dir;
use plist::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
    time::Duration,
};
use sysinfo::System;
use walkdir::WalkDir;

use crate::types::{AppInfo, ProgressUpdate, TaskKind};

// Core/business logic: scanning apps, reading bundle info, checking running processes,
// finding related paths, and performing file operations.

/// Scan candidate application directories and send coarse progress updates via tx.
pub fn find_app_bundles_progress(tx: &mpsc::Sender<ProgressUpdate>) -> Result<Vec<AppInfo>> {
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

/// Read CFBundleIdentifier and CFBundleName from Contents/Info.plist
pub fn read_info_from_app(path: &Path) -> Result<(Option<String>, Option<String>)> {
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

/// Minimal running check using sysinfo snapshot with stronger heuristics and less duplication
pub fn is_app_running(sys: &System, bundle_id: Option<&str>, app_name: Option<&str>) -> bool {
    // Precompute lowercase inputs once
    let bid_l = bundle_id.map(|s| s.to_lowercase());
    let bid_last_l = bundle_id
        .and_then(|s| s.rsplit('.').next())
        .map(|s| s.to_lowercase());
    let an_l = app_name.map(|s| s.to_lowercase());

    // Small helper to check common app-bundle path patterns inside a string
    let matches_app_path = |hay: &str, key: &str| {
        hay.contains(&format!("/{}.app/", key))
            || hay.contains(&format!("/{}.app", key))
            || hay.ends_with(&format!("/{}", key))
    };

    for (_pid, proc_) in sys.processes() {
        let name_l = proc_.name().to_string_lossy().to_lowercase();

        // 1) Prefer exact-ish name matches to avoid false positives
        if let Some(ref an) = an_l {
            if name_l == *an || name_l == format!("{}.app", an) {
                return true;
            }
        }
        if let Some(ref last) = bid_last_l {
            if name_l == *last || name_l == format!("{}.app", last) {
                return true;
            }
        }

        // 2) Check executable path, which often resides inside "AppName.app"
        if let Some(exe_path) = proc_.exe() {
            let exe_s = exe_path.to_string_lossy().to_lowercase();
            if let Some(ref an) = an_l {
                if matches_app_path(&exe_s, an) {
                    return true;
                }
            }
            if let Some(ref bid) = bid_l {
                if exe_s.contains(bid) {
                    return true;
                }
            }
            if let Some(ref last) = bid_last_l {
                if matches_app_path(&exe_s, last) {
                    return true;
                }
            }
        }

        // 3) Check command line for precise fragments
        // Build lazily and only once per process
        let cmdline_l = proc_
            .cmd()
            .iter()
            .map(|s| s.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();

        if let Some(ref bid) = bid_l {
            if cmdline_l.contains(bid) {
                return true;
            }
        }
        if let Some(ref an) = an_l {
            // Prefer to match either an ".app" path or exact token
            if cmdline_l.contains(&format!("/{}.app", an))
                || cmdline_l.contains(&format!(" {} ", an))
                || cmdline_l.starts_with(&format!("{} ", an))
                || cmdline_l.ends_with(&format!(" {}", an))
            {
                return true;
            }
        }
        if let Some(ref last) = bid_last_l {
            if cmdline_l.contains(&format!("/{}.app", last)) {
                return true;
            }
        }
    }
    false
}

/// Simpler runtime check (fresh System inside)
pub fn is_app_running_simple(bundle_id: Option<&str>, app_name: Option<&str>) -> bool {
    let mut sys = System::new_all();
    sys.refresh_all();
    is_app_running(&sys, bundle_id, app_name)
}

/// Find related paths (by bundle id and app name)
pub fn find_related_paths(bundle_id: Option<&str>, app_name: Option<&str>) -> Vec<PathBuf> {
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

pub fn common_paths_for_bundle_id(bid: &str) -> Vec<PathBuf> {
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

/// Move to trash (preferred) else remove directly
pub fn move_to_trash_or_remove(path: &Path) -> Result<()> {
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

/// Reveal path in Finder (macOS)
pub fn reveal_in_finder(path: &Path) -> Result<()> {
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

/// Return the list of application directories to scan (system and user Applications).
pub fn candidate_app_dirs() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/Applications"),
        home_dir()
            .map(|h| h.join("Applications"))
            .unwrap_or_default(),
    ]
}

/// Scan a directory for .app bundles and extract AppInfo items.
pub fn scan_apps_in_dir(sys: &System, dir: &Path) -> Result<Vec<AppInfo>> {
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
