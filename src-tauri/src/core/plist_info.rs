//! Read metadata from a `.app` bundle's `Info.plist`.

use anyhow::{Context, Result};
use plist::Value;
use std::path::Path;

#[derive(Default, Debug)]
pub struct PlistInfo {
    pub bundle_id: Option<String>,
    pub bundle_name: Option<String>,
    pub version: Option<String>,
    pub executable: Option<String>,
}

pub fn read_info_from_app(path: &Path) -> Result<PlistInfo> {
    let info = path.join("Contents").join("Info.plist");
    if !info.exists() {
        return Ok(PlistInfo::default());
    }
    let v = Value::from_file(&info).context("Read plist")?;
    let dict = v.as_dictionary();
    let read = |key: &str| {
        dict.and_then(|d| d.get(key))
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
    };
    Ok(PlistInfo {
        bundle_id: read("CFBundleIdentifier"),
        bundle_name: read("CFBundleName").or_else(|| read("CFBundleDisplayName")),
        version: read("CFBundleShortVersionString").or_else(|| read("CFBundleVersion")),
        executable: read("CFBundleExecutable"),
    })
}
