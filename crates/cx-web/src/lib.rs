mod bootstrap;
mod error;
mod extract;

use std::str::FromStr;

use rattler_conda_types::Platform;
use rattler_lock::LockFile;
use serde::Serialize;
use wasm_bindgen::prelude::*;

use error::CxWebError;

#[wasm_bindgen(typescript_custom_section)]
const TS_TYPES: &str = r#"
export interface ExtractedFile {
    path: string;
    size: number;
}

export interface CondaPackageContents {
    info_files: ExtractedFile[];
    pkg_files: ExtractedFile[];
    total_size: number;
}

export interface PackageResult {
    name: string;
    version: string;
    url: string;
    info_files: ExtractedFile[];
    pkg_files: ExtractedFile[];
    total_size: number;
}

export interface BootstrapResult {
    platform: string;
    packages: PackageResult[];
    total_packages: number;
    total_files: number;
    total_size: number;
    errors: string[];
}

export interface PackagePlanEntry {
    name: string;
    version: string;
    url: string;
    size: number | null;
}

export interface BootstrapPlan {
    platform: string;
    package_count: number;
    total_download_size: number;
    packages: PackagePlanEntry[];
}
"#;

fn to_js<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value).map_err(|e| CxWebError::SerializeFailed(e.to_string()).into())
}

fn parse_platform(platform_str: &str) -> Result<Platform, JsValue> {
    Platform::from_str(platform_str)
        .map_err(|_| CxWebError::PlatformUnknown(platform_str.to_string()).into())
}

fn parse_lockfile(lockfile_content: &str) -> Result<LockFile, JsValue> {
    let reader = std::io::Cursor::new(lockfile_content.as_bytes());
    LockFile::from_reader(reader).map_err(|e| CxWebError::LockfileParse(e.to_string()).into())
}

/// Return a JS array of all platform strings found in a lockfile.
#[wasm_bindgen]
pub fn get_platforms(lockfile_content: &str) -> Result<JsValue, JsValue> {
    let lockfile = parse_lockfile(lockfile_content)?;
    let env = lockfile
        .default_environment()
        .ok_or::<JsValue>(CxWebError::NoDefaultEnvironment.into())?;

    let platforms: Vec<String> = env.platforms().map(|p| p.as_str().to_string()).collect();
    to_js(&platforms)
}

/// Parse a lockfile and return package names as a JS array for the given platform.
#[wasm_bindgen]
pub fn get_package_names(
    lockfile_content: &str,
    platform_str: &str,
) -> Result<JsValue, JsValue> {
    let platform = parse_platform(platform_str)?;
    let records = bootstrap::get_records(lockfile_content, platform)?;

    let mut names: Vec<String> = records
        .into_iter()
        .map(|r| r.package_record.name.as_normalized().to_string())
        .collect();
    names.sort();
    to_js(&names)
}

/// Parse a lockfile and return package download URLs as a JS array for the given platform.
#[wasm_bindgen]
pub fn get_package_urls(
    lockfile_content: &str,
    platform_str: &str,
) -> Result<JsValue, JsValue> {
    let platform = parse_platform(platform_str)?;
    let records = bootstrap::get_records(lockfile_content, platform)?;

    let urls: Vec<String> = records.into_iter().map(|r| r.url.to_string()).collect();
    to_js(&urls)
}

#[wasm_bindgen]
pub fn cx_init() -> String {
    web_sys::console::log_1(&"cx-web initialized".into());
    format!("cx-web v{}", env!("CARGO_PKG_VERSION"))
}

