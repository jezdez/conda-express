//! Combined repodata fetch + solve entry point.
//!
//! [`cx_fetch_and_solve`] fetches repodata for all channel/subdir pairs, feeds
//! the parsed records directly into the solver, and returns a solution — all
//! without intermediate JSON serialization between Rust and JS/Python.

use std::collections::BTreeSet;

use rattler_conda_types::MatchSpec;
use serde::Deserialize;
use wasm_bindgen::prelude::*;

use crate::error::CxWasmError;
use crate::sharded::fetch_repodata_records;
use crate::solve::{
    SolveSolution, VirtualPackageInput, merge_virtual_packages, parse_channel_priority,
    parse_strategy, solve_with_records,
};

#[wasm_bindgen(typescript_custom_section)]
const TS_GATEWAY_TYPES: &str = r#"
export interface ChannelInput {
    url: string;
    subdirs: string[];
}

export interface FetchAndSolveRequest {
    channels: ChannelInput[];
    specs: string[];
    seed_names?: string[];
    installed?: InstalledRecord[];
    virtual_packages?: VirtualPackageInput[];
    platform?: string;
    channel_priority?: "strict" | "disabled";
    strategy?: "highest" | "lowest-version" | "lowest-version-direct";
}

export interface InstalledRecord {
    name: string;
    version: string;
    build: string;
    build_number?: number;
    subdir?: string;
    fn?: string;
    url?: string;
    channel?: string;
    depends?: string[];
    constrains?: string[];
    md5?: string;
    sha256?: string;
}
"#;

