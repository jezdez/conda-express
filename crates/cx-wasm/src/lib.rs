mod bootstrap;
mod error;
mod extract;
mod solve;

use std::str::FromStr;

use rattler_conda_types::Platform;
use rattler_lock::LockFile;
use serde::Serialize;
use wasm_bindgen::prelude::*;

use error::CxWasmError;

#[cfg(cx_embedded_lockfile)]
const EMBEDDED_LOCKFILE: &str = include_str!(concat!(env!("OUT_DIR"), "/embedded_lockfile.txt"));

#[cfg(cx_embedded_platform)]
const EMBEDDED_PLATFORM: &str = include_str!(concat!(env!("OUT_DIR"), "/embedded_platform.txt"));

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
    build: string;
    build_number: number;
    subdir: string;
    url: string;
    channel: string;
    fn_name: string;
    size: number | null;
    sha256: string | null;
    md5: string | null;
}

export interface BootstrapPlan {
    platform: string;
    package_count: number;
    total_download_size: number;
    packages: PackagePlanEntry[];
}

export interface StreamingBootstrapResult {
    platform: string;
    total_packages: number;
    packages_installed: number;
    total_files: number;
    total_size: number;
    errors: string[];
}

export interface ExtractStats {
    file_count: number;
    total_size: number;
}

export type OnFileCallback = (packageName: string, path: string, bytes: Uint8Array) => void;
export type OnSingleFileCallback = (path: string, bytes: Uint8Array) => void;
export type OnProgressCallback = (current: number, total: number, packageName: string) => void;
"#;

fn to_js<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value).map_err(|e| CxWasmError::SerializeFailed(e.to_string()).into())
}

fn parse_platform(platform_str: &str) -> Result<Platform, JsValue> {
    Platform::from_str(platform_str)
        .map_err(|_| CxWasmError::PlatformUnknown(platform_str.to_string()).into())
}

fn parse_lockfile(lockfile_content: &str) -> Result<LockFile, JsValue> {
    let reader = std::io::Cursor::new(lockfile_content.as_bytes());
    LockFile::from_reader(reader).map_err(|e| CxWasmError::LockfileParse(e.to_string()).into())
}

/// Return a JS array of all platform strings found in a lockfile.
#[wasm_bindgen]
pub fn get_platforms(lockfile_content: &str) -> Result<JsValue, JsValue> {
    let lockfile = parse_lockfile(lockfile_content)?;
    let env = lockfile
        .default_environment()
        .ok_or::<JsValue>(CxWasmError::NoDefaultEnvironment.into())?;

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
    web_sys::console::log_1(&"cx-wasm initialized".into());
    format!("cx-wasm v{}", env!("CARGO_PKG_VERSION"))
}

/// Returns the embedded lockfile content, or `undefined` if none was baked in at build time.
#[wasm_bindgen]
pub fn cx_embedded_lockfile() -> Option<String> {
    #[cfg(cx_embedded_lockfile)]
    {
        Some(EMBEDDED_LOCKFILE.to_string())
    }
    #[cfg(not(cx_embedded_lockfile))]
    {
        None
    }
}

/// Returns the embedded target platform (e.g. "emscripten-wasm32"), or `undefined`.
#[wasm_bindgen]
pub fn cx_embedded_platform() -> Option<String> {
    #[cfg(cx_embedded_platform)]
    {
        Some(EMBEDDED_PLATFORM.to_string())
    }
    #[cfg(not(cx_embedded_platform))]
    {
        None
    }
}

/// Convenience: streaming bootstrap using the embedded lockfile and platform.
///
/// Fails if no lockfile or platform was embedded at build time.
#[wasm_bindgen]
pub async fn cx_bootstrap_embedded(
    on_progress: Option<js_sys::Function>,
    on_file: js_sys::Function,
) -> Result<JsValue, JsValue> {
    let lockfile = cx_embedded_lockfile()
        .ok_or_else(|| CxWasmError::NotEmbedded("lockfile".into()))?;
    let platform = cx_embedded_platform()
        .ok_or_else(|| CxWasmError::NotEmbedded("platform".into()))?;

    let result = bootstrap::bootstrap_streaming_impl(
        &lockfile,
        &platform,
        on_progress.as_ref(),
        &on_file,
    )
    .await?;
    to_js(&result)
}

