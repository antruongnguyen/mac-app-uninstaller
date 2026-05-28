//! App Uninstaller backend (Tauri).
//!
//! `core` holds the pure filesystem/process logic; `commands` exposes it
//! to the React frontend through Tauri commands. `progress` defines the
//! single typed event channel used to stream task progress to the UI.

pub mod commands;
pub mod core;
pub mod models;
pub mod progress;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_apps,
            commands::find_related,
            commands::is_app_running,
            commands::kill_app,
            commands::get_app_size,
            commands::uninstall,
            commands::reveal_in_finder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
