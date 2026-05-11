use std::io::{Cursor, Read};

use bzip2_rs::DecoderReader;
use ruzstd::decoding::StreamingDecoder;
use serde::Serialize;

use crate::error::CxWasmError;

const MAX_ENTRY_SIZE: u64 = 256 * 1024 * 1024; // 256 MB per file
const MAX_TOTAL_SIZE: u64 = 2 * 1024 * 1024 * 1024; // 2 GB total

#[derive(Debug, Default, Serialize)]
pub struct ExtractStats {
    pub file_count: usize,
    pub total_size: usize,
}

fn is_safe_tar_path(path: &str) -> bool {
    if path.is_empty() {
        return false;
    }
    if path.starts_with('/') || path.starts_with('\\') {
        return false;
    }
    if path.contains("..") {
        return false;
    }
    if path.contains('\\') {
        return false;
    }
    if path.starts_with("C:") || path.starts_with("c:") || path.contains(":\\") {
        return false;
    }
    true
}

fn stream_tar_entries<R: Read, F>(
    tar: &mut tar::Archive<R>,
    on_file: &mut F,
) -> Result<ExtractStats, CxWasmError>
where
    F: FnMut(&str, &[u8]) -> Result<(), CxWasmError>,
{
    use std::collections::HashMap;

    let mut stats = ExtractStats::default();
    let mut file_contents: HashMap<String, Vec<u8>> = HashMap::new();
    let mut deferred_links: Vec<(String, String)> = Vec::new();

    for entry_result in tar
        .entries()
        .map_err(|e| CxWasmError::ExtractFailed(format!("tar entries error: {e}")))?
    {
        let mut entry = entry_result
            .map_err(|e| CxWasmError::ExtractFailed(format!("tar entry error: {e}")))?;

        let entry_type = entry.header().entry_type();

        let path = entry
            .path()
            .map_err(|e| CxWasmError::ExtractFailed(format!("tar path error: {e}")))?
            .to_string_lossy()
            .into_owned();

        if !is_safe_tar_path(&path) {
            return Err(CxWasmError::ExtractFailed(format!(
                "unsafe tar path rejected: {path}"
            )));
        }

        if entry_type == tar::EntryType::Symlink || entry_type == tar::EntryType::Link {
            let link_target = entry
                .link_name()
                .map_err(|e| {
                    CxWasmError::ExtractFailed(format!("reading link name for {path}: {e}"))
                })?
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default();

            let resolved = if entry_type == tar::EntryType::Symlink {
                if let Some(parent) = std::path::Path::new(&path).parent() {
                    parent
                        .join(&link_target)
                        .components()
                        .fold(std::path::PathBuf::new(), |mut acc, c| {
                            match c {
                                std::path::Component::ParentDir => { acc.pop(); }
                                std::path::Component::Normal(s) => acc.push(s),
                                _ => {}
                            }
                            acc
                        })
                        .to_string_lossy()
                        .into_owned()
                } else {
                    link_target.clone()
                }
            } else {
                link_target.clone()
            };

            deferred_links.push((path, resolved));
            continue;
        }

        if !entry_type.is_file() {
            continue;
        }

        let declared_size = entry.size();
        if declared_size > MAX_ENTRY_SIZE {
            return Err(CxWasmError::ExtractFailed(format!(
                "tar entry too large ({} bytes): {path}",
                declared_size
            )));
        }

        let capacity = (declared_size as usize).min(16 * 1024 * 1024);
        let mut buf = Vec::with_capacity(capacity);
        entry
            .read_to_end(&mut buf)
            .map_err(|e| CxWasmError::ExtractFailed(format!("reading tar entry {path}: {e}")))?;

        stats.file_count += 1;
        stats.total_size += buf.len();

        if stats.total_size as u64 > MAX_TOTAL_SIZE {
            return Err(CxWasmError::ExtractFailed(
                "extraction exceeded total size limit (2 GB)".into(),
            ));
        }

        on_file(&path, &buf)?;
        file_contents.insert(path, buf);
    }

    for (path, target) in deferred_links {
        if let Some(data) = file_contents.get(&target) {
            stats.file_count += 1;
            stats.total_size += data.len();
            on_file(&path, data)?;
        } else {
            stats.file_count += 1;
            on_file(&path, &[])?;
        }
    }

    Ok(stats)
}

pub fn extract_conda_streaming<F>(bytes: &[u8], mut on_file: F) -> Result<ExtractStats, CxWasmError>
where
    F: FnMut(&str, &[u8]) -> Result<(), CxWasmError>,
{
    let reader = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(reader)
        .map_err(|e| CxWasmError::ExtractFailed(format!("opening ZIP: {e}")))?;

    let mut stats = ExtractStats::default();

    let entry_names: Vec<String> = (0..archive.len())
        .filter_map(|i| archive.by_index(i).ok().map(|e| e.name().to_string()))
        .collect();

    for name in &entry_names {
        if name.ends_with(".tar.zst") {
            let entry = archive.by_name(name).map_err(|e| {
                CxWasmError::ExtractFailed(format!("reading ZIP entry {name}: {e}"))
            })?;
            let inner = extract_tar_zst_streaming(entry, &mut on_file)?;
            stats.file_count += inner.file_count;
            stats.total_size += inner.total_size;
        }
    }

    Ok(stats)
}

pub fn extract_tar_bz2_streaming<F>(
    bytes: &[u8],
    mut on_file: F,
) -> Result<ExtractStats, CxWasmError>
where
    F: FnMut(&str, &[u8]) -> Result<(), CxWasmError>,
{
    let reader = Cursor::new(bytes);
    let decoder = DecoderReader::new(reader);
    let mut tar = tar::Archive::new(decoder);
    stream_tar_entries(&mut tar, &mut on_file)
}

fn extract_tar_zst_streaming<R: Read, F>(
    zst_reader: R,
    on_file: &mut F,
) -> Result<ExtractStats, CxWasmError>
where
    F: FnMut(&str, &[u8]) -> Result<(), CxWasmError>,
{
    let mut zst_reader = zst_reader;
    let decoder = StreamingDecoder::new(&mut zst_reader)
        .map_err(|e| CxWasmError::ExtractFailed(format!("zstd decode error: {e}")))?;

    let mut tar = tar::Archive::new(decoder);
    stream_tar_entries(&mut tar, on_file)
}