/// Fetch bytes from a URL using the browser Fetch API with a 5-minute timeout.
pub(crate) async fn fetch_bytes(url: &str) -> Result<Vec<u8>, CxWebError> {
    use js_sys::{ArrayBuffer, Uint8Array};
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{AbortController, Request, RequestInit, RequestMode, Response};

    let controller = AbortController::new()
        .map_err(|e| CxWebError::FetchFailed(format!("AbortController error: {e:?}")))?;
    let signal = controller.signal();

    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);
    opts.set_signal(Some(&signal));

    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|e| CxWebError::FetchFailed(format!("request error: {e:?}")))?;

    let window = web_sys::window().ok_or(CxWebError::FetchFailed("no global window".into()))?;

    // into_js_value() leaks the Closure (calls forget() internally).
    // Acceptable here: one small alloc per fetch, freed when page unloads.
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
        .map_err(|e| CxWebError::FetchFailed(format!("setTimeout error: {e:?}")))?;

    let result = async {
        let resp_val = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|e| CxWebError::FetchFailed(format!("fetch error (timeout or CORS?): {e:?}")))?;
        let resp: Response = resp_val
            .dyn_into()
            .map_err(|_| CxWebError::FetchFailed("response cast failed".into()))?;

        if !resp.ok() {
            return Err(CxWebError::FetchFailed(format!("HTTP {}", resp.status())));
        }

        let buf_promise = resp
            .array_buffer()
            .map_err(|e| CxWebError::FetchFailed(format!("array_buffer error: {e:?}")))?;
        let buf_val = JsFuture::from(buf_promise)
            .await
            .map_err(|e| CxWebError::FetchFailed(format!("buffer read error: {e:?}")))?;
        let buf: ArrayBuffer = buf_val
            .dyn_into()
            .map_err(|_| CxWebError::FetchFailed("buffer cast failed".into()))?;
        let array = Uint8Array::new(&buf);

        Ok(array.to_vec())
    }
    .await;

    window.clear_timeout_with_handle(timeout_id);
    result
}

/// Download a .conda package from the given URL and return its extracted contents.
#[wasm_bindgen]
pub async fn download_and_list_package(url: String) -> Result<JsValue, JsValue> {
    let contents = download_and_list_impl(&url).await?;
    to_js(&contents)
}

async fn download_and_list_impl(
    url: &str,
) -> Result<extract::CondaPackageContents, CxWebError> {
    web_sys::console::log_1(&format!("Downloading {url}...").into());

    let bytes = fetch_bytes(url).await?;
    let size_kb = bytes.len() / 1024;
    web_sys::console::log_1(&format!("Downloaded {size_kb} KB, extracting...").into());

    let contents = if url.ends_with(".conda") {
        extract::extract_conda(&bytes)?
    } else if url.ends_with(".tar.bz2") {
        extract::extract_tar_bz2(&bytes)?
    } else {
        return Err(CxWebError::UnknownPackageFormat(url.to_string()));
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

    Ok(contents)
}

/// Bootstrap all packages from a lockfile for the given platform.
///
/// Downloads and extracts every .conda package. Returns a JS object with the full file tree.
/// `progress` is an optional JS callback: `progress(current, total, packageName)`.
#[wasm_bindgen]
pub async fn cx_bootstrap(
    lockfile_content: String,
    platform: String,
    progress: Option<js_sys::Function>,
) -> Result<JsValue, JsValue> {
    let result =
        bootstrap::bootstrap_impl(&lockfile_content, &platform, progress.as_ref()).await?;
    to_js(&result)
}

#[derive(Debug, Serialize)]
struct BootstrapPlan {
    platform: String,
    package_count: usize,
    total_download_size: u64,
    packages: Vec<PackagePlanEntry>,
}

#[derive(Debug, Serialize)]
struct PackagePlanEntry {
    name: String,
    version: String,
    url: String,
    size: Option<u64>,
}

/// Get a summary of what bootstrap would do: package count, names, total download size.
#[wasm_bindgen]
pub fn cx_bootstrap_plan(
    lockfile_content: &str,
    platform_str: &str,
) -> Result<JsValue, JsValue> {
    let platform = parse_platform(platform_str)?;
    let records = bootstrap::get_records(lockfile_content, platform)?;

    let packages: Vec<PackagePlanEntry> = records
        .iter()
        .map(|r| PackagePlanEntry {
            name: r.package_record.name.as_normalized().to_string(),
            version: r.package_record.version.to_string(),
            url: r.url.to_string(),
            size: r.package_record.size,
        })
        .collect();

    let total_size: u64 = records
        .iter()
        .filter_map(|r| r.package_record.size)
        .sum();

    let plan = BootstrapPlan {
        platform: platform_str.to_string(),
        package_count: packages.len(),
        total_download_size: total_size,
        packages,
    };

    to_js(&plan)
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
