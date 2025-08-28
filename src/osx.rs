//! macOS-specific utilities.

/// Set the Dock icon from our .icns if possible (bundle or dev path).
#[cfg(target_os = "macos")]
#[allow(deprecated)]
pub fn try_set_dock_icon_from_icns() {
    use cocoa::appkit::{NSApp, NSApplication, NSImage};
    use cocoa::base::{id, nil};
    use cocoa::foundation::{NSAutoreleasePool, NSString};
    use std::path::PathBuf;

    unsafe {
        let _pool = NSAutoreleasePool::new(nil);
        // Ensure NSApplication exists
        let _app = NSApplication::sharedApplication(nil);

        // Candidate locations
        let mut candidates: Vec<PathBuf> = Vec::new();
        if let Ok(exe) = std::env::current_exe() {
            // .../My.app/Contents/MacOS/exe -> .../My.app/Contents/Resources/icon.icns
            if let Some(contents) = exe.parent().and_then(|p| p.parent()) {
                candidates.push(contents.join("Resources").join("icon.icns"));
            }
        }
        if let Ok(cwd) = std::env::current_dir() {
            candidates.push(cwd.join("resources").join("icon.icns"));
            candidates.push(cwd.join("../resources").join("icon.icns"));
            candidates.push(cwd.join("../../resources").join("icon.icns"));
        }

        for p in candidates {
            if p.exists() {
                let ns_path = NSString::alloc(nil).init_str(&p.to_string_lossy());
                let img: id = NSImage::alloc(nil).initByReferencingFile_(ns_path);
                if img != nil {
                    let app = NSApp();
                    app.setApplicationIconImage_(img);
                    break;
                }
            }
        }
    }
}

/// Try to open System Settings to the Full Disk Access pane (best-effort).
#[allow(dead_code)]
pub fn open_full_disk_access_settings() {
    // Open System Settings → Privacy & Security → Full Disk Access
    if cfg!(target_os = "macos") {
        // Newer macOS may support x-apple.systempreferences url
        let _ = std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles")
            .spawn();
    }
}
