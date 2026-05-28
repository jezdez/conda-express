use std::{env, fs, io::Read, path::PathBuf};

use sha2::{Digest, Sha256};

fn main() {
    println!("cargo:rerun-if-env-changed=CONDA_EXPRESS_BINARY_DIR");
    println!("cargo:rerun-if-env-changed=CONDA_EXPRESS_RELEASE_BASE_URL");

    let target = env::var("TARGET").expect("TARGET not set");
    let version = env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION not set");
    let asset_name = asset_name(&target);
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let binary_path = out_dir.join(if target.contains("windows") {
        "cx.exe"
    } else {
        "cx"
    });

    if let Ok(binary_dir) = env::var("CONDA_EXPRESS_BINARY_DIR") {
        let source = PathBuf::from(binary_dir).join(&asset_name);
        fs::copy(&source, &binary_path)
            .unwrap_or_else(|e| panic!("failed to copy {}: {e}", source.display()));
    } else {
        let base_url = env::var("CONDA_EXPRESS_RELEASE_BASE_URL").unwrap_or_else(|_| {
            format!("https://github.com/jezdez/conda-express/releases/download/v{version}")
        });
        let binary_url = format!("{base_url}/{asset_name}");
        let checksum_url = format!("{binary_url}.sha256");
        let bytes = download(&binary_url);
        let checksum = download_text(&checksum_url);
        verify_checksum(&asset_name, &bytes, &checksum);
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

fn download_text(url: &str) -> String {
    String::from_utf8(download(url)).unwrap_or_else(|e| panic!("invalid UTF-8 from {url}: {e}"))
}

fn verify_checksum(asset_name: &str, bytes: &[u8], checksum: &str) {
    let expected = checksum
        .split_whitespace()
        .next()
        .unwrap_or_else(|| panic!("empty checksum for {asset_name}"));
    let actual = format!("{:x}", Sha256::digest(bytes));
    assert!(
        actual.eq_ignore_ascii_case(expected),
        "checksum mismatch for {asset_name}: expected {expected}, got {actual}"
    );
}
