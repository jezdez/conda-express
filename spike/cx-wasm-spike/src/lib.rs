mod bootstrap;
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

/// Fetch bytes from a URL using the browser Fetch API with a 5-minute timeout.
pub(crate) async fn fetch_bytes(url: &str) -> Result<Vec<u8>, String> {
    use js_sys::{ArrayBuffer, Uint8Array};
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{AbortController, Request, RequestInit, RequestMode, Response};

    let controller =
        AbortController::new().map_err(|e| format!("AbortController error: {e:?}"))?;
    let signal = controller.signal();

    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);
    opts.set_signal(Some(&signal));

    let request =
        Request::new_with_str_and_init(url, &opts).map_err(|e| format!("request error: {e:?}"))?;

    let window = web_sys::window().ok_or("no global window")?;

    let timeout_id = window
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            &wasm_bindgen::closure::Closure::<dyn Fn()>::new({
                let controller = controller.clone();
                move || controller.abort()
            })
            .into_js_value()
            .unchecked_into(),
            300_000, // 5-minute timeout for large packages
        )
        .map_err(|e| format!("setTimeout error: {e:?}"))?;

    let result = async {
        let resp_val = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|e| format!("fetch error (timeout or CORS?): {e:?}"))?;
        let resp: Response = resp_val.dyn_into().map_err(|_| "response cast failed")?;

        if !resp.ok() {
            return Err(format!("HTTP {}", resp.status()));
        }

        let buf_promise = resp
            .array_buffer()
            .map_err(|e| format!("array_buffer error: {e:?}"))?;
        let buf_val = JsFuture::from(buf_promise)
            .await
            .map_err(|e| format!("buffer read error: {e:?}"))?;
        let buf: ArrayBuffer = buf_val.dyn_into().map_err(|_| "buffer cast failed")?;
        let array = Uint8Array::new(&buf);

        Ok(array.to_vec())
    }
    .await;

    window.clear_timeout_with_handle(timeout_id);
    result
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

    let contents = if url.ends_with(".tar.bz2") {
        extract::extract_tar_bz2(&bytes)?
    } else {
        extract::extract_conda(&bytes)?
    };

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

/// Bootstrap all packages from a lockfile for the given platform.
///
/// Downloads and extracts every .conda package. Returns JSON with the full file tree.
/// `progress` is an optional JS callback: `progress(current, total, packageName)`.
#[wasm_bindgen]
pub async fn cx_bootstrap(
    lockfile_content: String,
    platform: String,
    progress: Option<js_sys::Function>,
) -> String {
    match bootstrap::bootstrap_impl(&lockfile_content, &platform, progress.as_ref()).await {
        Ok(result) => serde_json::to_string(&result).unwrap_or_else(|e| {
            serde_json::json!({"error": format!("json error: {e}")}).to_string()
        }),
        Err(e) => serde_json::json!({"error": e}).to_string(),
    }
}

/// Get a summary of what bootstrap would do: package count, names, total download size.
#[wasm_bindgen]
pub fn cx_bootstrap_plan(lockfile_content: &str, platform_str: &str) -> String {
    let platform = match Platform::from_str(platform_str) {
        Ok(p) => p,
        Err(_) => {
            return serde_json::json!({"error": format!("unknown platform: {platform_str}")})
                .to_string()
        }
    };

    let reader = Cursor::new(lockfile_content.as_bytes());
    let lockfile = match LockFile::from_reader(reader) {
        Ok(lf) => lf,
        Err(e) => return serde_json::json!({"error": format!("parse error: {e}")}).to_string(),
    };

    let env = match lockfile.default_environment() {
        Some(e) => e,
        None => return serde_json::json!({"error": "no default environment"}).to_string(),
    };

    let records: Vec<RepoDataRecord> = env
        .conda_repodata_records(platform)
        .ok()
        .flatten()
        .unwrap_or_default();

    let packages: Vec<serde_json::Value> = records
        .iter()
        .map(|r| {
            serde_json::json!({
                "name": r.package_record.name.as_normalized(),
                "version": r.package_record.version.to_string(),
                "url": r.url.to_string(),
                "size": r.package_record.size,
            })
        })
        .collect();

    let total_size: u64 = records
        .iter()
        .filter_map(|r| r.package_record.size)
        .sum();

    serde_json::json!({
        "platform": platform_str,
        "package_count": packages.len(),
        "total_download_size": total_size,
        "packages": packages,
    })
    .to_string()
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
