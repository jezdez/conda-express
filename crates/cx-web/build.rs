/// Build script for cx-web: optionally embeds a lockfile and target platform
/// into the WASM binary so JS consumers don't need to pass them at runtime.
///
/// Set `CX_LOCKFILE=/path/to/cx.lock` and `CX_PLATFORM=emscripten-wasm32`
/// at build time to bake them in.
fn main() {
    println!("cargo::rustc-check-cfg=cfg(cx_embedded_lockfile)");
    println!("cargo::rustc-check-cfg=cfg(cx_embedded_platform)");
    println!("cargo:rerun-if-env-changed=CX_LOCKFILE");
    println!("cargo:rerun-if-env-changed=CX_PLATFORM");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = std::path::Path::new(&out_dir);

    if let Ok(path) = std::env::var("CX_LOCKFILE") {
        if !path.is_empty() {
            let lockfile_path = std::path::Path::new(&path);
            let lockfile_path = if lockfile_path.is_absolute() {
                lockfile_path.to_path_buf()
            } else {
                let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
                std::path::Path::new(&manifest_dir).join(lockfile_path)
            };
            println!("cargo:rerun-if-changed={}", lockfile_path.display());
            let dest = out_path.join("embedded_lockfile.txt");
            std::fs::copy(&lockfile_path, &dest).unwrap_or_else(|e| {
                panic!(
                    "cx-web: failed to copy CX_LOCKFILE '{}': {e}",
                    lockfile_path.display()
                )
            });
            println!("cargo:rustc-cfg=cx_embedded_lockfile");
            eprintln!("cx-web: embedding lockfile from {}", lockfile_path.display());
        }
    }

    if let Ok(platform) = std::env::var("CX_PLATFORM") {
        if !platform.is_empty() {
            let dest = out_path.join("embedded_platform.txt");
            std::fs::write(&dest, &platform).unwrap_or_else(|e| {
                panic!("cx-web: failed to write embedded platform: {e}")
            });
            println!("cargo:rustc-cfg=cx_embedded_platform");
            eprintln!("cx-web: embedding platform '{platform}'");
        }
    }
}
