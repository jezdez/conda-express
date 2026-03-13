mod bootstrap;
mod error;
mod extract;
mod gateway;
mod sharded;
mod solve;

use std::str::FromStr;

use rattler_conda_types::Platform;
use serde::Serialize;
use wasm_bindgen::prelude::*;

use error::CxWasmError;

#[cfg(cx_embedded_lockfile)]
const EMBEDDED_LOCKFILE: &str = include_str!(concat!(env!("OUT_DIR"), "/embedded_lockfile.txt"));

#[cfg(cx_embedded_platform)]
const EMBEDDED_PLATFORM: &str = include_str!(concat!(env!("OUT_DIR"), "/embedded_platform.txt"));

#[wasm_bindgen(typescript_custom_section)]
const TS_TYPES: &str = r#"
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
    serde_wasm_bindgen::to_value(value)
        .map_err(|e| CxWasmError::SerializeFailed(e.to_string()).into())
}

fn parse_platform(platform_str: &str) -> Result<Platform, JsValue> {
    Platform::from_str(platform_str)
        .map_err(|_| CxWasmError::PlatformUnknown(platform_str.to_string()).into())
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

// Global fetch/setTimeout/clearTimeout bindings that work in both Window and Worker contexts.
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = fetch)]
    fn global_fetch(input: &web_sys::Request) -> js_sys::Promise;

    #[wasm_bindgen(js_name = setTimeout)]
    fn global_set_timeout(handler: &js_sys::Function, timeout: i32) -> i32;

    #[wasm_bindgen(js_name = clearTimeout)]
    fn global_clear_timeout(id: i32);
}

/// Fetch bytes from a URL using the browser Fetch API with a 5-minute timeout.
/// Works in both Window (main thread) and Worker contexts.
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

    let timeout_id = global_set_timeout(
        &wasm_bindgen::closure::Closure::<dyn Fn()>::new({
            let controller = controller.clone();
            move || controller.abort()
        })
        .into_js_value()
        .unchecked_into(),
        300_000, // 5-minute timeout for large packages
    );

    let result = async {
        let resp_val = JsFuture::from(global_fetch(&request)).await.map_err(|e| {
            CxWasmError::FetchFailed(format!("fetch error (timeout or CORS?): {e:?}"))
        })?;
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

    global_clear_timeout(timeout_id);
    result
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

/// Extract a `.conda` or `.tar.bz2` package from raw bytes (already in memory).
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
pub fn cx_bootstrap_plan(lockfile_content: &str, platform_str: &str) -> Result<JsValue, JsValue> {
    let platform = parse_platform(platform_str)?;
    let records = bootstrap::get_records(lockfile_content, platform)?;

    let packages: Vec<PackagePlanEntry> = records
        .iter()
        .map(|r| {
            let url_str = r.url.to_string();
            let fn_name = url_str.rsplit('/').next().unwrap_or("unknown").to_string();
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

    let total_size: u64 = records.iter().filter_map(|r| r.package_record.size).sum();

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
