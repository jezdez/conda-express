# conda-express (cx) design document

## What is conda-express?

conda-express (cx) is a lightweight, single-binary bootstrapper for conda, written in Rust using the [rattler](https://github.com/conda/rattler) crate ecosystem. It replaces the miniconda/constructor install pattern with a ~17 MB static binary that can install a fully functional conda environment in seconds.

Inspired by uv's single-binary distribution model, cx aims to be the fastest way to get a working conda installation.

## Current status

**Working PoC** — all core functionality is implemented and tested on macOS ARM64.

| Feature | Status |
|---|---|
| Single-binary bootstrapper | Done |
| Compile-time lockfile (rattler-lock v6) | Done |
| Post-solve package exclusion | Done |
| conda-libmamba-solver removal | Done |
| conda-rattler-solver as default | Done |
| conda-spawn activation model | Done (installed) |
| `cx shell` alias for `conda spawn` | Done |
| Disabled commands (`activate`, `deactivate`, `init`) | Done |
| Auto-bootstrap on first `conda` command | Done |
| `.condarc` with `solver: rattler` | Done |
| External lockfile override (`--lockfile`) | Done |
| Live solve fallback (`--no-lock`) | Done |
| Multi-platform CI (via pixi) | Done |
| Release binary builds | Done |
| CEP 22 frozen base prefix | Done |
| `cx help` (clap auto-generated) | Done |
| Output filtering (create/env create) | Done |
| Installer scripts (get-cx.sh, get-cx.ps1) | Done |
| `cx uninstall` (anti-bootstrap) | Done |
| Reusable GitHub Action / composite action | Done |
| Build-time env var overrides (`CX_PACKAGES`, etc.) | Done |
| Self-update (via conda-self plugin) | Not started |

### Numbers (macOS ARM64, debug/release)

| Metric | Value |
|---|---|
| Release binary size | ~17 MB |
| Installed packages (base) | 86 |
| Excluded packages (libmamba tree) | 27 |
| Bootstrap time (embedded lockfile) | ~3–5 s |
| Bootstrap time (live solve) | ~7–8 s |
| Lockfile size | ~1050 lines (rattler-lock v6) |

## Architecture

```
pixi.toml              [tool.cx]: packages, channels, excludes
       |
       v
    build.rs           Compile-time: solve + filter + write lockfile
       |
       v
    cx.lock            rattler-lock v6 (embedded via include_str!)
       |
       v
      cx               Single binary (~17 MB release)
       |
       +---> bootstrap -----> install from lockfile (fast path)
       |                       or live solve (fallback)
       |                       write CEP 22 frozen marker
       |
       +---> status -----------> show cx prefix metadata
       |
       +---> shell -----------> alias for `conda spawn` (activate via subshell)
       |
       +---> uninstall -------> remove prefix, envs, binary, PATH entries
       |
       +---> help -----------> clap auto-generated help with quick start
       |
       +---> activate/deactivate/init --> disabled (guides to conda-spawn)
       |
       +---> <any conda arg> --> hand off to installed conda binary
       |                         (includes `conda self update` via conda-self)
```

### Compile-time lockfile

`build.rs` performs the full solve at `cargo build` time:

1. Reads `[tool.cx]` from `pixi.toml` (packages, channels, excludes)
2. Applies environment variable overrides if set (`CX_PACKAGES`, `CX_CHANNELS`, `CX_EXCLUDE`)
3. Hashes the config (including overrides); skips solve if cached lockfile matches
4. Fetches repodata via `rattler_repodata_gateway` (sharded)
5. Solves via `rattler_solve` (resolvo)
6. Filters out excluded packages and their exclusive dependencies
7. Writes a rattler-lock v6 lockfile to `$OUT_DIR/cx.lock`
8. Binary embeds it via `include_str!`

At runtime, bootstrap parses the embedded lockfile, extracts `RepoDataRecord`s, and passes them directly to `rattler::install::Installer` — no repodata fetch, no solve.

### Package exclusion

conda on conda-forge hard-depends on `conda-libmamba-solver`. Since cx uses `conda-rattler-solver` instead, it removes libmamba and its 27 exclusive native dependencies (libsolv, libarchive, libcurl, spdlog, etc.) via a post-solve transitive dependency pruning algorithm. This happens both at compile time (in `build.rs`) and optionally at runtime (for live solve or external lockfiles).

### Disabled commands

cx intercepts three conda commands that conflict with the conda-spawn activation model:

- **`activate` / `deactivate`** — prints a message directing users to `conda spawn` instead.
- **`init`** — explains that shell profile modifications are unnecessary; guides the user to add `condabin` to their PATH.

These commands exit with a non-zero status to prevent scripts from silently succeeding.

### Process hand-off

When cx receives a command it doesn't own (anything other than `bootstrap`, `status`, `shell`, `help`, or a disabled command), it replaces its own process with the installed `conda` binary using the Unix execvp syscall. For `create` and `env create`, cx runs conda as a subprocess to filter misleading `conda activate` hints from the output, replacing them with `cx shell` guidance. This means conda's full feature set is available transparently — cx is invisible after bootstrap.

### Frozen base prefix (CEP 22)

After bootstrap, cx writes a `conda-meta/frozen` marker file per [CEP 22](https://conda.org/learn/ceps/cep-0022/). This protects the base prefix from accidental modification — users should create named environments for their work. Updating the base installation is handled by `conda self update` (via conda-self), which internally overrides the frozen check.

## File structure

```
conda-express/
  Cargo.toml            Rust project manifest (crate: conda-express, binary: cx)
  pyproject.toml        maturin config for PyPI wheel builds
  pixi.toml             Dev environment + [tool.cx] package config + docs deps
  action.yml            Composite GitHub Action for building custom cx binaries
  pixi.lock             Locked dev dependencies
  build.rs              Compile-time solver and lockfile generator
  cx.lock               Cached rattler-lock v6 lockfile (checked in)
  cx.lock.hash          Config hash for cx.lock cache invalidation
  CHANGELOG.md          Release changelog
  LICENSE               BSD 3-Clause
  README.md             User-facing documentation
  DESIGN.md             This file
  PLAN.md               Feasibility analysis and implementation plan
  src/
    main.rs             Entry point, command dispatch, disabled commands
    cli.rs              CLI definitions (clap)
    config.rs           Embedded config, prefix metadata, .condarc, CEP 22 frozen
    install.rs          Package installation (lockfile + live-solve paths)
    exec.rs             Process replacement (exec into installed conda)
  python/
    conda_express/
      __init__.py       Exposes find_cx_bin()
      __main__.py       python -m conda_express -> exec cx
      _find_cx.py       Locate cx binary in sysconfig paths
      py.typed          PEP 561 type marker
  scripts/
    get-cx.sh           Installer script for macOS/Linux
    get-cx.ps1          Installer script for Windows (PowerShell)
  docs/
    conf.py             Sphinx config (conda-sphinx-theme, MyST)
    index.md            Homepage with install tabs, grid cards
    quickstart.md       Installation and first steps
    features.md         Feature descriptions
    configuration.md    Build-time and runtime config reference
    design.md           Includes DESIGN.md via MyST include
    changelog.md        Symlink to ../CHANGELOG.md
    reference/
      cli.md            CLI reference
      installer.md      Installer script reference
  .github/
    workflows/
      ci.yml            CI: build, test, lint on all platforms (canary artifacts)
      release.yml       Build binaries + wheels, publish to GitHub Releases, PyPI, and crates.io
      build.yml      Reusable workflow for building custom cx binaries (workflow_call)
      docs.yml          Build and deploy Sphinx docs to GitHub Pages
```

## Development environment

cx uses [pixi](https://pixi.sh) to manage the Rust toolchain from conda-forge, ensuring consistent builds across local development and CI:

```bash
# Install pixi (if not already installed)
curl -fsSL https://pixi.sh/install.sh | bash

# Build, test, lint
pixi run build         # cargo build --release
pixi run test          # cargo test
pixi run lint          # fmt-check + clippy
```

The `pixi.toml` pins `rust >= 1.85` from conda-forge. CI workflows use `prefix-dev/setup-pixi` to replicate the same environment on all platforms.

## Configuration

The `[tool.cx]` section in `pixi.toml` is the single source of truth for what gets installed:

```toml
[tool.cx]
channels = ["conda-forge"]
packages = [
    "python >=3.12",
    "conda >=25.1",
    "conda-rattler-solver",
    "conda-spawn",
    "conda-pypi",
    "conda-self",
]
exclude = ["conda-libmamba-solver"]
```

Both `build.rs` (compile-time) and the runtime binary read from `pixi.toml`. Changing it triggers an automatic re-solve on the next `cargo build`.

### Build-time environment variable overrides

For custom builds (e.g. via the reusable GitHub Action), `build.rs` supports environment variable overrides that replace the `pixi.toml` values:

| Variable | Overrides | Format |
|---|---|---|
| `CX_PACKAGES` | `[tool.cx].packages` | Comma-separated match specs |
| `CX_CHANNELS` | `[tool.cx].channels` | Comma-separated channel names |
| `CX_EXCLUDE` | `[tool.cx].exclude` | Comma-separated package names |

When overrides are active, the checked-in `cx.lock` is skipped (a fresh solve runs) and the repo-root lockfile is not overwritten.

## CLI

```
cx bootstrap [--force] [--prefix DIR] [--channel CH] [--package PKG]
             [--exclude PKG] [--no-exclude] [--no-lock] [--lockfile PATH]
cx status [--prefix DIR]
cx shell [ENV]           # alias for conda spawn (activate via subshell)
cx uninstall [--prefix DIR] [--yes]  # remove prefix, envs, binary, PATH entries
cx <any-conda-command>   # transparently delegates to conda
```

Default prefix: `~/.cx`

### Disabled commands

| Command | Behavior |
|---|---|
| `cx activate` | Prints guidance to use `conda spawn` instead |
| `cx deactivate` | Prints guidance to use `conda spawn` instead |
| `cx init` | Explains `conda init` is unnecessary with conda-spawn |

## Default installed plugins

| Plugin | Purpose |
|---|---|
| conda-rattler-solver | Rust-based solver (replaces libmamba) |
| conda-spawn | Subprocess-based activation (replaces `conda activate`) |
| conda-pypi | PyPI interoperability (install, solve, convert) |
| conda-self | Base environment self-management |

## Lockfile compatibility

The embedded `cx.lock` is a standard rattler-lock v6 file, compatible with:

- pixi (same lockfile format)
- conda-lockfiles (`RattlerLockV6Loader`)
- Version control (can be checked in for audit)

## Key dependencies

All from the [rattler](https://github.com/conda/rattler) ecosystem:

| Crate | Role |
|---|---|
| `rattler` | Package installation engine |
| `rattler_solve` (resolvo) | SAT-based dependency solver |
| `rattler_repodata_gateway` | Repodata fetching (sharded) |
| `rattler_conda_types` | conda type definitions |
| `rattler_lock` | Lockfile read/write (v6 format) |
| `rattler_virtual_packages` | Virtual package detection |
| `rattler_networking` | Auth middleware, OCI support |
| `rattler_cache` | Cache directory management |

## PyPI distribution

cx is published to PyPI as platform wheels via [maturin](https://github.com/PyO3/maturin) (`bindings = "bin"`), following the same pattern as [uv](https://github.com/astral-sh/uv). A tiny Python wrapper in `python/conda_express/` provides:

- `find_cx_bin()` — locates the binary via sysconfig
- `python -m conda_express` — finds and exec's the cx binary

## CI/CD

All workflows use `pixi` for toolchain management:

- **`ci.yml`** — runs on push to `main` and PRs. Builds and tests across 5 targets (linux-x64, linux-aarch64, macos-x64, macos-arm64, windows-x64). Uploads canary binaries as artifacts. Runs `pixi run lint` separately.
- **`release.yml`** — triggers on tag push (`v*`). Orchestrates the full release pipeline: builds native binaries, builds maturin platform wheels and sdist, creates a GitHub Release with binary assets, publishes wheels to PyPI via trusted publishing (OIDC), and publishes the crate to crates.io via trusted publishing (`rust-lang/crates-io-auth-action`). All steps run as separate jobs with dependency ordering.
- **`build.yml`** — reusable workflow (`workflow_call`) for building custom cx binaries. Accepts `packages`, `channels`, `exclude`, and `ref` inputs. Builds all 5 platforms using the composite action and uploads binary artifacts with checksums.
- **`docs.yml`** — triggers on push to `main` (docs paths), PRs, and manual dispatch. Builds Sphinx documentation and deploys to GitHub Pages.

### Composite action (`action.yml`)

The repo root contains a composite GitHub Action that lets other repos build custom cx binaries with `uses: jezdez/conda-express@main`. It accepts `packages`, `channels`, `exclude`, and `ref` inputs, checks out conda-express, builds with env var overrides, and outputs the `binary-path` and `asset-name`. Callers handle their own platform matrix.

## Future work

- **conda-self updater plugin**: Pluggable backend for conda-self so `conda self update` can delegate to cx/rattler for cx-managed prefixes (handles post-solve exclusion of libmamba). This is the canonical update path — cx intentionally does not implement its own update command.
- **Upstream conda-forge**: Make `conda-libmamba-solver` an optional dependency of conda on conda-forge, eliminating the need for post-solve exclusion entirely.
