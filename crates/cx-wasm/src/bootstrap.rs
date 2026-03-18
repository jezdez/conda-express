use std::io::Cursor;
use std::str::FromStr;

use rattler_conda_types::{Platform, RepoDataRecord};
use rattler_lock::LockFile;
use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::error::CxWasmError;
use crate::extract;

#[derive(Debug, Serialize)]
pub struct StreamingBootstrapResult {
    pub platform: String,
    pub total_packages: usize,
    pub packages_installed: usize,
    pub total_files: usize,
    pub total_size: usize,
    pub errors: Vec<String>,
}

pub(crate) fn get_records(
    lockfile_content: &str,
    platform: Platform,
) -> Result<Vec<RepoDataRecord>, CxWasmError> {
    let reader = Cursor::new(lockfile_content.as_bytes());
    let lockfile =
        LockFile::from_reader(reader).map_err(|e| CxWasmError::LockfileParse(e.to_string()))?;
    let env = lockfile
        .default_environment()
        .ok_or(CxWasmError::NoDefaultEnvironment)?;

    env.conda_repodata_records(platform)
        .map_err(|e| CxWasmError::LockfileParse(e.to_string()))?
        .ok_or_else(|| CxWasmError::NoRecordsForPlatform(platform.as_str().to_string()))
}

pub async fn bootstrap_streaming_impl(
    lockfile_content: &str,
    platform_str: &str,
    on_progress: Option<&js_sys::Function>,
    on_file: &js_sys::Function,
) -> Result<StreamingBootstrapResult, CxWasmError> {
    let platform = Platform::from_str(platform_str)
        .map_err(|_| CxWasmError::PlatformUnknown(platform_str.to_string()))?;

    let records = get_records(lockfile_content, platform)?;
    let total = records.len();

    web_sys::console::log_1(
        &format!("Streaming bootstrap: {total} packages for {platform_str}").into(),
    );

    let mut result = StreamingBootstrapResult {
        platform: platform_str.to_string(),
        total_packages: total,
        packages_installed: 0,
        total_files: 0,
        total_size: 0,
        errors: Vec::new(),
    };

    for (i, r) in records.into_iter().enumerate() {
        let name = r.package_record.name.as_normalized().to_string();
        let version = r.package_record.version.to_string();
        let url = r.url.to_string();

        if let Some(cb) = on_progress {
            let _ = cb.call3(
                &JsValue::NULL,
                &JsValue::from(i as u32),
                &JsValue::from(total as u32),
                &JsValue::from(&name),
            );
        }

        match download_and_extract_package_streaming(&name, &url, on_file).await {
            Ok(stats) => {
                result.packages_installed += 1;
                result.total_files += stats.file_count;
                result.total_size += stats.total_size;

                web_sys::console::log_1(
                    &format!(
                        "  Installed {name} {version}: {} files, {} KB",
                        stats.file_count,
                        stats.total_size / 1024
                    )
                    .into(),
                );
            }
            Err(e) => {
                let msg = format!("{name}: {e}");
                web_sys::console::warn_1(&format!("Skipping {msg}").into());
                result.errors.push(msg);
            }
        }
    }

    if let Some(cb) = on_progress {
        let _ = cb.call3(
            &JsValue::NULL,
            &JsValue::from(total as u32),
            &JsValue::from(total as u32),
            &JsValue::from("done"),
        );
    }

    web_sys::console::log_1(
        &format!(
            "Streaming bootstrap complete: {} packages, {} files, {} KB",
            result.packages_installed,
            result.total_files,
            result.total_size / 1024
        )
        .into(),
    );

    Ok(result)
}

async fn download_and_extract_package_streaming(
    name: &str,
    url: &str,
    on_file: &js_sys::Function,
) -> Result<extract::ExtractStats, CxWasmError> {
    web_sys::console::log_1(&format!("  Downloading {name} from {url}").into());
    let bytes = crate::fetch_bytes(url).await?;
    web_sys::console::log_1(
        &format!(
            "  Downloaded {name}: {} KB, extracting...",
            bytes.len() / 1024
        )
        .into(),
    );

    let js_name = JsValue::from(name);
    let mut file_cb = |path: &str, data: &[u8]| -> Result<(), CxWasmError> {
        let js_path = JsValue::from(path);
        let js_bytes = js_sys::Uint8Array::from(data);
        on_file
            .call3(&JsValue::NULL, &js_name, &js_path, &js_bytes)
            .map_err(|e| CxWasmError::CallbackFailed(format!("{e:?}")))?;
        Ok(())
    };

    if url.ends_with(".conda") {
        extract::extract_conda_streaming(&bytes, &mut file_cb)
    } else if url.ends_with(".tar.bz2") {
        extract::extract_tar_bz2_streaming(&bytes, &mut file_cb)
    } else {
        Err(CxWasmError::UnknownPackageFormat(url.to_string()))
    }
}
