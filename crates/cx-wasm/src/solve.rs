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
export interface VirtualPackageInput {
    name: string;
    version?: string;
    build_string?: string;
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

#[wasm_bindgen]
pub fn cx_solve_init() -> String {
    web_sys::console::log_1(&"cx-wasm solver initialized".into());
    format!("cx-wasm solver v{}", env!("CARGO_PKG_VERSION"))
}

#[derive(Deserialize)]
pub(crate) struct VirtualPackageInput {
    pub name: String,
    pub version: Option<String>,
    pub build_string: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct SolvedRecord {
    pub name: String,
    pub version: String,
    pub build: String,
    pub build_number: u64,
    pub subdir: String,
    pub url: String,
    pub channel: String,
    pub file_name: String,
    pub sha256: Option<String>,
    pub md5: Option<String>,
    pub size: Option<u64>,
    pub depends: Vec<String>,
    pub constrains: Vec<String>,
}

#[derive(Serialize)]
pub(crate) struct SolveSolution {
    pub records: Vec<SolvedRecord>,
    pub total_packages: usize,
}

pub(crate) fn parse_virtual_packages(
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

pub(crate) fn convert_solution(records: Vec<RepoDataRecord>) -> SolveSolution {
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

pub(crate) fn parse_channel_priority(s: Option<&str>) -> ChannelPriority {
    match s {
        Some("disabled") => ChannelPriority::Disabled,
        _ => ChannelPriority::Strict,
    }
}

pub(crate) fn parse_strategy(s: Option<&str>) -> SolveStrategy {
    match s {
        Some("lowest-version") => SolveStrategy::LowestVersion,
        Some("lowest-version-direct") => SolveStrategy::LowestVersionDirect,
        _ => SolveStrategy::Highest,
    }
}

/// Core solver: takes pre-parsed records and runs resolvo.
pub(crate) fn solve_with_records(
    available: Vec<Vec<RepoDataRecord>>,
    specs: Vec<MatchSpec>,
    locked_packages: Vec<RepoDataRecord>,
    virtual_packages: Vec<GenericVirtualPackage>,
    channel_priority: ChannelPriority,
    strategy: SolveStrategy,
) -> Result<SolveSolution, CxWasmError> {
    web_sys::console::log_1(
        &format!(
            "cx-wasm: solving {} specs against {} repodata sources ({} total records)",
            specs.len(),
            available.len(),
            available.iter().map(|r| r.len()).sum::<usize>()
        )
        .into(),
    );

    let available_packages: Vec<&Vec<RepoDataRecord>> = available.iter().collect();

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

    Ok(convert_solution(solved.records))
}

/// Merge default virtual packages with user-provided ones.
pub(crate) fn merge_virtual_packages(
    user_vpkgs: Option<&[VirtualPackageInput]>,
    platform_str: &str,
) -> Result<Vec<GenericVirtualPackage>, CxWasmError> {
    let defaults = default_virtual_packages(platform_str);

    match user_vpkgs {
        Some(vpkgs) if !vpkgs.is_empty() => {
            let mut merged = parse_virtual_packages(vpkgs)?;
            let names: std::collections::HashSet<String> =
                merged.iter().map(|v| v.name.as_normalized().to_string()).collect();
            for vp in defaults {
                if !names.contains(vp.name.as_normalized()) {
                    merged.push(vp);
                }
            }
            Ok(merged)
        }
        _ => Ok(defaults),
    }
}

fn embedded_emscripten_version() -> &'static str {
    #[cfg(cx_embedded_emscripten_version)]
    {
        include_str!(concat!(env!("OUT_DIR"), "/embedded_emscripten_version.txt"))
    }
    #[cfg(not(cx_embedded_emscripten_version))]
    {
        "3.1.58"
    }
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
                version: Version::from_str(embedded_emscripten_version()).unwrap(),
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
