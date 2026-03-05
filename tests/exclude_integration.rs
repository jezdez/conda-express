//! Integration tests for the exclude filter using the real embedded lockfile.

use std::str::FromStr;

use conda_express::exclude::{filter_excluded_packages, sorted_names};
use rattler_conda_types::{Platform, RepoDataRecord};
use rattler_lock::LockFile;

const EMBEDDED_LOCK: &str = include_str!(concat!(env!("OUT_DIR"), "/cx.lock"));

fn records_from_embedded_lock() -> Vec<RepoDataRecord> {
    let lock_file = LockFile::from_str(EMBEDDED_LOCK).expect("failed to parse embedded lockfile");
    let env = lock_file
        .default_environment()
        .expect("no default environment");
    let platform = Platform::current();
    env.conda_repodata_records(platform)
        .expect("failed to extract records")
        .expect("no records for current platform")
}

#[test]
fn test_embedded_lockfile_package_composition() {
    let records = records_from_embedded_lock();
    let names = sorted_names(&records);

    let excluded = ["conda-libmamba-solver", "libmamba", "libsolv"];
    for pkg in &excluded {
        assert!(
            !names.contains(&pkg.to_string()),
            "embedded lockfile should not contain {pkg} (pre-filtered by build.rs)"
        );
    }

    let required = [
        "conda",
        "conda-rattler-solver",
        "conda-spawn",
        "conda-pypi",
        "conda-self",
    ];
    for pkg in &required {
        assert!(
            names.contains(&pkg.to_string()),
            "embedded lockfile should contain {pkg}"
        );
    }
    assert!(
        names.iter().any(|n| n.starts_with("python")),
        "embedded lockfile should contain python"
    );
}

#[test]
fn test_filter_noop_on_already_filtered() {
    let records = records_from_embedded_lock();
    let original_count = records.len();
    let excludes = vec!["conda-libmamba-solver".to_string()];

    let (filtered, removed) = filter_excluded_packages(records, &excludes);

    assert!(
        removed.is_empty(),
        "nothing to remove from already-filtered lockfile"
    );
    assert_eq!(filtered.len(), original_count);
}
