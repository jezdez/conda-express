use std::io::Cursor;
use std::str::FromStr;

use rattler_conda_types::{Platform, RepoDataRecord};
use rattler_lock::LockFile;
use wasm_bindgen::prelude::*;

/// Return a JSON array of all platform strings found in a lockfile.
#[wasm_bindgen]
pub fn get_platforms(lockfile_content: &str) -> String {
    let reader = Cursor::new(lockfile_content.as_bytes());
    let lockfile = match LockFile::from_reader(reader) {
        Ok(lf) => lf,
        Err(_) => return "[]".to_string(),
    };
    let default_env = match lockfile.default_environment() {
        Some(env) => env,
        None => return "[]".to_string(),
    };

    let platforms: Vec<String> = default_env
        .platforms()
        .map(|p| p.as_str().to_string())
        .collect();

    serde_json::to_string(&platforms).unwrap_or_else(|_| "[]".to_string())
}

/// Parse a lockfile and return package names as a JSON array for the given platform.
#[wasm_bindgen]
pub fn get_package_names(lockfile_content: &str, platform_str: &str) -> String {
    let platform = match Platform::from_str(platform_str) {
        Ok(p) => p,
        Err(_) => return format!("{{\"error\": \"unknown platform: {platform_str}\"}}"),
    };
    let names = packages_from_lockfile(lockfile_content, platform);
    serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string())
}

/// Parse a lockfile and return package download URLs as a JSON array for the given platform.
#[wasm_bindgen]
pub fn get_package_urls(lockfile_content: &str, platform_str: &str) -> String {
    let platform = match Platform::from_str(platform_str) {
        Ok(p) => p,
        Err(_) => return format!("{{\"error\": \"unknown platform: {platform_str}\"}}"),
    };
    let urls = package_urls_from_lockfile(lockfile_content, platform);
    serde_json::to_string(&urls).unwrap_or_else(|_| "[]".to_string())
}

#[wasm_bindgen]
pub fn cx_init() -> String {
    web_sys::console::log_1(&"cx-wasm-spike initialized".into());
    format!("cx-wasm-spike v{}", env!("CARGO_PKG_VERSION"))
}

fn packages_from_lockfile(lockfile_content: &str, platform: Platform) -> Vec<String> {
    let reader = Cursor::new(lockfile_content.as_bytes());
    let lockfile = LockFile::from_reader(reader).expect("failed to parse lockfile");
    let default_env = lockfile
        .default_environment()
        .expect("no default environment");

    let records: Vec<RepoDataRecord> = default_env
        .conda_repodata_records(platform)
        .ok()
        .flatten()
        .unwrap_or_default();

    let mut names: Vec<String> = records
        .into_iter()
        .map(|r| r.package_record.name.as_normalized().to_string())
        .collect();

    names.sort();
    names
}

fn package_urls_from_lockfile(lockfile_content: &str, platform: Platform) -> Vec<String> {
    let reader = Cursor::new(lockfile_content.as_bytes());
    let lockfile = LockFile::from_reader(reader).expect("failed to parse lockfile");
    let default_env = lockfile
        .default_environment()
        .expect("no default environment");

    let records: Vec<RepoDataRecord> = default_env
        .conda_repodata_records(platform)
        .ok()
        .flatten()
        .unwrap_or_default();

    records.into_iter().map(|r| r.url.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emscripten_platform_exists() {
        let p = Platform::EmscriptenWasm32;
        assert_eq!(p.as_str(), "emscripten-wasm32");
    }
}