#[derive(Deserialize)]
struct ChannelInput {
    url: String,
    subdirs: Vec<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct InstalledRecord {
    name: String,
    version: String,
    build: String,
    #[serde(default)]
    build_number: u64,
    #[serde(default)]
    subdir: Option<String>,
    #[serde(rename = "fn")]
    #[serde(default)]
    fn_name: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    channel: Option<String>,
    #[serde(default)]
    depends: Vec<String>,
    #[serde(default)]
    constrains: Vec<String>,
    #[serde(default)]
    md5: Option<String>,
    #[serde(default)]
    sha256: Option<String>,
}

#[derive(Deserialize)]
struct FetchAndSolveRequest {
    channels: Vec<ChannelInput>,
    specs: Vec<String>,
    #[serde(default)]
    seed_names: Vec<String>,
    #[serde(default)]
    installed: Vec<InstalledRecord>,
    #[serde(default)]
    virtual_packages: Option<Vec<VirtualPackageInput>>,
    #[serde(default)]
    platform: Option<String>,
    #[serde(default)]
    channel_priority: Option<String>,
    #[serde(default)]
    strategy: Option<String>,
}

/// Fetch repodata for all channels/subdirs and solve in one call.
/// `fetch_binary`/`fetch_text` are JS sync XHR callbacks from the worker.
#[wasm_bindgen]
pub fn cx_fetch_and_solve(
    request: JsValue,
    fetch_binary: &js_sys::Function,
    fetch_text: &js_sys::Function,
) -> Result<JsValue, JsValue> {
    let request = if request.is_string() {
        let json_str = request.as_string().unwrap();
        js_sys::JSON::parse(&json_str)
            .map_err(|e| CxWasmError::InvalidInput(format!("JSON parse: {e:?}")))?
    } else {
        request
    };

    let req: FetchAndSolveRequest = serde_wasm_bindgen::from_value(request)
        .map_err(|e| CxWasmError::InvalidInput(format!("parsing fetch_and_solve request: {e}")))?;

    if req.channels.is_empty() {
        return Err(CxWasmError::InvalidInput("no channels provided".into()).into());
    }
    if req.specs.is_empty() {
        return Err(CxWasmError::InvalidInput("no specs provided".into()).into());
    }

    let specs: Vec<MatchSpec> = req
        .specs
        .iter()
        .map(|s| {
            MatchSpec::from_str(s, rattler_conda_types::ParseStrictness::Lenient)
                .map_err(|e| CxWasmError::SpecParse(format!("'{s}': {e}")))
        })
        .collect::<Result<_, _>>()?;

    let mut all_records = Vec::new();

    for ch in &req.channels {
        for sd in &ch.subdirs {
            match fetch_repodata_records(&ch.url, sd, &req.seed_names, fetch_binary, fetch_text) {
                Ok(recs) => {
                    web_sys::console::log_1(
                        &format!(
                            "cx-wasm: fetched {} records for {}/{}",
                            recs.len(),
                            ch.url,
                            sd,
                        )
                        .into(),
                    );
                    all_records.push(recs);
                }
                Err(e) => {
                    web_sys::console::warn_1(
                        &format!(
                            "cx-wasm: failed to fetch repodata for {}/{}: {}",
                            ch.url, sd, e,
                        )
                        .into(),
                    );
                }
            }
        }
    }

    if all_records.is_empty() {
        return Err(CxWasmError::RepodataParse(
            "could not fetch repodata from any channel/subdir".into(),
        )
        .into());
    }

    // Resolve cross-channel transitive dependencies.
    //
    // Channels form a unified priority list, but the per-channel shard
    // traversal only follows deps within its own shard index.  A package
    // on channel A may depend on a noarch package that only exists on
    // channel B.  After the initial pass we identify dependency names
    // that have no records yet and fetch their shards from all channels.
    const MAX_CROSS_CHANNEL_PASSES: usize = 5;
    for pass in 0..MAX_CROSS_CHANNEL_PASSES {
        let missing = compute_missing_deps(&all_records);
        if missing.is_empty() {
            break;
        }

        web_sys::console::log_1(
            &format!(
                "cx-wasm: cross-channel pass {}: resolving {} deps ({:?})",
                pass + 1,
                missing.len(),
                &missing[..missing.len().min(10)],
            )
            .into(),
        );

        let mut found_new = false;
        for ch in &req.channels {
            let base = ch.url.trim_end_matches('/');
            for sd in &ch.subdirs {
                let base_url = format!("{base}/{sd}/");
                match crate::sharded::fetch_sharded_records(
                    base,
                    sd,
                    &missing,
                    fetch_binary,
                    &base_url,
                ) {
                    Ok(recs) if !recs.is_empty() => {
                        web_sys::console::log_1(
                            &format!(
                                "cx-wasm: resolved {} cross-channel records from {}/{}",
                                recs.len(),
                                ch.url,
                                sd,
                            )
                            .into(),
                        );
                        all_records.push(recs);
                        found_new = true;
                    }
                    _ => {}
                }
            }
        }

        if !found_new {
            break;
        }
    }

    let locked_packages = parse_installed(&req.installed)?;

    let platform_str = req.platform.as_deref().unwrap_or("emscripten-wasm32");
    let virtual_packages = merge_virtual_packages(req.virtual_packages.as_deref(), platform_str)?;

    let channel_priority = parse_channel_priority(req.channel_priority.as_deref());
    let strategy = parse_strategy(req.strategy.as_deref());

    let solution = solve_with_records(
        all_records,
        specs,
        locked_packages,
        virtual_packages,
        channel_priority,
        strategy,
    )
    .map_err(|e| -> JsValue { e.into() })?;

    solve_to_js(&solution)
}

fn solve_to_js(solution: &SolveSolution) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(solution)
        .map_err(|e| CxWasmError::SerializeFailed(e.to_string()).into())
}

#[derive(Deserialize)]
struct GetShardUrlsRequest {
    channels: Vec<ChannelInput>,
    seeds: Vec<String>,
}

