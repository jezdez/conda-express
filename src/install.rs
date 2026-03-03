//! Package installation — lockfile fast-path and live-solve fallback.

use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    env,
    future::IntoFuture,
    path::Path,
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};

use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use miette::{Context, IntoDiagnostic};
use rattler::{
    default_cache_dir,
    install::{IndicatifReporter, Installer},
    package_cache::PackageCache,
};
use rattler_conda_types::{
    Channel, ChannelConfig, GenericVirtualPackage, MatchSpec, PackageName, ParseMatchSpecOptions,
    Platform, PrefixRecord, RepoDataRecord,
};
use rattler_lock::LockFile;
use rattler_networking::AuthenticationMiddleware;
use rattler_repodata_gateway::{Gateway, RepoData, SourceConfig};
use rattler_solve::{SolverImpl, SolverTask, resolvo};

use crate::config;

// ─── Shared progress bar ─────────────────────────────────────────────────────

static GLOBAL_MP: std::sync::LazyLock<MultiProgress> = std::sync::LazyLock::new(|| {
    let mp = MultiProgress::new();
    mp.set_draw_target(ProgressDrawTarget::stderr_with_hz(20));
    mp
});

fn multi_progress() -> MultiProgress {
    GLOBAL_MP.clone()
}

// ─── Install from lockfile (fast path) ───────────────────────────────────────

/// Install packages from a pre-solved lockfile.
///
/// Skips repodata fetching and solving entirely — parses the lockfile, extracts
/// records for the current platform, applies exclusions, and passes straight to
/// the installer.
pub async fn from_lockfile(
    prefix: &Path,
    lock_content: &str,
    excludes: &[String],
) -> miette::Result<()> {
    let lock_file = LockFile::from_str(lock_content)
        .into_diagnostic()
        .context("failed to parse lockfile")?;

    let env = lock_file
        .default_environment()
        .ok_or_else(|| miette::miette!("lockfile has no default environment"))?;

    let platform = Platform::current();
    let records = env
        .conda_repodata_records(platform)
        .into_diagnostic()
        .context("failed to extract records from lockfile")?
        .ok_or_else(|| miette::miette!("lockfile has no records for platform {}", platform))?;

    eprintln!(
        "   Lockfile contains {} packages for {}",
        records.len(),
        platform
    );

    let required_packages = apply_excludes(records, excludes);

    let cfg = config::embedded_config();
    let match_specs = parse_specs(&cfg.packages)?;
    let installed = PrefixRecord::collect_from_prefix::<PrefixRecord>(prefix).into_diagnostic()?;
    let client = make_download_client()?;

    run_installer(
        prefix,
        platform,
        &installed,
        &match_specs,
        client,
        required_packages,
    )
    .await
}

// ─── Install via live solve ──────────────────────────────────────────────────

