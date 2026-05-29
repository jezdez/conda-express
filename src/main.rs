use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process::Command,
};

use sha2::{Digest, Sha256};

const CX_BYTES: &[u8] = include_bytes!(env!("CONDA_EXPRESS_EMBEDDED_BINARY"));
const TARGET: &str = env!("CONDA_EXPRESS_TARGET");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let binary = ensure_binary().unwrap_or_else(|e| {
        eprintln!("failed to prepare embedded cx binary: {e}");
        std::process::exit(1);
    });

    let args: Vec<String> = env::args().skip(1).collect();

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let error = Command::new(&binary).args(&args).exec();
        eprintln!("failed to execute {}: {error}", binary.display());
        std::process::exit(1);
    }

    #[cfg(not(unix))]
    {
        let status = Command::new(&binary)
            .args(&args)
            .status()
            .unwrap_or_else(|e| {
                eprintln!("failed to execute {}: {e}", binary.display());
                std::process::exit(1);
            });
        std::process::exit(status.code().unwrap_or(1));
    }
}

fn ensure_binary() -> io::Result<PathBuf> {
    let binary = cache_dir().join(VERSION).join(TARGET).join(binary_name());
    if binary.exists() && cached_binary_matches(&binary)? {
        return Ok(binary);
    }

    if let Some(parent) = binary.parent() {
        fs::create_dir_all(parent)?;
    }

    let tmp = binary.with_extension("tmp");
    fs::write(&tmp, CX_BYTES)?;
    make_executable(&tmp)?;
    fs::rename(&tmp, &binary)?;
    Ok(binary)
}

fn cached_binary_matches(path: &Path) -> io::Result<bool> {
    if fs::metadata(path)?.len() != CX_BYTES.len() as u64 {
        return Ok(false);
    }
    let bytes = fs::read(path)?;
    Ok(Sha256::digest(&bytes).as_slice() == Sha256::digest(CX_BYTES).as_slice())
}

fn cache_dir() -> PathBuf {
    if let Some(path) = env::var_os("CONDA_EXPRESS_CARGO_CACHE") {
        return PathBuf::from(path);
    }
    if let Some(path) = env::var_os("XDG_CACHE_HOME") {
        return PathBuf::from(path).join("conda-express").join("cargo");
    }
    if cfg!(windows)
        && let Some(path) = env::var_os("LOCALAPPDATA")
    {
        return PathBuf::from(path).join("conda-express").join("cargo");
    }
    if let Some(path) = env::var_os("HOME") {
        return PathBuf::from(path)
            .join(".cache")
            .join("conda-express")
            .join("cargo");
    }
    env::temp_dir().join("conda-express").join("cargo")
}

fn binary_name() -> &'static str {
    if cfg!(windows) { "cx.exe" } else { "cx" }
}

#[cfg(unix)]
fn make_executable(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> io::Result<()> {
    Ok(())
}
