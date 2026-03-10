use std::str::FromStr;

use rattler_conda_types::{
    GenericVirtualPackage, MatchSpec, PackageName, Platform, RepoDataRecord, Version,
};
use rattler_solve::{ChannelPriority, SolveStrategy, SolverImpl, SolverTask, resolvo};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::error::CxWasmError;

#[wasm_bindgen(typescript_custom_section)]
const TS_SOLVE_TYPES: &str = r#"
export interface RepodataInput {
    channel: string;
    subdir: string;
    repodata: string;
}

export interface VirtualPackageInput {
    name: string;
    version?: string;
    build_string?: string;
}

export interface SolveRequest {
    repodata: RepodataInput[];
    specs: string[];
    installed?: string;
    virtual_packages?: VirtualPackageInput[];
    platform?: string;
    channel_priority?: "strict" | "disabled";
    strategy?: "highest" | "lowest-version" | "lowest-version-direct";
}

export interface SolvedRecord {
    name: string;
    version: string;
    build: string;
    build_number: number;
    subdir: string;
    url: string;
    channel: string;
    file_name: string;
    sha256?: string;
    md5?: string;
    size?: number;
    depends: string[];
    constrains: string[];
}

export interface SolveSolution {
    records: SolvedRecord[];
    total_packages: number;
}
"#;

fn solve_to_js<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value)
        .map_err(|e| CxWasmError::SerializeFailed(e.to_string()).into())
}

#[wasm_bindgen]
pub fn cx_solve_init() -> String {
    web_sys::console::log_1(&"cx-wasm solver initialized".into());
    format!("cx-wasm solver v{}", env!("CARGO_PKG_VERSION"))
}

#[derive(Deserialize)]
struct RepodataInput {
    channel: String,
    subdir: String,
    repodata: String,
}

#[derive(Deserialize)]
struct VirtualPackageInput {
    name: String,
    version: Option<String>,
    build_string: Option<String>,
}

#[derive(Deserialize)]
struct SolveRequest {
    repodata: Vec<RepodataInput>,
    specs: Vec<String>,
    #[serde(default)]
    installed: Option<String>,
    #[serde(default)]
    virtual_packages: Option<Vec<VirtualPackageInput>>,
    #[serde(default)]
    platform: Option<String>,
    #[serde(default)]
    channel_priority: Option<String>,
    #[serde(default)]
    strategy: Option<String>,
}

#[derive(Serialize)]
struct SolvedRecord {
    name: String,
    version: String,
    build: String,
    build_number: u64,
    subdir: String,
    url: String,
    channel: String,
    file_name: String,
    sha256: Option<String>,
    md5: Option<String>,
    size: Option<u64>,
    depends: Vec<String>,
    constrains: Vec<String>,
}

#[derive(Serialize)]
struct SolveSolution {
    records: Vec<SolvedRecord>,
    total_packages: usize,
}

fn parse_repodata_records(
    entries: &[RepodataInput],
) -> Result<Vec<Vec<RepoDataRecord>>, CxWasmError> {
    let mut all_records = Vec::new();

    for entry in entries {
        let repo: rattler_conda_types::RepoData =
            serde_json::from_str(&entry.repodata).map_err(|e| {
                CxWasmError::RepodataParse(format!(
                    "parsing repodata for {}/{}: {e}",
                    entry.channel, entry.subdir
                ))
            })?;

        let base_url = format!(
            "{}/{}/",
            entry.channel.trim_end_matches('/'),
            entry.subdir
        );

        let mut records = Vec::new();

        let all_packages = repo
            .packages
            .into_iter()
            .chain(repo.conda_packages.into_iter());

        for (identifier, pkg) in all_packages {
            let url = url::Url::parse(&format!("{base_url}{identifier}")).map_err(|e| {
                CxWasmError::RepodataParse(format!("invalid URL for {identifier}: {e}"))
            })?;

            records.push(RepoDataRecord {
                package_record: pkg,
                identifier,
                url,
                channel: Some(entry.channel.clone()),
            });
        }

        web_sys::console::log_1(
            &format!(
                "cx-wasm: loaded {} records from {}/{}",
                records.len(),
                entry.channel,
                entry.subdir
            )
            .into(),
        );

        all_records.push(records);
    }

    Ok(all_records)
}

fn parse_installed_records(json: &str) -> Result<Vec<RepoDataRecord>, CxWasmError> {
    serde_json::from_str(json)
        .map_err(|e| CxWasmError::PackageParse(format!("parsing installed records: {e}")))
}

fn parse_virtual_packages(
    vpkgs: &[VirtualPackageInput],
) -> Result<Vec<GenericVirtualPackage>, CxWasmError> {
    vpkgs
        .iter()
        .map(|vp| {
            let name = PackageName::from_str(&vp.name).map_err(|e| {
                CxWasmError::PackageParse(format!("virtual package name '{}': {e}", vp.name))
            })?;
            let version = match &vp.version {
                Some(v) => Version::from_str(v).map_err(|e| {
                    CxWasmError::PackageParse(format!("virtual package version '{v}': {e}"))
                })?,
                None => Version::from_str("0").unwrap(),
            };
            let build_string = vp.build_string.clone().unwrap_or_default();
            Ok(GenericVirtualPackage {
                name,
                version,
                build_string,
            })
        })
        .collect()
}