/// Fetch repodata, solve, and install packages into the prefix.
pub async fn from_solve(
    prefix: &Path,
    channels: &[String],
    specs: &[String],
    excludes: &[String],
) -> miette::Result<()> {
    let channel_config =
        ChannelConfig::default_with_root_dir(env::current_dir().into_diagnostic()?);
    let platform = Platform::current();
    let match_specs = parse_specs(specs)?;

    let cache_dir = default_cache_dir()
        .map_err(|e| miette::miette!("could not determine cache directory: {}", e))?;
    rattler_cache::ensure_cache_dir(&cache_dir)
        .map_err(|e| miette::miette!("could not create cache directory: {}", e))?;

    let parsed_channels: Vec<Channel> = channels
        .iter()
        .map(|c| Channel::from_str(c, &channel_config))
        .collect::<Result<Vec<_>, _>>()
        .into_diagnostic()?;

    let installed = PrefixRecord::collect_from_prefix::<PrefixRecord>(prefix).into_diagnostic()?;
    let client = make_download_client()?;

    let gateway = Gateway::builder()
        .with_cache_dir(cache_dir.join(rattler_cache::REPODATA_CACHE_DIR))
        .with_package_cache(PackageCache::new(
            cache_dir.join(rattler_cache::PACKAGE_CACHE_DIR),
        ))
        .with_client(client.clone())
        .with_channel_config(rattler_repodata_gateway::ChannelConfig {
            default: SourceConfig {
                sharded_enabled: true,
                ..SourceConfig::default()
            },
            per_channel: HashMap::new(),
        })
        .finish();

    let start = Instant::now();
    let repo_data = wrap_async_spinner(
        "fetching repodata",
        gateway
            .query(
                parsed_channels,
                [platform, Platform::NoArch],
                match_specs.clone(),
            )
            .recursive(true),
    )
    .await
    .into_diagnostic()
    .context("failed to load repodata")?;

    let total_records: usize = repo_data.iter().map(RepoData::len).sum();
    eprintln!(
        "   Loaded {} records in {:.1}s",
        total_records,
        start.elapsed().as_secs_f64()
    );

    let virtual_packages = rattler_virtual_packages::VirtualPackage::detect(
        &rattler_virtual_packages::VirtualPackageOverrides::default(),
    )
    .map(|vpkgs| {
        vpkgs
            .iter()
            .map(|vpkg| GenericVirtualPackage::from(vpkg.clone()))
            .collect::<Vec<_>>()
    })
    .into_diagnostic()?;

    let locked_packages = installed
        .iter()
        .map(|record| record.repodata_record.clone())
        .collect();

    let solver_task = SolverTask {
        locked_packages,
        virtual_packages,
        specs: match_specs.clone(),
        ..SolverTask::from_iter(&repo_data)
    };

    let solved = wrap_spinner("solving environment", move || {
        resolvo::Solver.solve(solver_task)
    })
    .into_diagnostic()
    .context("failed to solve environment")?
    .records;

    let required_packages = apply_excludes(solved, excludes);

    run_installer(
        prefix,
        platform,
        &installed,
        &match_specs,
        client,
        required_packages,
    )
    .await
}

// ─── Shared helpers ──────────────────────────────────────────────────────────

fn parse_specs(specs: &[String]) -> miette::Result<Vec<MatchSpec>> {
    specs
        .iter()
        .map(|s| MatchSpec::from_str(s, ParseMatchSpecOptions::default()))
        .collect::<Result<Vec<_>, _>>()
        .into_diagnostic()
        .context("failed to parse package specs")
}

fn make_download_client() -> miette::Result<reqwest_middleware::ClientWithMiddleware> {
    let raw = reqwest::Client::builder()
        .no_gzip()
        .build()
        .expect("failed to create HTTP client");

    Ok(reqwest_middleware::ClientBuilder::new(raw.clone())
        .with_arc(Arc::new(
            AuthenticationMiddleware::from_env_and_defaults().into_diagnostic()?,
        ))
        .with(rattler_networking::OciMiddleware::new(raw))
        .build())
}

async fn run_installer(
    prefix: &Path,
    platform: Platform,
    installed: &[PrefixRecord],
    specs: &[MatchSpec],
    client: reqwest_middleware::ClientWithMiddleware,
    packages: Vec<RepoDataRecord>,
) -> miette::Result<()> {
    let start = Instant::now();
    let result = Installer::new()
        .with_download_client(client)
        .with_target_platform(platform)
        .with_installed_packages(installed.to_vec())
        .with_execute_link_scripts(true)
        .with_requested_specs(specs.to_vec())
        .with_reporter(
            IndicatifReporter::builder()
                .with_multi_progress(multi_progress())
                .finish(),
        )
        .install(prefix, packages)
        .await
        .into_diagnostic()
        .context("failed to install packages")?;

    if result.transaction.operations.is_empty() {
        eprintln!("   {} Already up to date", console::style("✔").green());
    } else {
        eprintln!(
            "   Installed {} packages in {:.1}s",
            result.transaction.operations.len(),
            start.elapsed().as_secs_f64()
        );
    }
    Ok(())
}

