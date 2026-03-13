//! Combined repodata fetch + solve entry point.
//!
//! [`cx_fetch_and_solve`] fetches repodata for all channel/subdir pairs, feeds
//! the parsed records directly into the solver, and returns a solution — all
//! without intermediate JSON serialization between Rust and JS/Python.

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
///
/// This eliminates the JSON serialization roundtrip between Rust and
/// JS/Python: repodata is fetched via JS callbacks, decoded directly into
/// `RepoDataRecord`s, and passed straight to the solver.
///
/// `fetch_binary` and `fetch_text` are JS functions for synchronous HTTP
/// requests (sync XHR in the Web Worker).
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

    for channel in &req.channels {
        for subdir in &channel.subdirs {
            match fetch_repodata_records(
                &channel.url,
                subdir,
                &req.seed_names,
                fetch_binary,
                fetch_text,
            ) {
                Ok(records) => {
                    web_sys::console::log_1(
                        &format!(
                            "cx-wasm: fetched {} records for {}/{}",
                            records.len(),
                            channel.url,
                            subdir,
                        )
                        .into(),
                    );
                    all_records.push(records);
                }
                Err(e) => {
                    web_sys::console::warn_1(
                        &format!(
                            "cx-wasm: failed to fetch repodata for {}/{}: {}",
                            channel.url, subdir, e,
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

    let locked_packages = parse_installed(&req.installed)?;

    let platform_str = req.platform.as_deref().unwrap_or("emscripten-wasm32");
    let virtual_packages = merge_virtual_packages(
        req.virtual_packages.as_deref(),
        platform_str,
    )?;

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

fn parse_installed(
    records: &[InstalledRecord],
) -> Result<Vec<rattler_conda_types::RepoDataRecord>, CxWasmError> {
    use std::str::FromStr;
    use rattler_conda_types::{PackageName, Version};

    records
        .iter()
        .map(|r| {
            let subdir = r.subdir.as_deref().unwrap_or("noarch");
            let fn_name = r.fn_name.clone().unwrap_or_else(|| {
                format!("{}-{}-{}.conda", r.name, r.version, r.build)
            });
            let channel = r.channel.as_deref().unwrap_or("https://conda.anaconda.org/unknown");
            let url_str = r.url.clone().unwrap_or_else(|| {
                format!("{}/{}/{}", channel, subdir, fn_name)
            });
            let url = url::Url::parse(&url_str).map_err(|e| {
                CxWasmError::PackageParse(format!("invalid URL for {}: {e}", r.name))
            })?;

            let name = PackageName::from_str(&r.name).map_err(|e| {
                CxWasmError::PackageParse(format!("package name '{}': {e}", r.name))
            })?;
            let version = Version::from_str(&r.version).map_err(|e| {
                CxWasmError::PackageParse(format!("version '{}': {e}", r.version))
            })?;

            let mut pkg = rattler_conda_types::PackageRecord::new(
                name,
                version,
                r.build.clone(),
            );
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
