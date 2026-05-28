//! Find files and folders related to an app (Application Support, Caches,
//! Preferences, Containers, Logs, LaunchAgents, plus system receipts).

use home::home_dir;
use std::path::PathBuf;
use walkdir::WalkDir;

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
                for ent in WalkDir::new(&lib)
                    .max_depth(2)
                    .min_depth(1)
                    .into_iter()
                    .flatten()
                {
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

    res.sort();
    res.dedup();
    res.retain(|p| p.exists());
    res
}

fn common_paths_for_bundle_id(bid: &str) -> Vec<PathBuf> {
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
