//! CEP-16 sharded repodata support for browser environments.
//!
//! The main entry point is [`fetch_repodata_records`], which fetches repodata
//! for a channel/subdir using sharded repodata (CEP-16) with fallback to full
//! `repodata.json`, returning parsed `RepoDataRecord`s directly.  HTTP
//! requests are performed via JS callbacks (sync XHR in the Web Worker).
//! All binary decoding (zstd + msgpack) and dependency crawling happens in Rust.
//!
//! ## Session-level caching
//!
//! Inspired by the native `cx` CLI's rattler Gateway (which caches repodata
//! on disk), this module caches both shard indices and individual shard
//! contents in memory for the lifetime of the WASM instance.  This avoids
//! redundant network fetches when the user runs multiple `%cx install`
//! commands in the same notebook session.
//!
//! - **Index cache**: keyed by `{channel}/{subdir}`, stores the decoded
//!   shard index (package-name → shard-hash mapping).
//! - **Shard cache**: keyed by shard URL (content-addressed via the hash),
//!   stores parsed `RepoDataRecord`s and dependency names.  Since the URL
//!   includes the content hash, cache entries naturally invalidate when
//!   upstream data changes.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::Read;
use std::rc::Rc;

use rattler_conda_types::{PackageRecord, Shard};
use serde::Deserialize;
use wasm_bindgen::prelude::*;

use crate::error::CxWasmError;

const MAX_SHARDS: usize = 2000;

// ─── Session-level caches ────────────────────────────────────────────────────

struct CachedShard {
    records: Vec<rattler_conda_types::RepoDataRecord>,
    dep_names: BTreeSet<String>,
}

thread_local! {
    static INDEX_CACHE: RefCell<HashMap<String, Rc<DecodedShardIndex>>> =
        RefCell::new(HashMap::new());
    static SHARD_CACHE: RefCell<HashMap<String, Rc<CachedShard>>> =
        RefCell::new(HashMap::new());
}

/// Clear all cached shard indices and shard contents.
///
/// Exposed to JS/Python so the user can force a full re-fetch if needed
/// (e.g. after a channel update mid-session).
#[wasm_bindgen]
pub fn cx_clear_repodata_cache() {
    INDEX_CACHE.with(|c| c.borrow_mut().clear());
    SHARD_CACHE.with(|c| c.borrow_mut().clear());
    web_sys::console::log_1(&"cx-wasm: repodata cache cleared".into());
}

fn get_cache_stats() -> (usize, usize) {
    let indices = INDEX_CACHE.with(|c| c.borrow().len());
    let shards = SHARD_CACHE.with(|c| c.borrow().len());
    (indices, shards)
}

// ─── Shard index types ───────────────────────────────────────────────────────

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

#[derive(Clone)]
struct DecodedShardIndex {
    #[allow(dead_code)]
    base_url: String,
    shards_base_url: String,
    shards: BTreeMap<String, String>,
}

// ─── Internal helpers ────────────────────────────────────────────────────────

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

fn decode_shard_index(compressed: &[u8]) -> Result<DecodedShardIndex, CxWasmError> {
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

    Ok(DecodedShardIndex {
        base_url: index.info.base_url,
        shards_base_url: index.info.shards_base_url,
        shards,
    })
}

/// Extract unique dependency package names from a Shard.
fn shard_dep_names(shard: &Shard) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for record in shard.packages.values().chain(shard.conda_packages.values()) {
        extract_dep_names(record, &mut names);
    }
    names
}

fn extract_dep_names(record: &PackageRecord, names: &mut BTreeSet<String>) {
    for dep in &record.depends {
        if let Some(name) = dep.split_whitespace().next() {
            names.insert(name.to_string());
        }
    }
}

/// Call a JS function with one string argument and get back a Uint8Array.
fn call_fetch_binary(cb: &js_sys::Function, url: &str) -> Result<Vec<u8>, CxWasmError> {
    let js_url = JsValue::from_str(url);
    let result = cb
        .call1(&JsValue::NULL, &js_url)
        .map_err(|e| CxWasmError::FetchFailed(format!("{url}: {e:?}")))?;

    let array = js_sys::Uint8Array::new(&result);
    Ok(array.to_vec())
}

