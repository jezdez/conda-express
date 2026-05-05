use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use rattler_conda_types::Platform;
use rattler_lock::{LockFile, LockFileBuilder};
use sha2::{Digest, Sha256};

#[derive(Parser)]
#[command(name = "xtask", about = "Internal build tools for conda-express")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Extract cx.lock from pixi.lock's cx-env environment
    GenLock {
        /// Only verify cx.lock is up-to-date; exit 1 if stale
        #[arg(long)]
        check: bool,

        /// Project root (default: auto-detect from Cargo workspace)
        #[arg(long)]
        root: Option<PathBuf>,
    },

    /// Download packages from cx.lock and bundle into payload.tar.zst
    GenPayload {
        /// Target platform (default: current)
        #[arg(long)]
        platform: Option<String>,

        /// Project root (default: auto-detect from Cargo workspace)
        #[arg(long)]
        root: Option<PathBuf>,
    },
}

fn project_root(override_root: Option<&Path>) -> PathBuf {
    if let Some(root) = override_root {
        return root.to_path_buf();
    }
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .ancestors()
        .find(|p| p.join("pixi.toml").exists())
        .expect("could not find project root containing pixi.toml")
        .to_path_buf()
}

fn gen_lock(check: bool, root_override: Option<PathBuf>) {
    let root = project_root(root_override.as_deref());
    let pixi_lock_path = root.join("pixi.lock");
    let cx_lock_path = root.join("cx.lock");
    let cx_hash_path = root.join("cx.lock.hash");
    let pixi_toml_path = root.join("pixi.toml");

    let pixi_lock = LockFile::from_path(&pixi_lock_path)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", pixi_lock_path.display()));

    let cx_env = pixi_lock.environment("cx-env").unwrap_or_else(|| {
        panic!(
            "cx-env environment not found in {}",
            pixi_lock_path.display()
        )
    });

    let pixi_toml = std::fs::read_to_string(&pixi_toml_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", pixi_toml_path.display()));

    let input_hash = {
        let mut hasher = Sha256::new();
        hasher.update(pixi_toml.as_bytes());
        format!("{:x}", hasher.finalize())
    };

    if check {
        if !cx_lock_path.exists() {
            eprintln!("cx.lock does not exist; run `cargo xtask gen-lock` to create it");
            std::process::exit(1);
        }
        if !cx_hash_path.exists() {
            eprintln!("cx.lock.hash does not exist; run `cargo xtask gen-lock` to create it");
            std::process::exit(1);
        }
        let stored_hash = std::fs::read_to_string(&cx_hash_path).unwrap_or_default();
        if stored_hash.trim() != input_hash {
            eprintln!("cx.lock is stale (hash mismatch); run `cargo xtask gen-lock` to update");
            std::process::exit(1);
        }
        eprintln!("cx.lock is up-to-date");
        return;
    }

    let mut builder = LockFileBuilder::new();

    if !cx_env.channels().is_empty() {
        builder.set_channels("default", cx_env.channels().iter().cloned());
    }

    for (platform, packages) in cx_env.conda_packages_by_platform() {
        for pkg in packages {
            builder.add_conda_package("default", platform, pkg.clone());
        }
    }

    let new_lock = builder.finish();
    let new_content = new_lock
        .render_to_string()
        .expect("failed to render cx.lock");

    std::fs::write(&cx_lock_path, &new_content)
        .unwrap_or_else(|e| panic!("failed to write {}: {e}", cx_lock_path.display()));
    std::fs::write(&cx_hash_path, &input_hash)
        .unwrap_or_else(|e| panic!("failed to write {}: {e}", cx_hash_path.display()));

    let platforms: Vec<Platform> = cx_env.platforms().collect();
    let pkg_count: usize = cx_env
        .conda_packages_by_platform()
        .map(|(_, pkgs)| pkgs.count())
        .sum();
    eprintln!(
        "wrote cx.lock: {} packages across {} platforms",
        pkg_count,
        platforms.len()
    );
}

fn gen_payload(platform_str: Option<String>, root_override: Option<PathBuf>) {
    let root = project_root(root_override.as_deref());
    let cx_lock_path = root.join("cx.lock");
    let payload_path = root.join("payload.tar.zst");

    let platform = if let Some(ref s) = platform_str {
        s.parse::<Platform>()
            .unwrap_or_else(|_| panic!("invalid platform: {s}"))
    } else {
        Platform::current()
    };

    let lock_file = LockFile::from_path(&cx_lock_path)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", cx_lock_path.display()));

    let env = lock_file
        .default_environment()
        .unwrap_or_else(|| panic!("no default environment in {}", cx_lock_path.display()));

    let packages: Vec<_> = env
        .conda_packages_by_platform()
        .filter(|(p, _)| *p == platform)
        .flat_map(|(_, pkgs)| pkgs)
        .collect();

    if packages.is_empty() {
        panic!(
            "no packages for platform {platform} in {}",
            cx_lock_path.display()
        );
    }

    eprintln!(
        "downloading {} packages for {platform}...",
        packages.len()
    );

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to create tokio runtime");

    rt.block_on(download_and_bundle(&packages, &payload_path))
        .expect("failed to download/bundle payload");
}

async fn download_and_bundle(
    packages: &[&rattler_lock::CondaPackageData],
    payload_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder().no_gzip().build()?;

    let payload_dir = payload_path
        .parent()
        .expect("payload path has parent")
        .join("payload");
    std::fs::create_dir_all(&payload_dir)?;

    let start = std::time::Instant::now();

    for pkg in packages {
        let url = pkg.location().as_url().expect("package has URL");
        let archive_name = url
            .path_segments()
            .and_then(|mut s| s.next_back())
            .unwrap_or("unknown");

        let dest = payload_dir.join(archive_name);

        if dest.exists() {
            if let Some(ref expected) = pkg.record().sha256 {
                let data = std::fs::read(&dest)?;
                let actual = Sha256::digest(&data);
                if actual == *expected {
                    continue;
                }
                eprintln!("SHA256 mismatch for {archive_name}, re-downloading");
                std::fs::remove_file(&dest)?;
            } else {
                continue;
            }
        }

        let response = client
            .get(url.clone())
            .send()
            .await
            .map_err(|e| format!("failed to fetch {archive_name}: {e}"))?;

        let status = response.status();
        if !status.is_success() {
            return Err(format!("HTTP {status} fetching {archive_name}").into());
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| format!("failed to read {archive_name}: {e}"))?;

        if let Some(ref expected) = pkg.record().sha256 {
            let actual = Sha256::digest(&bytes);
            if actual != *expected {
                return Err(format!("SHA256 mismatch for {archive_name}").into());
            }
        }

        std::fs::write(&dest, &bytes)?;
    }

    eprintln!(
        "downloaded {} packages in {:.1}s, bundling...",
        packages.len(),
        start.elapsed().as_secs_f64()
    );

    let bundle_start = std::time::Instant::now();
    let out_file = std::fs::File::create(payload_path)?;
    let zstd_encoder = zstd::Encoder::new(out_file, 1)?;
    let mut tar_builder = tar::Builder::new(zstd_encoder);

    for entry in std::fs::read_dir(&payload_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let name = path.file_name().unwrap();
            tar_builder.append_path_with_name(&path, name)?;
        }
    }

    let zstd_encoder = tar_builder.into_inner()?;
    zstd_encoder.finish()?;

    let payload_size = std::fs::metadata(payload_path)?.len();
    eprintln!(
        "payload.tar.zst = {:.1} MB ({} packages, bundled in {:.1}s)",
        payload_size as f64 / 1_048_576.0,
        packages.len(),
        bundle_start.elapsed().as_secs_f64()
    );

    Ok(())
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::GenLock { check, root } => gen_lock(check, root),
        Command::GenPayload { platform, root } => gen_payload(platform, root),
    }
}
