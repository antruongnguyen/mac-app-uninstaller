//! Detect whether an app is currently running, and force-quit it.
//!
//! The authoritative match is **the process's executable path lives inside
//! the app bundle**. macOS guarantees that an app's processes are spawned
//! from `<bundle>/Contents/MacOS/`, so `proc.exe()?.starts_with(bundle_path)`
//! is unambiguous: it cannot collide with a CLI tool that happens to share
//! a name with the bundle (e.g. the `claude` CLI vs. `Claude.app`).
//!
//! When `proc.exe()` is unreadable (rare — typically kernel/system processes
//! the current user cannot inspect), we fall back to the same `Info.plist`
//! heuristics the egui version used: `CFBundleExecutable`/`CFBundleName`/
//! `CFBundleIdentifier` against `proc.name()` and the cmdline. The fallback
//! only fires when there is no exe path to disambiguate, so it cannot
//! produce false positives for processes whose path we *can* read.
//!
//! The same predicate is used by [`is_app_running`] and [`kill_app`] so the
//! running indicator in the UI and the kill target stay in sync.

use std::{
    path::{Path, PathBuf},
    thread::sleep,
    time::Duration,
};
use sysinfo::{
    Pid, Process, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System, UpdateKind,
};

/// Build a `System` populated only with the process fields `process_matches`
/// reads: `name` (always available), `exe`, and `cmd`. Skipping memory, CPU,
/// disks, networks, users, env, cwd, etc. cuts both the work the OS does to
/// fill the snapshot and the RAM the snapshot occupies.
fn process_only_snapshot() -> System {
    let kind = RefreshKind::nothing().with_processes(
        ProcessRefreshKind::nothing()
            .with_exe(UpdateKind::OnlyIfNotSet)
            .with_cmd(UpdateKind::OnlyIfNotSet),
    );
    System::new_with_specifics(kind)
}

/// Lowercased keys derived from a (bundle_path, bundle_id, app_name,
/// executable) tuple, computed once and reused across all process matches.
struct MatchKeys {
    /// The `.app` bundle path, e.g. `/Applications/Claude.app`. When set,
    /// this is the authoritative match: any process whose `exe()` starts
    /// with this path belongs to this app.
    bundle_path: Option<PathBuf>,
    bid: Option<String>,
    bid_last: Option<String>,
    name: Option<String>,
    /// `CFBundleExecutable` from `Info.plist`. Used only as a fallback when
    /// `proc.exe()` is unreadable — on its own it's unreliable for short or
    /// generic names (e.g. `Claude.app`'s executable is "Claude", which
    /// collides with the `claude` CLI).
    exe: Option<String>,
}

impl MatchKeys {
    fn new(
        bundle_path: Option<&Path>,
        bundle_id: Option<&str>,
        app_name: Option<&str>,
        executable: Option<&str>,
    ) -> Self {
        Self {
            bundle_path: bundle_path.map(|p| p.to_path_buf()),
            bid: bundle_id.map(|s| s.to_lowercase()),
            bid_last: bundle_id
                .and_then(|s| s.rsplit('.').next())
                .map(|s| s.to_lowercase()),
            name: app_name.map(|s| s.to_lowercase()),
            exe: executable.map(|s| s.to_lowercase()),
        }
    }
}

fn matches_app_path(hay: &str, key: &str) -> bool {
    hay.contains(&format!("/{}.app/", key))
        || hay.contains(&format!("/{}.app", key))
        || hay.ends_with(&format!("/{}", key))
}

/// True if `bid` appears in `hay` as a path component or LaunchServices-style
/// argument, not as a bare substring. This rejects matches like `log show`
/// commands that mention `com.apple.mail` in their text without actually
/// belonging to the Mail process.
fn matches_bundle_id_boundary(hay: &str, bid: &str) -> bool {
    hay.contains(&format!("/{}/", bid))
        || hay.contains(&format!("/{}.app", bid))
        || hay.contains(&format!("={}", bid))
        || hay.ends_with(&format!("/{}", bid))
}

