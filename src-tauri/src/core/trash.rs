//! Trash-or-remove helpers and "is this path system-protected?" classifier.

use anyhow::{Context, Result};
use std::{fs, path::Path};

pub fn move_to_trash_or_remove(path: &Path) -> Result<()> {
    match trash::delete(path) {
        Ok(_) => Ok(()),
        Err(_trash_err) => {
            if path.is_dir() {
                fs::remove_dir_all(path)
                    .with_context(|| format!("Failed to remove dir {}", path.display()))?;
            } else if path.is_file() {
                fs::remove_file(path)
                    .with_context(|| format!("Failed to remove file {}", path.display()))?;
            } else {
                return Err(anyhow::anyhow!("Unknown path type: {}", path.display()));
            }
            Ok(())
        }
    }
}

pub fn reveal_in_finder(path: &Path) -> Result<()> {
    let p = path
        .canonicalize()
        .with_context(|| format!("Canonicalize {}", path.display()))?;
    std::process::Command::new("open")
        .arg("-R")
        .arg(p)
        .status()
        .context("Failed to run `open -R`")?;
    Ok(())
}

/// Heuristic: paths under system-managed locations require admin auth.
pub fn is_protected_path(p: &Path) -> bool {
    let s = p.to_string_lossy();
    s.starts_with("/Library")
        || s.starts_with("/System")
        || s.starts_with("/System/Volumes")
        || s.starts_with("/System/Volumes/Data")
        || s.starts_with("/Applications")
        || s.starts_with("/private")
        || s.starts_with("/usr")
        || s.starts_with("/bin")
        || s.starts_with("/sbin")
        || s.starts_with("/var")
        || s.starts_with("/opt")
        || s.starts_with("/etc")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn protected_paths_are_classified() {
        assert!(is_protected_path(&PathBuf::from("/Library/Foo")));
        assert!(is_protected_path(&PathBuf::from(
            "/private/var/db/receipts/x"
        )));
        assert!(is_protected_path(&PathBuf::from("/Applications/Bar.app")));
    }

    #[test]
    fn user_paths_are_unprotected() {
        assert!(!is_protected_path(&PathBuf::from(
            "/Users/alice/Library/Caches/x"
        )));
    }
}
