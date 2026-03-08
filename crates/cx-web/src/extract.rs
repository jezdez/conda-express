use std::io::{Cursor, Read};

use bzip2_rs::DecoderReader;
use ruzstd::decoding::StreamingDecoder;
use serde::Serialize;

use crate::error::CxWebError;

#[derive(Debug, Serialize)]
pub struct ExtractedFile {
    pub path: String,
    pub size: usize,
}

#[derive(Debug, Serialize)]
pub struct CondaPackageContents {
    pub info_files: Vec<ExtractedFile>,
    pub pkg_files: Vec<ExtractedFile>,
    pub total_size: usize,
}

/// Extract a `.conda` archive from raw bytes, returning metadata about contents.
///
/// `.conda` format: outer uncompressed ZIP containing:
///   - `info-*.tar.zst` (package metadata)
///   - `pkg-*.tar.zst` (package files)
pub fn extract_conda(bytes: &[u8]) -> Result<CondaPackageContents, CxWebError> {
    let reader = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(reader)
        .map_err(|e| CxWebError::ExtractFailed(format!("opening ZIP: {e}")))?;

    let mut info_files = Vec::new();
    let mut pkg_files = Vec::new();
    let mut total_size = 0usize;

    let entry_names: Vec<String> = (0..archive.len())
        .filter_map(|i| archive.by_index(i).ok().map(|e| e.name().to_string()))
        .collect();

    for name in &entry_names {
        if name.ends_with(".tar.zst") {
            let entry = archive.by_name(name).map_err(|e| {
                CxWebError::ExtractFailed(format!("reading ZIP entry {name}: {e}"))
            })?;

            let is_info = name.starts_with("info-");
            for file in extract_tar_zst(entry)? {
                total_size += file.size;
                if is_info {
                    info_files.push(file);
                } else {
                    pkg_files.push(file);
                }
            }
        }
    }

    Ok(CondaPackageContents {
        info_files,
        pkg_files,
        total_size,
    })
}

/// Extract a `.tar.bz2` archive from raw bytes.
pub fn extract_tar_bz2(bytes: &[u8]) -> Result<CondaPackageContents, CxWebError> {
    let reader = Cursor::new(bytes);
    let decoder = DecoderReader::new(reader);
    let mut tar = tar::Archive::new(decoder);
    let mut info_files = Vec::new();
    let mut pkg_files = Vec::new();
    let mut total_size = 0usize;

    for entry_result in tar
        .entries()
        .map_err(|e| CxWebError::ExtractFailed(format!("tar entries error: {e}")))?
    {
        let entry =
            entry_result.map_err(|e| CxWebError::ExtractFailed(format!("tar entry error: {e}")))?;
        let path = entry
            .path()
            .map_err(|e| CxWebError::ExtractFailed(format!("tar path error: {e}")))?
            .to_string_lossy()
            .into_owned();
        let size = entry.size() as usize;
        total_size += size;

        let file = ExtractedFile {
            path: path.clone(),
            size,
        };
        if path.starts_with("info/") {
            info_files.push(file);
        } else {
            pkg_files.push(file);
        }
    }

    Ok(CondaPackageContents {
        info_files,
        pkg_files,
        total_size,
    })
}

/// Decompress a zstd-compressed tar stream and list the entries.
fn extract_tar_zst<R: Read>(zst_reader: R) -> Result<Vec<ExtractedFile>, CxWebError> {
    let mut zst_reader = zst_reader;
    let decoder = StreamingDecoder::new(&mut zst_reader)
        .map_err(|e| CxWebError::ExtractFailed(format!("zstd decode error: {e}")))?;

    let mut tar = tar::Archive::new(decoder);
    let mut files = Vec::new();

    for entry_result in tar
        .entries()
        .map_err(|e| CxWebError::ExtractFailed(format!("tar entries error: {e}")))?
    {
        let entry =
            entry_result.map_err(|e| CxWebError::ExtractFailed(format!("tar entry error: {e}")))?;
        let path = entry
            .path()
            .map_err(|e| CxWebError::ExtractFailed(format!("tar path error: {e}")))?
            .to_string_lossy()
            .into_owned();
        let size = entry.size() as usize;
        files.push(ExtractedFile { path, size });
    }

    Ok(files)
}