/// True if `proc_` looks like it belongs to the app described by `keys`.
fn process_matches(proc_: &Process, keys: &MatchKeys) -> bool {
    // Authoritative: the process's executable path is inside the bundle.
    // When we have both a bundle path and a readable exe path, this is the
    // *only* check we run — a non-match here is a real non-match, no
    // string-heuristic should override it.
    if let Some(exe_path) = proc_.exe() {
        if let Some(ref bundle) = keys.bundle_path {
            return exe_path.starts_with(bundle);
        }
        // No bundle path provided (on-demand call without it). Fall back to
        // path-shaped checks against the exe path.
        let exe_s = exe_path.to_string_lossy().to_lowercase();
        if let Some(ref an) = keys.name {
            if matches_app_path(&exe_s, an) {
                return true;
            }
        }
        if let Some(ref bid) = keys.bid {
            if matches_bundle_id_boundary(&exe_s, bid) {
                return true;
            }
        }
        if let Some(ref last) = keys.bid_last {
            if matches_app_path(&exe_s, last) {
                return true;
            }
        }
        return false;
    }

    // Fallback: `proc.exe()` was unreadable. Use process name and cmdline.
    // This is rare and mostly affects kernel/system processes the user
    // doesn't care about, but we keep it so detection still works for
    // unusual cases.
    let name_l = proc_.name().to_string_lossy().to_lowercase();

    if let Some(ref exe) = keys.exe {
        if name_l == *exe {
            return true;
        }
    }
    if let Some(ref an) = keys.name {
        if name_l == *an || name_l == format!("{}.app", an) {
            return true;
        }
    }
    if let Some(ref last) = keys.bid_last {
        if name_l == *last || name_l == format!("{}.app", last) {
            return true;
        }
    }

    let cmdline_l = proc_
        .cmd()
        .iter()
        .map(|s| s.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();

    if let Some(ref bid) = keys.bid {
        if matches_bundle_id_boundary(&cmdline_l, bid) {
            return true;
        }
    }
    if let Some(ref an) = keys.name {
        if cmdline_l.contains(&format!("/{}.app", an)) {
            return true;
        }
    }
    if let Some(ref last) = keys.bid_last {
        if cmdline_l.contains(&format!("/{}.app", last)) {
            return true;
        }
    }
    false
}

pub fn is_app_running(
    sys: &System,
    bundle_path: Option<&Path>,
    bundle_id: Option<&str>,
    app_name: Option<&str>,
    executable: Option<&str>,
) -> bool {
    let keys = MatchKeys::new(bundle_path, bundle_id, app_name, executable);
    sys.processes()
        .values()
        .any(|proc_| process_matches(proc_, &keys))
}

pub fn is_app_running_simple(
    bundle_path: Option<&Path>,
    bundle_id: Option<&str>,
    app_name: Option<&str>,
) -> bool {
    let sys = process_only_snapshot();
    is_app_running(&sys, bundle_path, bundle_id, app_name, None)
}

/// Send SIGKILL to every process that matches the given app, then wait (with a
/// short timeout) for the kernel to actually reap them. Returning only after
/// the processes are gone guarantees that the next scan reports the app as
/// no longer running, so the UI's "Quit" button can hide and the sidebar's
/// running indicator clears in the same refresh cycle.
///
/// Returns the number of processes the kernel accepted SIGKILL for.
pub fn kill_app(
    bundle_path: Option<&Path>,
    bundle_id: Option<&str>,
    app_name: Option<&str>,
) -> u32 {
    let mut sys = process_only_snapshot();
    let keys = MatchKeys::new(bundle_path, bundle_id, app_name, None);

    let mut targets: Vec<Pid> = Vec::new();
    let mut killed: u32 = 0;
    for proc_ in sys.processes().values() {
        if !process_matches(proc_, &keys) {
            continue;
        }
        targets.push(proc_.pid());
        if proc_.kill() {
            killed += 1;
        }
    }

    if killed == 0 {
        return 0;
    }

    // Poll until the targeted PIDs disappear from the snapshot, or we hit the
    // budget. SIGKILL is honoured by the kernel quickly but is observable
    // through sysinfo only on the next refresh. We reuse `sys` and refresh
    // with `ProcessRefreshKind::nothing()` — we only need the live PID set,
    // not any per-process detail — so each iteration is cheap.
    let deadline = std::time::Instant::now() + Duration::from_millis(2000);
    loop {
        sleep(Duration::from_millis(50));
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing(),
        );
        let still_alive = sys.processes().keys().any(|pid| targets.contains(pid));
        if !still_alive || std::time::Instant::now() >= deadline {
            break;
        }
    }

    killed
}
