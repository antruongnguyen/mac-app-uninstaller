//! Pure business logic. No Tauri / UI dependencies.

pub mod apps;
pub mod plist_info;
pub mod related;
pub mod running;
pub mod trash;

pub use apps::{candidate_app_dirs, compute_size, scan_apps, scan_apps_with_progress};
pub use plist_info::read_info_from_app;
pub use related::find_related_paths;
pub use running::{is_app_running, is_app_running_simple, kill_app};
pub use trash::{is_protected_path, move_to_trash_or_remove, reveal_in_finder};
