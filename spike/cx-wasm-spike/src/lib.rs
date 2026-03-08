mod extract;

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

/// Fetch bytes from a URL using the browser Fetch API.
async fn fetch_bytes(url: &str) -> Result<Vec<u8>, String> {
    use js_sys::{ArrayBuffer, Uint8Array};
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, RequestMode, Response};

    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    let request =
        Request::new_with_str_and_init(url, &opts).map_err(|e| format!("request error: {e:?}"))?;

    let window = web_sys::window().ok_or("no global window")?;
    let resp_val = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("fetch error: {e:?}"))?;
    let resp: Response = resp_val.dyn_into().map_err(|_| "response cast failed")?;

    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }

    let buf_promise = resp
        .array_buffer()
        .map_err(|e| format!("array_buffer error: {e:?}"))?;
    let buf_val = JsFuture::from(buf_promise)
        .await
        .map_err(|e| format!("buffer error: {e:?}"))?;
    let buf: ArrayBuffer = buf_val.dyn_into().map_err(|_| "buffer cast failed")?;
    let array = Uint8Array::new(&buf);

    Ok(array.to_vec())
}

/// Download a .conda package from the given URL and return a JSON listing of its contents.
#[wasm_bindgen]
pub async fn download_and_list_package(url: String) -> String {
    match download_and_list_impl(&url).await {
        Ok(json) => json,
        Err(e) => serde_json::json!({"error": e}).to_string(),
    }
}

async fn download_and_list_impl(url: &str) -> Result<String, String> {
    web_sys::console::log_1(&format!("Downloading {url}...").into());

    let bytes = fetch_bytes(url).await?;
    let size_kb = bytes.len() / 1024;
    web_sys::console::log_1(&format!("Downloaded {size_kb} KB, extracting...").into());

    let contents = extract::extract_conda(&bytes)?;

    web_sys::console::log_1(
        &format!(
            "Extracted {} info files + {} pkg files ({} KB total)",
            contents.info_files.len(),
            contents.pkg_files.len(),
            contents.total_size / 1024
        )
        .into(),
    );

    serde_json::to_string(&contents).map_err(|e| format!("json error: {e}"))
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
