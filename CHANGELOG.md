# Changelog

## 0.5.3 (2026-03-31)

### Fixes

- Add `channels: [conda-forge]` to the generated `.condarc`. conda 25.x requires `channels` to be set explicitly — `default_channels` alone is not sufficient, causing `NoChannelsConfiguredError` on commands like `conda create`.

### Docs

- Fix `features.md` ASCII diagram: payload size was unlabeled, cx binary label was duplicated in the cxz column.
- Add air-gapped/cxz card to `index.md` landing page and mention cxz in the intro.
- Add cxz context to `background.md` rationale sections.
- Add cxz binaries to `quickstart.md` release download table.
- Add cxz tip to `installer.md` offline options.

## 0.5.2 (2026-03-30)

### Fixes

- Fix file ownership in `Dockerfile.cxz`: use `--chown=nonroot:nonroot` when copying the bootstrapped prefix so the nonroot user in the distroless image can read `conda-meta/` files.

## 0.5.1 (2026-03-30)

### Fixes

- Allow `cxz` binaries through `docker/.dockerignore` so the `docker-cxz-images` release job can copy them into the build context.

### Docs

- Update binary size estimates across all documentation to match actual 0.5.0 release artifacts: `cx` 7-11 MB (platform-dependent), `cxz` 50-95 MB, payload increase 40-85 MB.

## 0.5.0 (2026-03-30)

### Features

- **Offline bootstrap** — New `--payload DIR` and `--offline` flags for `cx bootstrap` enable fully air-gapped installations from pre-downloaded `.conda` / `.tar.bz2` archives. Also available via `CX_PAYLOAD` and `CX_OFFLINE` environment variables for use in native installer post-install scripts (macOS PKG, Windows MSI) and CI pipelines.
- **Self-contained binary (`cxz`)** — Build with `CX_EMBED_PAYLOAD=1` to bundle all locked package archives directly into the binary. One 50-95 MB file (varies by platform), zero network access — drop it anywhere and run `cxz bootstrap`. Auto-detects the embedded payload at runtime; all other `cx` flags and subcommands work identically.
- **Docker cxz image** — Pre-bootstrapped multi-arch Docker image built on `gcr.io/distroless/cc-debian12:nonroot`, published alongside the existing `cx` image on GHCR.
- **GitHub Action `embed-payload` input** — Build `cxz` binaries via the Action or reusable workflow with `embed-payload: "true"`.
- **Release profile optimizations** — `lto = "fat"`, `codegen-units = 1`, `opt-level = "z"` reduce the `cx` binary from ~17 MB to 7-11 MB (varies by platform).

### Improvements

- SHA256 verification of all packages downloaded during `cxz` build, with automatic re-download on checksum mismatch.
- Embedded payload temp directory is cleaned up after extraction.
- New `lockfile_records()` helper deduplicates lockfile parsing across `from_lockfile`, `from_lockfile_with_payload`, and `from_lockfile_offline`.
- `cx status` shows `cxz` as the binary name and embedded payload size when applicable.
- Use idiomatic `&Path` instead of `&PathBuf` in build script function signatures.

### Tests

- Parameterized `CX_OFFLINE` env var parsing tests (7 truthy/falsy cases).
- `CX_PAYLOAD` env var forwarding test.
- `cx status` binary name and version output test.
- `--offline --no-lock` rejection and bad `--payload` directory tests.
- Online-gated integration tests for full offline and payload bootstrap workflows.

### Docs

- CLI reference for `--payload`, `--offline`, and `cxz bootstrap` examples.
- Configuration reference for `CX_PAYLOAD`, `CX_OFFLINE`, and `CX_EMBED_PAYLOAD`.
- Features page with `cxz` section and ASCII architecture diagram.
- Custom builds guide for building `cxz` locally, via GitHub Action, and via reusable workflow.
- Docker quickstart tab for the pre-bootstrapped `cxz` image.

### CI

- `cxz` build and smoke test in CI (Linux x86_64).
- Release workflow: `cxz` build matrix (Linux, macOS, Windows), pre-bootstrapped Docker image build and push.

## 0.4.1 (2026-03-30)

### Fixes

- **cx create / cx env create** — Avoid piping conda stdout when stdin is a TTY and `-y` / `--yes` is not set. Conda prints confirmation prompts to stdout without a trailing newline, then reads stdin; line-oriented output filtering blocked the prompt and made input appear swallowed. Activation-hint filtering still runs for non-interactive use (`-y` / `--yes`) or when stdin is not a terminal.
- Add unix integration test reproducing conda's stdout/stdin prompt pattern (`BufRead::read_line` and `lines()`).

### Tests

- **Uninstall integration tests** — Use explicit `--prefix` for the interactive uninstall test on Windows (`dirs` 6 resolves home via known-folder profile, not `HOME` / `USERPROFILE`). Parametrize status vs uninstall missing-prefix cases with rstest; add a unix-only test for default prefix when `HOME` points at a synthetic layout.

## 0.4.0 (2026-03-31)

### Features

- **cx-wasm** — WebAssembly build of the rattler solver and package extractor for use in the browser (`crates/cx-wasm/`).
- **conda-emscripten** — conda plugin for Emscripten: `CxWasmSolver` (`CONDA_SOLVER=cx-wasm`), WASM extraction, `%cx` / `%conda` IPython magics, MEMFS-oriented patches (downloads, subprocess no-op, extractor), shared-library loading for C extensions after install.
- **cx-jupyterlite** — JupyterLite federated extension rewrites bare `conda` cell commands so the kernel magics handle them.
- **cx-wasm-kernel** — conda recipe packaging WASM artifacts and `cx_wasm_bridge` (async repodata shard prefetch at kernel startup for fast solves).
- **JupyterLite demo** (`lite/`) — static site with xeus-python; demo notebooks under `lite/files/notebooks/demos/`; GitHub Pages deploy at `/demo/`.
- **Async shard prefetch** — two-phase fetch (parallel `fetch()` at startup) + sync solve; large solve-time improvement when using sharded repodata (CEP-16).
- **Docker** — minimal multi-arch images on GHCR for `cx` in containers.
- **Docs** — browser/WASM guide, Diátaxis-aligned docs updates, DESIGN/PLAN refresh for WASM; **Background & rationale** page; **Implementation plan** and changelog included in Sphinx; GitHub issue templates (`type::feature`, epic, bug).

### Fixes

- cx-wasm / conda-emscripten: cross-channel transitive dependency resolution, pyjs coercion, repodata URL derivation, session-level shard caching, and related install-path fixes.
- Demo notebooks: WASM-friendly examples (e.g. `lz4`, `np.linalg.eigh`), runtime `conda install` where appropriate, scipy in kernel env.

### Notes

- Default embedded stack still uses **conda-rattler-solver** and excludes **conda-libmamba-solver**; lockfile updated in step with conda-forge pins (e.g. conda-rattler-solver / py-rattler bumps as recorded in commits).

## 0.3.1 (2026-03-06)

### Fixes

- Fix crates.io publish — `build.rs` writes lockfiles during compilation,
  which `cargo publish --verify` rejects. Skip verification since builds are
  already validated by CI.
- Fix `build.yml` reusable workflow validation error on push events — pin
  action reference to `@main` instead of dynamic `inputs.ref` in the `uses:`
  field.

## 0.3.0 (2026-03-06)

_No changelog entry was added for this release._

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
