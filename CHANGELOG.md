# Changelog

## Unreleased

## 0.2.0

### Features

- Add `cx uninstall` subcommand — removes the conda prefix, all named
  environments, the cx binary, and PATH entries from shell profiles. Requires
  interactive confirmation (or `--yes` to skip).
- Add reusable GitHub Action (`action.yml`) for building custom cx binaries
  with configurable packages. Use `uses: jezdez/conda-express@main` with
  `packages`, `channels`, and `exclude` inputs.
- Add reusable workflow (`.github/workflows/build.yml`) that builds custom
  cx binaries for all 5 platforms via `workflow_call`.
- Support build-time environment variable overrides (`CX_PACKAGES`,
  `CX_CHANNELS`, `CX_EXCLUDE`) in `build.rs` for custom builds without
  editing `pixi.toml`.
- Add Homebrew formula (`Formula/cx.rb`) as a same-repo tap. Install with
  `brew tap jezdez/conda-express https://github.com/jezdez/conda-express`
  followed by `brew install jezdez/conda-express/cx`.
- Homebrew is now the recommended installation method for macOS and Linux;
  shell scripts are repositioned as an alternative for CI and Windows.
- Release workflow automatically updates the Homebrew formula with new
  version and checksums on each tag push.

### Fixes

- Fix Homebrew formula update in release workflow — shell variables were
  not expanded inside the single-quoted Python heredoc, resulting in
  placeholder checksums.

### Documentation

- Restructure docs following the Diataxis framework (tutorials, how-to
  guides, reference, explanation).
- Add GitHub Action reference page (`docs/reference/github-action.md`).
- Add custom builds how-to guide (`docs/guides/custom-builds.md`).
- Add sphinx-design enhancements: badges on Action inputs, dropdowns for
  CLI output examples, octicon icons on landing page cards.
- Consolidate installation tabs (merge PyPI and crates.io into one tab).

## 0.1.7

### Internal

- Fix release workflow to fall back to `gh release upload` when release
  already exists

## 0.1.6

### Fixes

- Fix PyPI wheel versioning — use dynamic version from `Cargo.toml` instead
  of a hardcoded version in `pyproject.toml`
- Remove Windows ARM64 (`aarch64-pc-windows-msvc`) target (conda not available
  on conda-forge for win-arm64)

## 0.1.5

### Features

- Use clap's built-in help system for `cx help` and `cx --help` instead of
  hand-crafted output — `cx help bootstrap` now works automatically
- Add `cx shell` as a proper clap subcommand (alias for `conda spawn`)
- Replace misleading `conda activate` hints after `cx create` / `cx env create`
  with `cx shell` guidance via output filtering
- Rename `cx info` to `cx status` so `cx info` passes through to `conda info`
- Add `get-cx.sh` (macOS/Linux) and `get-cx.ps1` (Windows) installer scripts
  served from GitHub Pages

### Documentation

- Add installer reference page (`docs/reference/installer.md`)
- Improve GitHub Releases install instructions with platform tables and
  one-liner commands
- Add installer script as the recommended installation method
- Fix binary size references (~10 MB → ~17 MB)
- Fix copyright year to 2026
- Correct GitHub URLs and workflow references throughout

### Internal

- Extract `ensure_bootstrapped()` helper to deduplicate auto-bootstrap logic
- Move installer scripts to `scripts/` directory
- Serve installer scripts from GitHub Pages instead of raw.githubusercontent
- Move changelog to repo root with symlink into docs
- Add `rust-cache` to wheel and crates.io publish jobs

## 0.1.3

### Changes

- Consolidated release, PyPI, and crates.io publishing into a single
  workflow triggered on tag push
- Fixed GitHub Release asset upload (immutable releases compatibility)
- Fixed manylinux compliance for Linux PyPI wheels
- Fixed Windows checksum generation (`sha256sum` instead of `shasum`)

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
