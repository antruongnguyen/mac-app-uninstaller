//! Detect whether an app is currently running, and force-quit it.
//!
//! Mirrors the heuristics from the egui version: prefer exact-ish name matches,
//! then fall back to executable path and command-line. The same predicate is
//! used by [`is_app_running`] and [`force_quit_app`] so the running indicator
//! in the UI and the kill target stay in sync.

use std::{thread::sleep, time::Duration};
use sysinfo::{Pid, Process, System};

/// Lowercased keys derived from a (bundle_id, app_name) pair, computed once
/// and reused across all process matches.
struct MatchKeys {
    bid: Option<String>,
    bid_last: Option<String>,
    name: Option<String>,
}

impl MatchKeys {
    fn new(bundle_id: Option<&str>, app_name: Option<&str>) -> Self {
        Self {
            bid: bundle_id.map(|s| s.to_lowercase()),
            bid_last: bundle_id
                .and_then(|s| s.rsplit('.').next())
                .map(|s| s.to_lowercase()),
            name: app_name.map(|s| s.to_lowercase()),
        }
    }
}

fn matches_app_path(hay: &str, key: &str) -> bool {
    hay.contains(&format!("/{}.app/", key))
        || hay.contains(&format!("/{}.app", key))
        || hay.ends_with(&format!("/{}", key))
}

/// True if `proc_` looks like it belongs to the app described by `keys`.
fn process_matches(proc_: &Process, keys: &MatchKeys) -> bool {
    let name_l = proc_.name().to_string_lossy().to_lowercase();

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

    if let Some(exe_path) = proc_.exe() {
        let exe_s = exe_path.to_string_lossy().to_lowercase();
        if let Some(ref an) = keys.name {
            if matches_app_path(&exe_s, an) {
                return true;
            }
        }
        if let Some(ref bid) = keys.bid {
            if exe_s.contains(bid) {
                return true;
            }
        }
        if let Some(ref last) = keys.bid_last {
            if matches_app_path(&exe_s, last) {
                return true;
            }
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
        if cmdline_l.contains(bid) {
            return true;
        }
    }
    if let Some(ref an) = keys.name {
        if cmdline_l.contains(&format!("/{}.app", an))
            || cmdline_l.contains(&format!(" {} ", an))
            || cmdline_l.starts_with(&format!("{} ", an))
            || cmdline_l.ends_with(&format!(" {}", an))
        {
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

pub fn is_app_running(sys: &System, bundle_id: Option<&str>, app_name: Option<&str>) -> bool {
    let keys = MatchKeys::new(bundle_id, app_name);
    sys.processes()
        .values()
        .any(|proc_| process_matches(proc_, &keys))
}

pub fn is_app_running_simple(bundle_id: Option<&str>, app_name: Option<&str>) -> bool {
    let mut sys = System::new_all();
    sys.refresh_all();
    is_app_running(&sys, bundle_id, app_name)
}

/// Send SIGKILL to every process that matches the given app, then wait (with a
/// short timeout) for the kernel to actually reap them. Returning only after
/// the processes are gone guarantees that the next scan reports the app as
/// no longer running, so the UI's "Quit" button can hide and the sidebar's
/// running indicator clears in the same refresh cycle.
///
/// Returns the number of processes the kernel accepted SIGKILL for.
pub fn kill_app(bundle_id: Option<&str>, app_name: Option<&str>) -> u32 {
    let mut sys = System::new_all();
    sys.refresh_all();
    let keys = MatchKeys::new(bundle_id, app_name);

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
    // through sysinfo only on the next refresh.
    let deadline = std::time::Instant::now() + Duration::from_millis(2000);
    loop {
        sleep(Duration::from_millis(50));
        let mut probe = System::new_all();
        probe.refresh_all();
        let still_alive = probe
            .processes()
            .keys()
            .any(|pid| targets.contains(pid));
        if !still_alive || std::time::Instant::now() >= deadline {
            break;
        }
    }

    killed
}
