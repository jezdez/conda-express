# conda-express (cx)

[![CI](https://github.com/jezdez/conda-express/actions/workflows/ci.yml/badge.svg)](https://github.com/jezdez/conda-express/actions/workflows/ci.yml)
[![Docs](https://github.com/jezdez/conda-express/actions/workflows/docs.yml/badge.svg)](https://jezdez.github.io/conda-express/)
[![codecov](https://codecov.io/gh/jezdez/conda-express/graph/badge.svg)](https://codecov.io/gh/jezdez/conda-express)
[![License](https://img.shields.io/github/license/jezdez/conda-express)](https://github.com/jezdez/conda-express/blob/main/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/conda-express)](https://crates.io/crates/conda-express)
[![PyPI](https://img.shields.io/pypi/v/conda-express)](https://pypi.org/project/conda-express/)

A lightweight, single-binary bootstrapper for [conda](https://github.com/conda/conda), powered by [rattler](https://github.com/conda/rattler). The `cx` binary is short for **c**onda e**x**press.

cx replaces the miniconda/constructor installation pattern with a 7-11 MB static binary that bootstraps a fully functional conda environment in seconds.

## Quick start

![Bootstrap conda, create an environment, and activate it](demos/quickstart.gif)

```bash
# Bootstrap a conda installation (first run only, ~3–5 s)
cx bootstrap

# Use conda normally — cx delegates transparently
cx install -n myenv numpy pandas
cx create -n science python=3.12 scipy

# Activate environments using conda-spawn (no shell init needed)
cx shell myenv
```

On first use, cx automatically installs conda and its plugins into `~/.cx` from an embedded lockfile. Subsequent invocations hand off directly to the installed `conda` binary with no overhead.

## What gets installed

cx installs a minimal conda stack from conda-forge:

| Package | Role |
|---|---|
| python >= 3.12 | Runtime |
| conda >= 25.1 | Package manager |
| conda-rattler-solver | Rust-based solver (replaces libmamba) |
| conda-spawn | Subprocess-based environment activation |
| conda-pypi | PyPI interoperability |
| conda-self | Base environment self-management |
| [conda-workspaces](https://conda-incubator.github.io/conda-workspaces/) | Multi-environment workspace and task management |

The `conda-libmamba-solver` and its 27 exclusive native dependencies (libsolv, libarchive, libcurl, spdlog, etc.) are excluded by default, reducing the install size significantly.

## Installation

### Homebrew (recommended)

The easiest way to install on macOS and Linux:

```bash
brew tap jezdez/conda-express https://github.com/jezdez/conda-express
brew install jezdez/conda-express/cx
```

Update later with `brew upgrade cx`.

### Shell script

**macOS / Linux:**

```bash
curl -fsSL https://jezdez.github.io/conda-express/get-cx.sh | sh
```

**Windows (PowerShell):**

```powershell
powershell -ExecutionPolicy ByPass -c "irm https://jezdez.github.io/conda-express/get-cx.ps1 | iex"
```

The script detects your platform, downloads the right binary, verifies the checksum, updates your shell profile / PATH, and runs `cx bootstrap`. Customize with environment variables:

- `CX_INSTALL_DIR` — where to place the binary (default: `~/.local/bin` or `%USERPROFILE%\.local\bin`)
- `CX_VERSION` — specific version to install (default: `latest`)
- `CX_NO_PATH_UPDATE` — set to skip shell profile / PATH modification
- `CX_NO_BOOTSTRAP` — set to skip running `cx bootstrap`

### From GitHub Releases

Download the binary for your platform from the
[latest release](https://github.com/jezdez/conda-express/releases/latest):

| Platform | File |
|---|---|
| Linux x86_64 | `cx-x86_64-unknown-linux-gnu` |
| Linux ARM64 | `cx-aarch64-unknown-linux-gnu` |
| macOS x86_64 (Intel) | `cx-x86_64-apple-darwin` |
| macOS ARM64 (Apple Silicon) | `cx-aarch64-apple-darwin` |
| Windows x86_64 | `cx-x86_64-pc-windows-msvc.exe` |

Each file has a matching `.sha256` checksum.

### Docker

A minimal, hardened multi-arch image (~37 MB) is published to GHCR on every release:

```bash
docker run --rm -v cx-data:/home/nonroot/.cx ghcr.io/jezdez/conda-express bootstrap
```

The image runs as non-root (uid 65532), works with `--read-only`, and includes provenance attestations and SBOMs. Docker Desktop on macOS and Windows automatically selects the right architecture (linux/amd64 or linux/arm64).

```bash
# Run conda commands through cx
docker run --rm -v cx-data:/home/nonroot/.cx ghcr.io/jezdez/conda-express create -n myenv python=3.12
```

Use as a container in CI (GitHub Actions):

```yaml
jobs:
  test:
    container: ghcr.io/jezdez/conda-express:latest
    steps:
      - run: cx bootstrap && cx create -n test python numpy
```

Use as a base for multi-stage application builds:

```dockerfile
FROM ghcr.io/jezdez/conda-express:latest AS conda-builder
RUN cx bootstrap && cx create -n app python numpy pandas
FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=conda-builder /home/nonroot/.cx/envs/app /opt/conda
```

### From PyPI

```bash
pip install conda-express
```

### From crates.io

```bash
cargo install conda-express
```

The package is published as `conda-express` on [PyPI](https://pypi.org/project/conda-express/) and [crates.io](https://crates.io/crates/conda-express).

## Building from source

Requires [pixi](https://pixi.sh) (recommended) or [Rust](https://rustup.rs/) (edition 2024).

### With pixi (recommended)

[pixi](https://pixi.sh) manages the Rust toolchain from conda-forge for reproducible builds:

```bash
git clone https://github.com/jezdez/conda-express.git
cd conda-express

pixi run build          # cargo build --release
pixi run test           # cargo test
pixi run lint           # fmt-check + clippy
```

### With system Rust

```bash
git clone https://github.com/jezdez/conda-express.git
cd conda-express

# Build (first build solves packages at compile time — needs network)
cargo build --release

# Binary is at target/release/cx
./target/release/cx --help
```

The first build runs a compile-time solve via `build.rs`, generating a rattler-lock v6 lockfile that gets embedded into the binary. Subsequent builds reuse the cached lockfile unless `pixi.toml` changes.

## Configuration

Package specs, channels, and exclusions live in the `[tool.cx]` section of `pixi.toml`:

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
    "conda-workspaces",
]
exclude = ["conda-libmamba-solver"]
```

Edit this section to customize what cx installs, then rebuild. You can also override these values at build time using environment variables — see [Building custom cx binaries](#building-custom-cx-binaries) below.

## CLI reference

```
cx bootstrap [OPTIONS]           Bootstrap a fresh conda installation
  --force                        Re-bootstrap even if prefix exists
  --prefix DIR                   Target directory (default: ~/.cx)
  --channel CH                   Channels (default: conda-forge)
  --package PKG                  Additional packages to install
  --exclude PKG                  Packages to exclude (default: conda-libmamba-solver)
  --no-exclude                   Disable default exclusions
  --no-lock                      Ignore embedded lockfile, do a live solve
  --lockfile PATH                Use an external lockfile instead

cx status [--prefix DIR]         Show cx installation status
cx shell [ENV]                   Alias for conda spawn (activate via subshell)
cx uninstall [OPTIONS]           Remove cx, conda prefix, and all environments
  --prefix DIR                   Target directory (default: ~/.cx)
  -y, --yes                      Skip confirmation prompt

cx help                          Getting-started guide
cx <conda-args>                  Passed through to conda
```

### Disabled commands

cx uses conda-spawn instead of traditional shell-based activation. The following commands are intentionally disabled:

| Command | Instead |
|---|---|
| `conda activate` / `deactivate` | `cx shell myenv` |
| `conda init` | Add `condabin` to your PATH (see below) |

### Frozen base prefix

The `~/.cx` prefix is protected with a [CEP 22](https://conda.org/learn/ceps/cep-0022/) frozen marker after bootstrap. This prevents accidental modification of the base environment (e.g., `conda install numpy` into base). Users should create named environments for their work:

```bash
cx create -n myenv numpy pandas
cx shell myenv
```

Updating the base installation is handled by `conda self update` (via conda-self).

## Building custom cx binaries

You can build a cx binary with your own set of packages using the composite GitHub Action or the reusable workflow.

### Composite action (`uses: jezdez/conda-express@main`)

Use in a step within your own workflow. You control the platform matrix:

```yaml
jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: jezdez/conda-express@main
        id: cx
        with:
          packages: "python >=3.12, conda >=25.1, conda-rattler-solver, conda-spawn, numpy, pandas"

      - uses: actions/upload-artifact@v4
        with:
          name: ${{ steps.cx.outputs.asset-name }}
          path: ${{ steps.cx.outputs.binary-path }}
```

### Reusable workflow

Builds all 5 platforms in one call:

```yaml
jobs:
  build-cx:
    uses: jezdez/conda-express/.github/workflows/build.yml@main
    with:
      packages: "python >=3.12, conda >=25.1, conda-rattler-solver, conda-spawn, numpy, pandas"
```

### Build-time environment variables

When building from source, you can override the package configuration without editing `pixi.toml`:

| Variable | Overrides | Format |
|---|---|---|
| `CX_PACKAGES` | `[tool.cx].packages` | Comma-separated match specs |
| `CX_CHANNELS` | `[tool.cx].channels` | Comma-separated channel names |
| `CX_EXCLUDE` | `[tool.cx].exclude` | Comma-separated package names |

```bash
CX_PACKAGES="python >=3.12, conda >=25.1, numpy" pixi run build
```

## Uninstalling

To completely remove cx, the conda prefix, all environments, and the cx binary:

```bash
cx uninstall
```

This will show what will be removed and ask for confirmation. Use `--yes` to skip the prompt. The command also cleans up PATH entries from shell profiles that were added by the installer.

## How it works

1. **Compile time**: `build.rs` reads `[tool.cx]` from `pixi.toml`, solves dependencies using rattler, filters excluded packages, and writes a rattler-lock v6 lockfile embedded into the binary.

2. **First run**: cx parses the embedded lockfile, downloads packages from conda-forge, and installs them into the prefix. No repodata fetch or solve needed at runtime.

3. **Subsequent runs**: cx detects the existing prefix and replaces its own process with the installed `conda` binary, passing all arguments through.

## Activation model

cx ships with conda-spawn instead of traditional `conda activate`. There is no need to run `conda init` or modify shell profiles.

```bash
# Add cx to PATH (one-time setup)
export PATH="$HOME/.cx/condabin:$PATH"

# Activate an environment (spawns a subshell)
cx shell myenv

# Deactivate by exiting the subshell
exit
```

## Lockfile format

The embedded lockfile uses the [rattler-lock v6](https://github.com/conda/rattler/tree/main/crates/rattler_lock) format (same as `pixi.lock`). It can be:

- Read by pixi
- Imported by conda-lockfiles
- Checked into version control for reproducibility auditing

## License

BSD 3-Clause. See [LICENSE](LICENSE).