fn convert_solution(records: Vec<RepoDataRecord>) -> SolveSolution {
    let total = records.len();
    let solved: Vec<SolvedRecord> = records
        .into_iter()
        .map(|r| {
            let pr = &r.package_record;
            SolvedRecord {
                name: pr.name.as_normalized().to_string(),
                version: pr.version.to_string(),
                build: pr.build.clone(),
                build_number: pr.build_number,
                subdir: pr.subdir.clone(),
                url: r.url.to_string(),
                channel: r.channel.unwrap_or_default(),
                file_name: r.identifier.to_string(),
                sha256: pr.sha256.map(|h| format!("{h:x}")),
                md5: pr.md5.map(|h| format!("{h:x}")),
                size: pr.size,
                depends: pr.depends.clone(),
                constrains: pr.constrains.clone(),
            }
        })
        .collect();

    SolveSolution {
        records: solved,
        total_packages: total,
    }
}

/// Solve conda dependencies given repodata, specs, and environment state.
///
/// Accepts a `SolveRequest` JS object and returns a `SolveSolution`.
#[wasm_bindgen]
pub fn cx_solve(request: JsValue) -> Result<JsValue, JsValue> {
    let req: SolveRequest = serde_wasm_bindgen::from_value(request)
        .map_err(|e| CxWasmError::InvalidInput(format!("parsing solve request: {e}")))?;

    if req.repodata.is_empty() {
        return Err(CxWasmError::InvalidInput("no repodata provided".into()).into());
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

    let repodata_records = parse_repodata_records(&req.repodata)?;

    let locked_packages = match &req.installed {
        Some(json) if !json.is_empty() => parse_installed_records(json)?,
        _ => Vec::new(),
    };

    let platform_str = req.platform.as_deref().unwrap_or("emscripten-wasm32");
    let defaults = default_virtual_packages(platform_str);

    let virtual_packages = match &req.virtual_packages {
        Some(vpkgs) if !vpkgs.is_empty() => {
            let mut merged = parse_virtual_packages(vpkgs)?;
            let names: std::collections::HashSet<String> =
                merged.iter().map(|v| v.name.as_normalized().to_string()).collect();
            for vp in defaults {
                if !names.contains(vp.name.as_normalized()) {
                    merged.push(vp);
                }
            }
            merged
        }
        _ => defaults,
    };

    let channel_priority = match req.channel_priority.as_deref() {
        Some("disabled") => ChannelPriority::Disabled,
        _ => ChannelPriority::Strict,
    };

    let strategy = match req.strategy.as_deref() {
        Some("lowest-version") => SolveStrategy::LowestVersion,
        Some("lowest-version-direct") => SolveStrategy::LowestVersionDirect,
        _ => SolveStrategy::Highest,
    };

    web_sys::console::log_1(
        &format!(
            "cx-wasm: solving {} specs against {} repodata sources ({} total records)",
            specs.len(),
            repodata_records.len(),
            repodata_records.iter().map(|r| r.len()).sum::<usize>()
        )
        .into(),
    );

    let available_packages: Vec<&Vec<RepoDataRecord>> =
        repodata_records.iter().collect();

    let solver_task = SolverTask {
        available_packages,
        locked_packages,
        pinned_packages: Vec::new(),
        virtual_packages,
        specs,
        constraints: Vec::new(),
        timeout: None,
        channel_priority,
        exclude_newer: None,
        min_age: None,
        strategy,
    };

    let solved = resolvo::Solver
        .solve(solver_task)
        .map_err(|e| CxWasmError::SolveFailed(format!("{e}")))?;

    web_sys::console::log_1(
        &format!("cx-wasm: solved — {} packages", solved.records.len()).into(),
    );

    let solution = convert_solution(solved.records);
    solve_to_js(&solution)
}

pub(crate) fn default_virtual_packages(platform_str: &str) -> Vec<GenericVirtualPackage> {
    let mut vpkgs = Vec::new();

    if let Ok(platform) = Platform::from_str(platform_str) {
        vpkgs.push(GenericVirtualPackage {
            name: PackageName::from_str("__unix").unwrap(),
            version: Version::from_str("0").unwrap(),
            build_string: String::new(),
        });

        if platform == Platform::EmscriptenWasm32 {
            vpkgs.push(GenericVirtualPackage {
                name: PackageName::from_str("__emscripten").unwrap(),
                version: Version::from_str("3.1.58").unwrap(),
                build_string: String::new(),
            });
        }
    }

    vpkgs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_specs() {
        let spec =
            MatchSpec::from_str("numpy >=1.24", rattler_conda_types::ParseStrictness::Lenient)
                .unwrap();
        assert!(spec.name.is_some());
        assert!(format!("{}", spec).contains("numpy"));
    }

    #[test]
    fn test_default_virtual_packages() {
        let vpkgs = default_virtual_packages("emscripten-wasm32");
        assert!(vpkgs.len() >= 2);
        assert!(vpkgs
            .iter()
            .any(|v| v.name.as_normalized() == "__emscripten"));
    }
}
