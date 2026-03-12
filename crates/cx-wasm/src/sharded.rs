//! CEP-16 sharded repodata support for browser environments.
//!
//! Provides synchronous decoding functions for the shard index and individual
//! shards.  HTTP fetching and iterative dependency crawling are handled by the
//! JavaScript worker; Rust handles the binary decoding (zstd + msgpack → JSON).

use std::collections::BTreeMap;
use std::io::Read;

use rattler_conda_types::{PackageRecord, Shard};
use serde::Deserialize;
use wasm_bindgen::prelude::*;

use crate::error::CxWasmError;

/// Raw shard index deserialized from msgpack.
///
/// We avoid rattler's `ShardedRepodata` type because its `serde_with` +
/// `ahash::HashMap` + `SerializableHash` chain silently produces an empty
/// map in wasm32 builds.  Using `serde_bytes::ByteBuf` directly handles
/// msgpack binary values without any custom serde adapters.
#[derive(Deserialize)]
struct RawShardedRepodata {
    info: RawShardedSubdirInfo,
    shards: BTreeMap<String, serde_bytes::ByteBuf>,
}

#[derive(Deserialize)]
struct RawShardedSubdirInfo {
    base_url: String,
    shards_base_url: String,
}

/// Decode a zstd+msgpack shard index into a JSON string.
///
/// Input: raw bytes of `repodata_shards.msgpack.zst`
/// Returns a JSON string: `{ "base_url": "...", "shards_base_url": "...", "shards": {"name": "hex_hash", ...} }`
#[wasm_bindgen]
pub fn cx_decode_shard_index(compressed: &[u8]) -> Result<String, JsValue> {
    let decompressed = decompress_zstd(compressed)?;
    let index: RawShardedRepodata = rmp_serde::from_slice(&decompressed)
        .map_err(|e| CxWasmError::RepodataParse(format!("msgpack decode shard index: {e}")))?;

    let shards: BTreeMap<String, String> = index
        .shards
        .into_iter()
        .map(|(name, hash)| {
            let hex: String = hash.iter().map(|b| format!("{b:02x}")).collect();
            (name, hex)
        })
        .collect();

    let result = serde_json::json!({
        "base_url": index.info.base_url,
        "shards_base_url": index.info.shards_base_url,
        "shards": shards,
    });

    serde_json::to_string(&result)
        .map_err(|e| CxWasmError::SerializeFailed(e.to_string()).into())
}

/// Decode a zstd+msgpack shard into repodata.json-compatible JSON string.
///
/// Input: raw bytes of a single `<hash>.msgpack.zst` shard file.
/// Returns a JSON string with `{ "packages": {...}, "packages.conda": {...} }`.
#[wasm_bindgen]
pub fn cx_decode_shard(compressed: &[u8]) -> Result<String, JsValue> {
    let decompressed = decompress_zstd(compressed)?;
    let shard: Shard = rmp_serde::from_slice(&decompressed)
        .map_err(|e| CxWasmError::RepodataParse(format!("msgpack decode shard: {e}")))?;

    shard_to_repodata_json(&shard)
        .map_err(|e| -> JsValue { e.into() })
}

/// Convert a `Shard` to a repodata.json-compatible JSON string.
///
/// The `cx_solve` function expects repodata in the standard format:
/// `{ "packages": { "file.tar.bz2": {...} }, "packages.conda": { "file.conda": {...} } }`
fn shard_to_repodata_json(shard: &Shard) -> Result<String, CxWasmError> {
    let mut packages = serde_json::Map::new();
    let mut conda_packages = serde_json::Map::new();

    for (id, record) in &shard.packages {
        let filename = id.to_string();
        if shard.removed.contains(id) {
            continue;
        }
        let value = serde_json::to_value(record)
            .map_err(|e| CxWasmError::SerializeFailed(format!("serialize package record: {e}")))?;
        packages.insert(filename, value);
    }

    for (id, record) in &shard.conda_packages {
        let filename = id.to_string();
        if shard.removed.contains(id) {
            continue;
        }
        let value = serde_json::to_value(record)
            .map_err(|e| CxWasmError::SerializeFailed(format!("serialize package record: {e}")))?;
        conda_packages.insert(filename, value);
    }

    let repodata = serde_json::json!({
        "packages": packages,
        "packages.conda": conda_packages,
    });

    serde_json::to_string(&repodata)
        .map_err(|e| CxWasmError::SerializeFailed(format!("serialize repodata: {e}")))
}

/// Decompress zstd-compressed data using ruzstd.
fn decompress_zstd(compressed: &[u8]) -> Result<Vec<u8>, CxWasmError> {
    if compressed.is_empty() {
        return Err(CxWasmError::RepodataParse(
            "empty zstd input (server returned 0 bytes)".into(),
        ));
    }

    let mut decoder = ruzstd::decoding::StreamingDecoder::new(compressed)
        .map_err(|e| CxWasmError::RepodataParse(format!("zstd init: {e}")))?;

    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| CxWasmError::RepodataParse(format!("zstd decompress: {e}")))?;

    Ok(decompressed)
}

/// Extract unique dependency package names from a decoded shard's records.
///
/// Useful for iterative dependency crawling: parse a shard, extract dep names,
/// fetch those shards next.
#[wasm_bindgen]
pub fn cx_shard_dep_names(compressed: &[u8]) -> Result<JsValue, JsValue> {
    let decompressed = decompress_zstd(compressed)?;
    let shard: Shard = rmp_serde::from_slice(&decompressed)
        .map_err(|e| CxWasmError::RepodataParse(format!("msgpack decode shard: {e}")))?;

    let mut dep_names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    let all_records = shard
        .packages
        .values()
        .chain(shard.conda_packages.values());

    for record in all_records {
        extract_dep_names(record, &mut dep_names);
    }

    serde_wasm_bindgen::to_value(&dep_names)
        .map_err(|e| CxWasmError::SerializeFailed(e.to_string()).into())
}

fn extract_dep_names(record: &PackageRecord, names: &mut std::collections::BTreeSet<String>) {
    for dep in &record.depends {
        if let Some(name) = dep.split_whitespace().next() {
            names.insert(name.to_string());
        }
    }
}
