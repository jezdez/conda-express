use std::io::Cursor;
use std::str::FromStr;

use rattler_conda_types::{Platform, RepoDataRecord};
use rattler_lock::LockFile;
use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::error::CxWasmError;
use crate::extract;

#[derive(Debug, Serialize)]
pub struct PackageResult {
    pub name: String,
    pub version: String,
    pub url: String,
    pub info_files: Vec<extract::ExtractedFile>,
    pub pkg_files: Vec<extract::ExtractedFile>,
    pub total_size: usize,
}

#[derive(Debug, Serialize)]
pub struct BootstrapResult {
    pub platform: String,
    pub packages: Vec<PackageResult>,
    pub total_packages: usize,
    pub total_files: usize,
    pub total_size: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct StreamingBootstrapResult {
    pub platform: String,
    pub total_packages: usize,
    pub packages_installed: usize,
    pub total_files: usize,
    pub total_size: usize,
    pub errors: Vec<String>,
}

/// Parse lockfile records for a given platform.
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

/// Bootstrap a conda environment from a lockfile: download and extract all packages.
///
/// `progress` is an optional JS callback invoked as `progress(current, total, package_name)`
/// for each package downloaded.
pub async fn bootstrap_impl(
    lockfile_content: &str,
    platform_str: &str,
    progress: Option<&js_sys::Function>,
) -> Result<BootstrapResult, CxWasmError> {
    let platform = Platform::from_str(platform_str)
        .map_err(|_| CxWasmError::PlatformUnknown(platform_str.to_string()))?;

    let records = get_records(lockfile_content, platform)?;
    let total = records.len();

    web_sys::console::log_1(&format!("Bootstrapping {total} packages for {platform_str}").into());

    let mut result = BootstrapResult {
        platform: platform_str.to_string(),
        packages: Vec::with_capacity(total),
        total_packages: total,
        total_files: 0,
        total_size: 0,
        errors: Vec::new(),
    };

    for (i, record) in records.into_iter().enumerate() {
        let name = record.package_record.name.as_normalized().to_string();
        let version = record.package_record.version.to_string();
        let url = record.url.to_string();

        if let Some(cb) = progress {
            let _ = cb.call3(
                &JsValue::NULL,
                &JsValue::from(i as u32),
                &JsValue::from(total as u32),
                &JsValue::from(&name),
            );
        }

        match download_and_extract_package(&name, &url).await {
            Ok(contents) => {
                let file_count = contents.info_files.len() + contents.pkg_files.len();
                result.total_files += file_count;
                result.total_size += contents.total_size;

                result.packages.push(PackageResult {
                    name,
                    version,
                    url,
                    info_files: contents.info_files,
                    pkg_files: contents.pkg_files,
                    total_size: contents.total_size,
                });
            }
            Err(e) => {
                let msg = format!("{name}: {e}");
                web_sys::console::warn_1(&format!("Skipping {msg}").into());
                result.errors.push(msg);
            }
        }
    }

    if let Some(cb) = progress {
        let _ = cb.call3(
            &JsValue::NULL,
            &JsValue::from(total as u32),
            &JsValue::from(total as u32),
            &JsValue::from("done"),
        );
    }

    web_sys::console::log_1(
        &format!(
            "Bootstrap complete: {} packages, {} files, {} KB",
            result.packages.len(),
            result.total_files,
            result.total_size / 1024
        )
        .into(),
    );

    Ok(result)
}

/// Streaming bootstrap: download and extract all packages, calling `on_file` for each
/// extracted file with its path and bytes.
///
/// `on_progress(current, total, package_name)` — called before each package download.
/// `on_file(package_name, path, bytes)` — called for each extracted file.
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

    for (i, record) in records.into_iter().enumerate() {
        let name = record.package_record.name.as_normalized().to_string();
        let version = record.package_record.version.to_string();
        let url = record.url.to_string();

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
        &format!("  Downloaded {name}: {} KB, extracting...", bytes.len() / 1024).into(),
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

async fn download_and_extract_package(
    name: &str,
    url: &str,
) -> Result<extract::CondaPackageContents, CxWasmError> {
    web_sys::console::log_1(&format!("  Downloading {name} from {url}").into());
    let bytes = crate::fetch_bytes(url).await?;
    web_sys::console::log_1(
        &format!("  Downloaded {name}: {} KB, extracting...", bytes.len() / 1024).into(),
    );

    let result = if url.ends_with(".conda") {
        extract::extract_conda(&bytes)
    } else if url.ends_with(".tar.bz2") {
        extract::extract_tar_bz2(&bytes)
    } else {
        Err(CxWasmError::UnknownPackageFormat(url.to_string()))
    };

    if let Ok(ref contents) = result {
        web_sys::console::log_1(
            &format!(
                "  Extracted {name}: {} files",
                contents.info_files.len() + contents.pkg_files.len()
            )
            .into(),
        );
    }

    result
}
