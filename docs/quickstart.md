# Quick start

## Installation

:::::{tab-set}

::::{tab-item} Homebrew
The easiest way to install on macOS and Linux:

```bash
brew tap jezdez/conda-express https://github.com/jezdez/conda-express
brew install jezdez/conda-express/cx
```

Update later with `brew upgrade cx`.
::::

::::{tab-item} Shell script
The shell script downloads the right binary for your platform, verifies its
checksum, updates your shell profile / PATH, and runs `cx bootstrap` — all
in one step.

**macOS / Linux:**

```bash
curl -fsSL https://jezdez.github.io/conda-express/get-cx.sh | sh
```

**Windows (PowerShell):**

```powershell
powershell -ExecutionPolicy ByPass -c "irm https://jezdez.github.io/conda-express/get-cx.ps1 | iex"
```

:::{admonition} Script options
:class: dropdown

All options work as environment variables on both platforms:

| Variable | Default | Description |
|---|---|---|
| `CX_INSTALL_DIR` | `~/.local/bin` (Unix) or `%USERPROFILE%\.local\bin` (Windows) | Where to place the `cx` binary |
| `CX_VERSION` | `latest` | Version to install (without `v` prefix) |
| `CX_NO_PATH_UPDATE` | *(unset)* | Set to skip shell profile / PATH modification |
| `CX_NO_BOOTSTRAP` | *(unset)* | Set to skip running `cx bootstrap` |
| `CX_PAYLOAD` | *(unset)* | Directory of package archives for offline bootstrap |
| `CX_OFFLINE` | *(unset)* | Set to disable network during bootstrap |

Unix example:

```bash
CX_VERSION=0.1.3 curl -fsSL https://jezdez.github.io/conda-express/get-cx.sh | sh
```

PowerShell example:

```powershell
$Env:CX_VERSION = "0.1.3"; irm https://jezdez.github.io/conda-express/get-cx.ps1 | iex
```
:::

::::

::::{tab-item} Releases
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

After downloading, make it executable and move it to your `PATH`:

```bash
chmod +x cx-*
sudo mv cx-* /usr/local/bin/cx
```
::::

::::{tab-item} pip / cargo
```bash
pip install conda-express
```

```bash
cargo install conda-express
```
::::

::::{tab-item} Docker
A minimal, hardened multi-arch image (~37 MB) is published to GHCR:

```bash
docker run --rm -v cx-data:/home/nonroot/.cx ghcr.io/jezdez/conda-express bootstrap
```

Works on Linux, macOS, and Windows via Docker Desktop. The image runs as
non-root (uid 65532), supports `--read-only`, and includes provenance
attestations and SBOMs.

```bash
docker run --rm -v cx-data:/home/nonroot/.cx ghcr.io/jezdez/conda-express create -n myenv python=3.12
```

A pre-bootstrapped `cxz` image is also available — conda is already installed,
no bootstrap step needed:

```bash
docker run --rm ghcr.io/jezdez/conda-express:latest-cxz create -n myenv python=3.12
```
::::

::::{tab-item} Source
[pixi](https://pixi.sh) manages the Rust toolchain from conda-forge:

```bash
git clone https://github.com/jezdez/conda-express.git
cd conda-express
pixi run build
# Binary is at target/release/cx
```
::::

:::::

## Bootstrap

If you used the installer script, bootstrap has already been run for you.
Otherwise, run it manually:

```bash
cx bootstrap
```

This takes ~3–5 seconds using the embedded lockfile. The prefix is protected
with a [CEP 22](https://conda.org/learn/ceps/cep-0022/) frozen marker to
prevent accidental modification.

## Set up your PATH

Add `condabin` to your shell profile so `conda` and `cx` commands are available:

```bash
export PATH="$HOME/.cx/condabin:$PATH"
```

## Create an environment

```bash
cx create -n myenv python=3.12 numpy pandas
```

## Activate an environment

cx uses [conda-spawn](https://github.com/conda-incubator/conda-spawn)
instead of traditional `conda activate`. This spawns a new subshell with the
environment activated — no `conda init` or shell profile modifications needed:

```bash
cx shell myenv
```

To leave the environment, exit the subshell:

```bash
exit    # or Ctrl+D
```

## Use conda normally

All conda commands work transparently through cx:

```bash
cx install -n myenv scipy matplotlib
cx list -n myenv
cx remove -n myenv scipy
cx env list
```

## Auto-bootstrap

If you skip `cx bootstrap` and run any conda command directly, cx will
automatically bootstrap on first use:

```bash
# This bootstraps ~/.cx automatically, then runs `conda create`
cx create -n myenv python=3.12
```

## Updating

To update the base conda installation, re-bootstrap:

```bash
cx bootstrap --force
```

:::{note}
In the future, `conda self update` (via conda-self) will be the canonical
update command. See the [design document](design.md) for details.
:::

## Uninstalling

To completely remove cx, the conda prefix, and all environments:

```bash
cx uninstall
```

This shows what will be removed and asks for confirmation. It also cleans up
PATH entries from shell profiles. See the {ref}`CLI reference <cli-cx-uninstall>`
for all options.