/// Call a JS function with one string argument and get back a string.
fn call_fetch_text(cb: &js_sys::Function, url: &str) -> Result<String, CxWasmError> {
    let js_url = JsValue::from_str(url);
    let result = cb
        .call1(&JsValue::NULL, &js_url)
        .map_err(|e| CxWasmError::FetchFailed(format!("{url}: {e:?}")))?;

    result
        .as_string()
        .ok_or_else(|| CxWasmError::FetchFailed(format!("{url}: response was not a string")))
}

/// Resolve the shards base URL relative to the index URL.
fn resolve_shards_base_url(shards_base_url: &str, index_url: &str) -> String {
    let mut base = shards_base_url.to_string();

    if base.is_empty() || !base.contains("://") {
        let index_base = &index_url[..index_url.rfind('/').unwrap_or(0) + 1];
        let trimmed = base.trim_start_matches("./");
        base = format!("{index_base}{trimmed}");
    }

    if !base.ends_with('/') {
        base.push('/');
    }

    base
}

// ─── Cache helpers ───────────────────────────────────────────────────────────

/// Get the shard index from cache, or fetch + decode + cache it.
fn get_or_fetch_index(
    cache_key: &str,
    base: &str,
    subdir: &str,
    fetch_binary: &js_sys::Function,
) -> Result<Rc<DecodedShardIndex>, CxWasmError> {
    let cached = INDEX_CACHE.with(|c| c.borrow().get(cache_key).cloned());
    if let Some(index) = cached {
        web_sys::console::log_1(
            &format!("cx-wasm: shard index cache hit for {cache_key}").into(),
        );
        return Ok(index);
    }

    let index_url = format!("{base}/{subdir}/repodata_shards.msgpack.zst");
    let index_bytes = call_fetch_binary(fetch_binary, &index_url)?;
    let index = Rc::new(decode_shard_index(&index_bytes)?);

    INDEX_CACHE.with(|c| {
        c.borrow_mut()
            .insert(cache_key.to_string(), Rc::clone(&index));
    });
    web_sys::console::log_1(
        &format!(
            "cx-wasm: cached shard index for {cache_key} ({} packages)",
            index.shards.len()
        )
        .into(),
    );

    Ok(index)
}

/// Get a shard's records from cache, or fetch + decode + cache them.
fn get_or_fetch_shard(
    shard_url: &str,
    name: &str,
    fetch_binary: &js_sys::Function,
    base_url: &str,
    channel_url: &str,
) -> Result<Rc<CachedShard>, CxWasmError> {
    let cached = SHARD_CACHE.with(|c| c.borrow().get(shard_url).cloned());
    if let Some(shard) = cached {
        return Ok(shard);
    }

    let shard_bytes = call_fetch_binary(fetch_binary, shard_url)
        .map_err(|e| CxWasmError::FetchFailed(format!("shard {name}: {e}")))?;
    let decompressed = decompress_zstd(&shard_bytes)?;
    let shard: Shard = rmp_serde::from_slice(&decompressed)
        .map_err(|e| CxWasmError::RepodataParse(format!("msgpack decode shard for {name}: {e}")))?;

    let dep_names = shard_dep_names(&shard);

    let mut records = Vec::new();
    for (id, record) in shard.packages.iter().chain(shard.conda_packages.iter()) {
        if shard.removed.contains(id) {
            continue;
        }
        let url = url::Url::parse(&format!("{base_url}{id}"))
            .map_err(|e| CxWasmError::RepodataParse(format!("invalid URL for {id}: {e}")))?;
        records.push(rattler_conda_types::RepoDataRecord {
            package_record: record.clone(),
            identifier: id.clone().into(),
            url,
            channel: Some(channel_url.to_string()),
        });
    }

    let cached = Rc::new(CachedShard { records, dep_names });
    SHARD_CACHE.with(|c| {
        c.borrow_mut()
            .insert(shard_url.to_string(), Rc::clone(&cached));
    });

    Ok(cached)
}

// ─── Main entry point ────────────────────────────────────────────────────────

