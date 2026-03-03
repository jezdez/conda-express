# Changelog

## 0.1.2

First published release.

### Features

- Single-binary conda bootstrapper powered by rattler
- Compile-time lockfile (rattler-lock v6) for deterministic bootstraps
- Post-solve exclusion of `conda-libmamba-solver` and 27 native dependencies
- conda-rattler-solver as default solver
- conda-spawn activation model with `cx shell` alias
- Disabled `activate`, `deactivate`, and `init` commands
- Auto-bootstrap on first conda command
- CEP 22 frozen base prefix protection
- PyPI distribution via maturin platform wheels (`pip install conda-express`)
- crates.io distribution (`cargo install conda-express`)
- Trusted publishing (OIDC) for both PyPI and crates.io
- Sphinx documentation with conda-sphinx-theme, published to GitHub Pages

### CI/CD

- Multi-platform builds: linux-x64, linux-aarch64, macos-x64, macos-arm64, windows-x64
- All GitHub Actions pinned to commit SHAs
- Swatinem/rust-cache for faster CI builds
- Checked-in `cx.lock` eliminates network solve in CI
- Thin LTO and parallel codegen for faster release builds
- GitHub Pages deployment for documentation
