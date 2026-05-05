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

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::GenLock { check, root } => gen_lock(check, root),
    }
}