/// Fetch bytes from a URL using the browser Fetch API with a 5-minute timeout.
pub(crate) async fn fetch_bytes(url: &str) -> Result<Vec<u8>, CxWasmError> {
    use js_sys::{ArrayBuffer, Uint8Array};
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{AbortController, Request, RequestInit, RequestMode, Response};

    let controller = AbortController::new()
        .map_err(|e| CxWasmError::FetchFailed(format!("AbortController error: {e:?}")))?;
    let signal = controller.signal();

    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);
    opts.set_signal(Some(&signal));

    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|e| CxWasmError::FetchFailed(format!("request error: {e:?}")))?;

    let window = web_sys::window().ok_or(CxWasmError::FetchFailed("no global window".into()))?;

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
        .map_err(|e| CxWasmError::FetchFailed(format!("setTimeout error: {e:?}")))?;

    let result = async {
        let resp_val = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|e| CxWasmError::FetchFailed(format!("fetch error (timeout or CORS?): {e:?}")))?;
        let resp: Response = resp_val
            .dyn_into()
            .map_err(|_| CxWasmError::FetchFailed("response cast failed".into()))?;

        if !resp.ok() {
            return Err(CxWasmError::FetchFailed(format!("HTTP {}", resp.status())));
        }

        let buf_promise = resp
            .array_buffer()
            .map_err(|e| CxWasmError::FetchFailed(format!("array_buffer error: {e:?}")))?;
        let buf_val = JsFuture::from(buf_promise)
            .await
            .map_err(|e| CxWasmError::FetchFailed(format!("buffer read error: {e:?}")))?;
        let buf: ArrayBuffer = buf_val
            .dyn_into()
            .map_err(|_| CxWasmError::FetchFailed("buffer cast failed".into()))?;
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
) -> Result<extract::CondaPackageContents, CxWasmError> {
    web_sys::console::log_1(&format!("Downloading {url}...").into());

    let bytes = fetch_bytes(url).await?;
    let size_kb = bytes.len() / 1024;
    web_sys::console::log_1(&format!("Downloaded {size_kb} KB, extracting...").into());

    let contents = if url.ends_with(".conda") {
        extract::extract_conda(&bytes)?
    } else if url.ends_with(".tar.bz2") {
        extract::extract_tar_bz2(&bytes)?
    } else {
        return Err(CxWasmError::UnknownPackageFormat(url.to_string()));
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

/// Streaming bootstrap: download and extract all packages, calling `on_file` for each
/// extracted file with `(packageName, path, bytes)`.
///
/// Use this to write files directly to a virtual filesystem (e.g., Emscripten MEMFS)
/// without buffering everything in memory.
///
/// `on_progress` is an optional callback: `on_progress(current, total, packageName)`.
/// `on_file` is a required callback: `on_file(packageName, path, bytes: Uint8Array)`.
#[wasm_bindgen]
pub async fn cx_bootstrap_streaming(
    lockfile_content: String,
    platform: String,
    on_progress: Option<js_sys::Function>,
    on_file: js_sys::Function,
) -> Result<JsValue, JsValue> {
    let result = bootstrap::bootstrap_streaming_impl(
        &lockfile_content,
        &platform,
        on_progress.as_ref(),
        &on_file,
    )
    .await?;
    to_js(&result)
}

/// Download a single package and stream its extracted files to `on_file(path, bytes)`.
///
/// Returns extraction stats (file count, total size).
#[wasm_bindgen]
pub async fn download_and_extract_package_streaming(
    url: String,
    on_file: js_sys::Function,
) -> Result<JsValue, JsValue> {
    let bytes = fetch_bytes(&url).await?;

    let mut file_cb = |path: &str, data: &[u8]| -> Result<(), CxWasmError> {
        let js_path = JsValue::from(path);
        let js_bytes = js_sys::Uint8Array::from(data);
        on_file
            .call2(&JsValue::NULL, &js_path, &js_bytes)
            .map_err(|e| CxWasmError::CallbackFailed(format!("{e:?}")))?;
        Ok(())
    };

    let stats = if url.ends_with(".conda") {
        extract::extract_conda_streaming(&bytes, &mut file_cb)?
    } else if url.ends_with(".tar.bz2") {
        extract::extract_tar_bz2_streaming(&bytes, &mut file_cb)?
    } else {
        return Err(CxWasmError::UnknownPackageFormat(url.to_string()).into());
    };

    to_js(&stats)
}

/// Extract a `.conda` or `.tar.bz2` package from raw bytes (already in memory).
///
/// This is the synchronous counterpart to `download_and_extract_package_streaming`:
/// it skips the download step and works directly on bytes read from the filesystem.
///
/// `on_file` callback signature: `(path: string, data: Uint8Array) => void`
#[wasm_bindgen]
pub fn cx_extract_package(
    bytes: &[u8],
    filename: &str,
    on_file: js_sys::Function,
) -> Result<JsValue, JsValue> {
    let mut file_cb = |path: &str, data: &[u8]| -> Result<(), CxWasmError> {
        let js_path = JsValue::from(path);
        let js_bytes = js_sys::Uint8Array::from(data);
        on_file
            .call2(&JsValue::NULL, &js_path, &js_bytes)
            .map_err(|e| CxWasmError::CallbackFailed(format!("{e:?}")))?;
        Ok(())
    };

    let stats = if filename.ends_with(".conda") {
        extract::extract_conda_streaming(bytes, &mut file_cb)?
    } else if filename.ends_with(".tar.bz2") {
        extract::extract_tar_bz2_streaming(bytes, &mut file_cb)?
    } else {
        return Err(CxWasmError::UnknownPackageFormat(filename.to_string()).into());
    };

    to_js(&stats)
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
    build: String,
    build_number: u64,
    subdir: String,
    url: String,
    channel: String,
    fn_name: String,
    size: Option<u64>,
    sha256: Option<String>,
    md5: Option<String>,
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
        .map(|r| {
            let url_str = r.url.to_string();
            let fn_name = url_str
                .rsplit('/')
                .next()
                .unwrap_or("unknown")
                .to_string();
            let channel = r.channel.clone().unwrap_or_default();
            PackagePlanEntry {
                name: r.package_record.name.as_normalized().to_string(),
                version: r.package_record.version.to_string(),
                build: r.package_record.build.clone(),
                build_number: r.package_record.build_number,
                subdir: r.package_record.subdir.clone(),
                url: url_str,
                channel,
                fn_name,
                size: r.package_record.size,
                sha256: r.package_record.sha256.map(|h| format!("{h:x}")),
                md5: r.package_record.md5.map(|h| format!("{h:x}")),
            }
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
