use std::{env, fs, io::Read, path::PathBuf};

use sha2::{Digest, Sha256};

const BINARY_SHA256: &str = include_str!("binary-sha256.txt");

fn main() {
    println!("cargo:rerun-if-env-changed=CONDA_EXPRESS_BINARY_DIR");
    println!("cargo:rerun-if-env-changed=CONDA_EXPRESS_BINARY_SHA256");
    println!("cargo:rerun-if-env-changed=CONDA_EXPRESS_RELEASE_BASE_URL");

    let target = env::var("TARGET").expect("TARGET not set");
    let version = env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION not set");
    let asset_name = asset_name(&target);
    let expected_sha256 = expected_sha256(&version, &target);
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let binary_path = out_dir.join(if target.contains("windows") {
        "cx.exe"
    } else {
        "cx"
    });

    if let Ok(binary_dir) = env::var("CONDA_EXPRESS_BINARY_DIR") {
        let source = PathBuf::from(binary_dir).join(&asset_name);
        let bytes = fs::read(&source)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", source.display()));
        if let Some(expected) = expected_sha256.as_deref() {
            verify_expected_checksum(&asset_name, &bytes, expected);
        }
        fs::write(&binary_path, bytes)
            .unwrap_or_else(|e| panic!("failed to write {}: {e}", binary_path.display()));
    } else {
        let base_url = env::var("CONDA_EXPRESS_RELEASE_BASE_URL").ok();
        let base_url = base_url.clone().unwrap_or_else(|| {
            format!("https://github.com/jezdez/conda-express/releases/download/v{version}")
        });
        let binary_url = format!("{base_url}/{asset_name}");
        let bytes = download(&binary_url);
        if let Some(expected) = expected_sha256.as_deref() {
            verify_expected_checksum(&asset_name, &bytes, expected);
        } else {
            panic!(
                "missing trusted checksum for conda-express {version} {target}; \
                 update binary-sha256.txt or set CONDA_EXPRESS_BINARY_SHA256"
            );
        }
        fs::write(&binary_path, bytes)
            .unwrap_or_else(|e| panic!("failed to write {}: {e}", binary_path.display()));
    }

    println!(
        "cargo:rustc-env=CONDA_EXPRESS_EMBEDDED_BINARY={}",
        binary_path.display()
    );
    println!("cargo:rustc-env=CONDA_EXPRESS_TARGET={target}");
}

fn asset_name(target: &str) -> String {
    if target.contains("windows") {
        format!("cx-{target}.exe")
    } else {
        format!("cx-{target}")
    }
}

fn download(url: &str) -> Vec<u8> {
    let response = ureq::get(url)
        .call()
        .unwrap_or_else(|e| panic!("failed to download {url}: {e}"));
    let mut bytes = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut bytes)
        .unwrap_or_else(|e| panic!("failed to read {url}: {e}"));
    bytes
}

fn expected_sha256(version: &str, target: &str) -> Option<String> {
    if let Ok(value) = env::var("CONDA_EXPRESS_BINARY_SHA256") {
        return Some(value);
    }

    for line in BINARY_SHA256.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut fields = line.split_whitespace();
        let Some(line_version) = fields.next() else {
            continue;
        };
        let Some(line_target) = fields.next() else {
            panic!("invalid binary-sha256.txt line: {line}");
        };
        let Some(line_sha256) = fields.next() else {
            panic!("invalid binary-sha256.txt line: {line}");
        };
        if fields.next().is_some() {
            panic!("invalid binary-sha256.txt line: {line}");
        }
        if line_version == version && line_target == target {
            return Some(line_sha256.to_string());
        }
    }

    None
}

fn verify_expected_checksum(asset_name: &str, bytes: &[u8], expected: &str) {
    assert!(
        expected.len() == 64 && expected.chars().all(|c| c.is_ascii_hexdigit()),
        "invalid SHA256 checksum for {asset_name}: {expected}"
    );
    let actual = format!("{:x}", Sha256::digest(bytes));
    assert!(
        actual.eq_ignore_ascii_case(expected),
        "checksum mismatch for {asset_name}: expected {expected}, got {actual}"
    );
}
