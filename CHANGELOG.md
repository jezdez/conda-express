# Changelog

## 26.5.2.post3 (2026-06-15)

### Runtime

- Add `conda-exec >=0.3.0` to the default runtime package set.
- Raise the default runtime package floors to `conda-completion >=0.3.0` and
  `conda-workspaces >=0.7.0`.

## 26.5.2.post2 (2026-06-11)

### Distribution

- Build CI and release artifacts with conda-ship `0.3.0`.
- Document conda-ship `0.3.0` runtime metadata, completion, and Windows ARM64
  support boundaries.

## 26.5.2.post1 (2026-06-03)

### Distribution

- Rebuild the PyPI wheels with single `manylinux2014` platform tags for Linux,
  so PyPI accepts the linux-64 and linux-aarch64 wheel uploads.

## 26.5.2 (2026-06-03)

### Versioning

- conda-express now follows the conda runtime version. The `26.5.2` release
  bootstraps conda `26.5.2`.
- conda-express-only rebuilds should use post-release versions such as
  `26.5.2.post1`.

### Runtime

- Pin the runtime conda package exactly to `26.5.2` across linux-64,
  linux-aarch64, osx-64, osx-arm64, and win-64.
- Keep the managed runtime prefix at `~/.conda/express` and keep the base
  prefix protected with the CEP 22 frozen marker after bootstrap.
- Include `conda-completion >=0.2.0` and `conda-workspaces >=0.5.0` in the
  default runtime package set, alongside `conda-rattler-solver`,
  `conda-spawn`, `conda-pypi`, `conda-self`, and `conda-global`.

### Distribution

- Build the published `cx` and `cxz` runtime binaries with conda-ship instead
  of the old repository-local Rust builder.
- Build with conda-ship `0.2.1` and derive the generated runtime version from
  project metadata, so release tags drive both the Python package version and
  the stamped binary version.
- Keep conda-express focused on the `cx` / `cxz` distribution.
  Custom runtime builds now belong in conda-ship.
- Continue publishing GitHub Release artifacts, installer scripts, Docker
  images, the Homebrew tap formula, and PyPI wheels.
- Stop publishing new crates.io releases.
- Attest release artifacts before publishing the immutable GitHub Release.

### Upgrade Notes

- Early `cx` releases used `~/.cx`; current releases use `~/.conda/express`.
  Upgrading the binary does not migrate that old prefix. Keep `~/.cx` until
  you have recreated or archived any environments you still need, then remove
  it manually.
- If an old Cargo-installed `cx` is still earlier on `PATH`, remove it and
  install `cx` through one of the current distribution channels.

### Installer Scripts

- Pass `CX_BUNDLE` and `CX_OFFLINE` through to `cx bootstrap` during installer
  bootstrapping, so scripted offline installs use the documented bundle and
  network settings.

### Documentation

- Rework the docs around conda-express as a distribution built with
  conda-ship, with conda-ship linked as the tool for custom runtimes.
- Add focused guides for installer-style conda distributions, offline and
  air-gapped use, release artifact verification, and upgrading from early `cx`
  releases.

## 0.6.0 (2026-05-06)

### Features

- **Slim `build.rs`** — Replace the 440-line `build.rs` with a small script
  that copies pre-generated `cx.lock` and `payload.tar.zst` to `$OUT_DIR`,
  reducing duplicate build-dependency compilation.
- **`cx-build` crate** — Add an internal build tool with `prepare`, `payload`,
  and `configure` subcommands for deriving `cx.lock`, downloading package
  archives, and configuring custom builds.
- **`cx-env` Pixi feature** — Define the bootstrap package set as a Pixi
  environment so `pixi lock` solves dependencies instead of `build.rs`.
- **`conda-global`** — Added to the default package set alongside existing conda plugins.
- **`conda-workspaces >=0.4.0`** — Added to the default package set with version pin.
- **sccache** — Local and CI build caching via `RUSTC_WRAPPER=sccache`.

### Fixes

- Stop self-deleting the `cx` binary on `cx uninstall` — now prints a hint for the user to remove it manually.
- Clean subenvironment artifacts (envs, pkgs cache) on uninstall.
- Precompile Python bytecode after bootstrap to avoid first-run `.pyc` compilation delays.
- Remove unused `default_channels` from generated `.condarc`.
- Pin `reqwest-middleware` and `sha2` versions to match rattler's transitive requirements.
- Fix `getrandom` 0.3 usage in cx-wasm to match ahash's transitive dependency.
- Fix JupyterLite `yarn.lock` TypeScript compatibility patch hash.

### Build

- Move exclude filtering from runtime to build time; the `cx` binary trusts its
  pre-filtered `cx.lock`.
- Remove `--exclude` and `--no-exclude` from `cx bootstrap`.
- Update `action.yml` to use the `cx-build configure`, `pixi lock`,
  `cx-build prepare`, and `cargo build` pipeline.
- Rename the `xtask` crate to `cx-build`.

### Docs

- Updated `DESIGN.md`, `README.md`, `docs/configuration.md`, and `docs/index.md` to reflect `cx-build` rename, `conda-global` addition, and updated version pins.
- Updated stale size and package count figures across all docs: lockfile 39 KB → ~130 KB, package counts 86/113 → ~95/~125, py-rattler wheel sizes ~28-31 MB → 13-33 MB.
- Embedded remaining demo GIFs in docs and README.
- Added VHS demos for conda-workspaces, quickstart, status, and passthrough.
- Documented conda-workspaces in features, README, and index pages.

### CI

- Allow Codecov upload to fail on PRs.
- Add `CHANGELOG.md` and `PLAN.md` to docs CI paths filter; drop release trigger from docs workflow.
- Only deploy GitHub Pages from `main` branch.
- Add Dependabot configuration for GitHub Actions, Cargo, npm, and pip.
- Fix `CX_EMBED_PAYLOAD` env var for Windows PowerShell compatibility.
- Scope `cargo publish` to the `conda-express` crate only.
- Make Trivy CVE scan non-blocking for upstream base image vulnerabilities.
- Enable sccache GitHub Actions cache backend for persistent build caching.

### Dependencies

- Bump rattler ecosystem and other Rust dependencies.
- Bump npm dependencies in cx-jupyterlite.
- Bump GitHub Actions to latest versions.

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

The browser/WebAssembly features from this historical release now live in
[`conda-wasm`](https://github.com/jezdez/conda-wasm). See the
`conda-wasm` changelog for the moved `cx-wasm`, `conda-emscripten`,
`cx-jupyterlite`, `cx-wasm-kernel`, and JupyterLite demo history.

### Features

- **Docker** — minimal multi-arch images on GHCR for `cx` in containers.

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