fn apply_excludes(packages: Vec<RepoDataRecord>, excludes: &[String]) -> Vec<RepoDataRecord> {
    if excludes.is_empty() {
        return packages;
    }
    let (filtered, removed) = filter_excluded_packages(packages, excludes);
    if !removed.is_empty() {
        eprintln!(
            "   Excluded {} packages ({})",
            removed.len(),
            removed.join(", ")
        );
    }
    filtered
}

// ─── Post-solve exclusion filter ─────────────────────────────────────────────

/// Remove explicitly excluded packages and any of their dependencies that are
/// not required by any remaining package.
///
/// Walks the reverse-dependency graph: starting from the excluded set, it
/// transitively removes dependencies whose *every* dependent has already been
/// removed.
fn filter_excluded_packages(
    packages: Vec<RepoDataRecord>,
    excludes: &[String],
) -> (Vec<RepoDataRecord>, Vec<String>) {
    let exclude_set: HashSet<&str> = excludes.iter().map(|s| s.as_str()).collect();

    let name_of = |r: &RepoDataRecord| r.package_record.name.as_normalized().to_string();
    let pkg_names: Vec<String> = packages.iter().map(name_of).collect();
    let name_to_idx: HashMap<&str, usize> = pkg_names
        .iter()
        .enumerate()
        .map(|(i, n)| (n.as_str(), i))
        .collect();

    let n = packages.len();
    let mut reverse_deps: Vec<HashSet<usize>> = vec![HashSet::new(); n];
    for (i, rec) in packages.iter().enumerate() {
        for dep_str in &rec.package_record.depends {
            let dep_name = PackageName::from_matchspec_str_unchecked(dep_str);
            if let Some(&dep_idx) = name_to_idx.get(dep_name.as_normalized()) {
                reverse_deps[dep_idx].insert(i);
            }
        }
    }

    let mut removed: HashSet<usize> = HashSet::new();
    let mut queue: Vec<usize> = Vec::new();
    for (i, name) in pkg_names.iter().enumerate() {
        if exclude_set.contains(name.as_str()) {
            removed.insert(i);
            queue.push(i);
        }
    }

    while let Some(pkg_idx) = queue.pop() {
        for dep_str in &packages[pkg_idx].package_record.depends {
            let dep_name = PackageName::from_matchspec_str_unchecked(dep_str);
            if let Some(&dep_idx) = name_to_idx.get(dep_name.as_normalized()) {
                if removed.contains(&dep_idx) {
                    continue;
                }
                let all_dependents_removed = reverse_deps[dep_idx]
                    .iter()
                    .all(|rdep| removed.contains(rdep));
                if all_dependents_removed {
                    removed.insert(dep_idx);
                    queue.push(dep_idx);
                }
            }
        }
    }

    let removed_names: Vec<String> = removed
        .iter()
        .map(|&i| pkg_names[i].clone())
        .collect::<Vec<_>>();

    let filtered: Vec<RepoDataRecord> = packages
        .into_iter()
        .enumerate()
        .filter(|(i, _)| !removed.contains(i))
        .map(|(_, r)| r)
        .collect();

    let mut sorted_names = removed_names;
    sorted_names.sort();
    (filtered, sorted_names)
}

// ─── Progress spinners ───────────────────────────────────────────────────────

fn wrap_spinner<T, F: FnOnce() -> T>(msg: impl Into<Cow<'static, str>>, func: F) -> T {
    let pb = multi_progress().add(ProgressBar::new_spinner());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_style(ProgressStyle::with_template("   {spinner:.green} {msg}").unwrap());
    pb.set_message(msg);
    let result = func();
    pb.finish_and_clear();
    result
}

async fn wrap_async_spinner<T, F: IntoFuture<Output = T>>(
    msg: impl Into<Cow<'static, str>>,
    fut: F,
) -> T {
    let pb = multi_progress().add(ProgressBar::new_spinner());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_style(ProgressStyle::with_template("   {spinner:.green} {msg}").unwrap());
    pb.set_message(msg);
    let result = fut.into_future().await;
    pb.finish_and_clear();
    result
}