/// Compute shard URLs for a set of package names without fetching shard contents.
///
/// Fetches shard indices (one per channel/subdir, cached) and looks up each
/// seed name to produce the corresponding shard URL.  Returns a deduplicated
/// JSON array of URL strings suitable for parallel async prefetching from JS.
#[wasm_bindgen]
pub fn cx_get_shard_urls(
    request: &str,
    fetch_binary: &js_sys::Function,
) -> Result<JsValue, JsValue> {
    let req: GetShardUrlsRequest = serde_json::from_str(request)
        .map_err(|e| CxWasmError::InvalidInput(format!("parsing get_shard_urls request: {e}")))?;

    let mut urls: Vec<String> = Vec::new();

    for ch in &req.channels {
        let base = ch.url.trim_end_matches('/');
        for subdir in &ch.subdirs {
            let cache_key = format!("{base}/{subdir}");
            let index =
                match crate::sharded::get_or_fetch_index(&cache_key, base, subdir, fetch_binary) {
                    Ok(idx) => idx,
                    Err(e) => {
                        web_sys::console::log_1(
                            &format!("cx-wasm: no shard index for {cache_key}, skipping ({e})")
                                .into(),
                        );
                        continue;
                    }
                };

            let idx_url = crate::sharded::shard_index_url(base, subdir);
            let shards_base =
                crate::sharded::resolve_shards_base_url(&index.shards_base_url, &idx_url);

            for seed in &req.seeds {
                if let Some(hash) = index.shards.get(seed) {
                    urls.push(format!("{shards_base}{hash}.msgpack.zst"));
                }
            }
        }
    }

    urls.sort();
    urls.dedup();

    web_sys::console::log_1(
        &format!(
            "cx-wasm: computed {} shard URLs for {} seeds across {} channels",
            urls.len(),
            req.seeds.len(),
            req.channels.len(),
        )
        .into(),
    );

    serde_wasm_bindgen::to_value(&urls)
        .map_err(|e| CxWasmError::SerializeFailed(e.to_string()).into())
}

/// Find dependency names that appear in records' `depends` but have no
/// corresponding package records yet.  Virtual packages (names starting
/// with `__`) are excluded since they are provided by the runtime, not
/// by any channel.
fn compute_missing_deps(all_records: &[Vec<rattler_conda_types::RepoDataRecord>]) -> Vec<String> {
    let mut names_with_records: BTreeSet<String> = BTreeSet::new();
    let mut all_dep_names: BTreeSet<String> = BTreeSet::new();

    for recs in all_records {
        for rec in recs {
            names_with_records.insert(rec.package_record.name.as_normalized().to_string());
            for dep in &rec.package_record.depends {
                if let Some(name) = dep.split_whitespace().next() {
                    all_dep_names.insert(name.to_string());
                }
            }
        }
    }

    all_dep_names
        .difference(&names_with_records)
        .filter(|name| !name.starts_with("__"))
        .cloned()
        .collect()
}

fn parse_installed(
    records: &[InstalledRecord],
) -> Result<Vec<rattler_conda_types::RepoDataRecord>, CxWasmError> {
    use rattler_conda_types::{PackageName, Version};
    use std::str::FromStr;

    records
        .iter()
        .map(|r| {
            let subdir = r.subdir.as_deref().unwrap_or("noarch");
            let fn_name = r
                .fn_name
                .clone()
                .unwrap_or_else(|| format!("{}-{}-{}.conda", r.name, r.version, r.build));
            let channel = r
                .channel
                .as_deref()
                .unwrap_or("https://conda.anaconda.org/unknown");
            let url_str = r
                .url
                .clone()
                .unwrap_or_else(|| format!("{}/{}/{}", channel, subdir, fn_name));
            let url = url::Url::parse(&url_str).map_err(|e| {
                CxWasmError::PackageParse(format!("invalid URL for {}: {e}", r.name))
            })?;

            let name = PackageName::from_str(&r.name).map_err(|e| {
                CxWasmError::PackageParse(format!("package name '{}': {e}", r.name))
            })?;
            let version = Version::from_str(&r.version)
                .map_err(|e| CxWasmError::PackageParse(format!("version '{}': {e}", r.version)))?;

            let mut pkg = rattler_conda_types::PackageRecord::new(name, version, r.build.clone());
            pkg.build_number = r.build_number;
            pkg.subdir = subdir.to_string();
            pkg.depends = r.depends.clone();
            pkg.constrains = r.constrains.clone();

            Ok(rattler_conda_types::RepoDataRecord {
                package_record: pkg,
                identifier: fn_name.parse().map_err(|e| {
                    CxWasmError::PackageParse(format!("identifier for {}: {e}", r.name))
                })?,
                url,
                channel: Some(channel.to_string()),
            })
        })
        .collect()
}