/// Fetch repodata and return parsed `RepoDataRecord`s directly (no JSON
/// roundtrip).  Used by [`crate::gateway::cx_fetch_and_solve`].
pub(crate) fn fetch_repodata_records(
    channel_url: &str,
    subdir: &str,
    seeds: &[String],
    fetch_binary: &js_sys::Function,
    fetch_text: &js_sys::Function,
) -> Result<Vec<rattler_conda_types::RepoDataRecord>, CxWasmError> {
    let base = channel_url.trim_end_matches('/');
    let base_url = format!("{base}/{subdir}/");

    if !seeds.is_empty() {
        match fetch_sharded_records(base, subdir, seeds, fetch_binary, &base_url) {
            Ok(records) => return Ok(records),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("fetch failed") {
                    web_sys::console::log_1(
                        &format!("cx-wasm: no shard index for {base}/{subdir}, trying fallback")
                            .into(),
                    );
                } else {
                    web_sys::console::warn_1(
                        &format!("cx-wasm: sharded repodata failed for {base}/{subdir}: {msg}")
                            .into(),
                    );
                }
            }
        }
    }

    let full_url = format!("{base}/{subdir}/repodata.json");
    let text = call_fetch_text(fetch_text, &full_url)?;
    parse_repodata_text(&text, channel_url, subdir, &base_url)
}

fn parse_repodata_text(
    text: &str,
    channel_url: &str,
    subdir: &str,
    base_url: &str,
) -> Result<Vec<rattler_conda_types::RepoDataRecord>, CxWasmError> {
    let repo: rattler_conda_types::RepoData = serde_json::from_str(text).map_err(|e| {
        CxWasmError::RepodataParse(format!("parsing repodata for {channel_url}/{subdir}: {e}"))
    })?;

    let mut records = Vec::new();
    for (identifier, pkg) in repo
        .packages
        .into_iter()
        .chain(repo.conda_packages.into_iter())
    {
        let url = url::Url::parse(&format!("{base_url}{identifier}")).map_err(|e| {
            CxWasmError::RepodataParse(format!("invalid URL for {identifier}: {e}"))
        })?;
        records.push(rattler_conda_types::RepoDataRecord {
            package_record: pkg,
            identifier,
            url,
            channel: Some(channel_url.to_string()),
        });
    }
    Ok(records)
}

fn fetch_sharded_records(
    base: &str,
    subdir: &str,
    seeds: &[String],
    fetch_binary: &js_sys::Function,
    base_url: &str,
) -> Result<Vec<rattler_conda_types::RepoDataRecord>, CxWasmError> {
    let cache_key = format!("{base}/{subdir}");
    let index = get_or_fetch_index(&cache_key, base, subdir, fetch_binary)?;

    let index_url = format!("{base}/{subdir}/repodata_shards.msgpack.zst");
    let shards_base = resolve_shards_base_url(&index.shards_base_url, &index_url);
    let channel_url = base.to_string();

    let mut fetched_names: BTreeSet<String> = BTreeSet::new();
    let mut all_records: Vec<rattler_conda_types::RepoDataRecord> = Vec::new();
    let mut fetched = 0usize;
    let mut cache_hits = 0usize;
    let mut to_fetch: Vec<String> = seeds.to_vec();

    while !to_fetch.is_empty() && fetched < MAX_SHARDS {
        let mut next_round: Vec<String> = Vec::new();

        for name in &to_fetch {
            if fetched >= MAX_SHARDS {
                break;
            }
            if fetched_names.contains(name) {
                continue;
            }
            fetched_names.insert(name.clone());

            let hash = match index.shards.get(name) {
                Some(h) => h,
                None => continue,
            };

            let shard_url = format!("{shards_base}{hash}.msgpack.zst");

            let was_cached =
                SHARD_CACHE.with(|c| c.borrow().contains_key(&shard_url));

            match get_or_fetch_shard(&shard_url, name, fetch_binary, base_url, &channel_url) {
                Ok(cached_shard) => {
                    all_records.extend(cached_shard.records.iter().cloned());
                    fetched += 1;
                    if was_cached {
                        cache_hits += 1;
                    }

                    for dep in &cached_shard.dep_names {
                        if !fetched_names.contains(dep) && index.shards.contains_key(dep) {
                            next_round.push(dep.clone());
                        }
                    }
                }
                Err(e) => {
                    web_sys::console::warn_1(
                        &format!("cx-wasm: failed to fetch shard for {name}: {e}").into(),
                    );
                }
            }
        }

        to_fetch = next_round;
    }

    if all_records.is_empty() {
        return Err(CxWasmError::RepodataParse(format!(
            "no sharded records for {base}/{subdir}"
        )));
    }

    let (total_indices, total_shards) = get_cache_stats();
    web_sys::console::log_1(
        &format!(
            "cx-wasm: {fetched} shards ({cache_hits} cached, {} fetched) => {} records for {base}/{subdir} [cache: {total_indices} indices, {total_shards} shards]",
            fetched - cache_hits,
            all_records.len(),
        )
        .into(),
    );

    Ok(all_records)
}
