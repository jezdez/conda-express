# Quick start

## Installation

:::::{tab-set}

::::{tab-item} Homebrew
Homebrew is the recommended install path on macOS and Linux:

```bash
brew tap jezdez/conda-express https://github.com/jezdez/conda-express
brew install jezdez/conda-express/cx
```

Update later with `brew upgrade cx`.
::::

::::{tab-item} Shell script
The shell script downloads the right binary for your platform, verifies its
checksum, updates your shell profile / PATH, and runs `cx bootstrap`.

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
| `CX_SKIP_VERIFY` | *(unset)* | Set to skip checksum verification |
| `CX_BUNDLE` | *(unset)* | Bundle directory used by `cx bootstrap` |
| `CX_OFFLINE` | *(unset)* | Set to force offline bootstrap |

`CX_VERSION` uses conda-express release versions, which follow the conda
runtime version in the lock. For example, `26.5.2` installs a `cx` release
that bootstraps conda `26.5.2`.

Unix example:

```bash
curl -fsSL https://jezdez.github.io/conda-express/get-cx.sh | env CX_VERSION=26.5.2 sh
```

PowerShell example:

```powershell
$Env:CX_VERSION = "26.5.2"; irm https://jezdez.github.io/conda-express/get-cx.ps1 | iex
```
:::

::::

::::{tab-item} Releases
Download the binary for your platform from the
[latest release](https://github.com/jezdez/conda-express/releases/latest):

| Platform | cx (7-11 MB) | cxz (50-95 MB) |
|---|---|---|
| Linux x86_64 | `cx-x86_64-unknown-linux-gnu` | `cxz-x86_64-unknown-linux-gnu` |
| Linux ARM64 | `cx-aarch64-unknown-linux-gnu` | `cxz-aarch64-unknown-linux-gnu` |
| macOS x86_64 (Intel) | `cx-x86_64-apple-darwin` | `cxz-x86_64-apple-darwin` |
| macOS ARM64 (Apple Silicon) | `cx-aarch64-apple-darwin` | `cxz-aarch64-apple-darwin` |
| Windows x86_64 | `cx-x86_64-pc-windows-msvc.exe` | `cxz-x86_64-pc-windows-msvc.exe` |

Each runtime has matching `.sha256`, `.info.json`, `.packages.txt`, and
`.runtime.lock` metadata. Direct downloads are also covered by GitHub Artifact
Attestations from the release workflow. For a quick attestation check:

```bash
gh attestation verify ./cx-x86_64-unknown-linux-gnu \
  -R jezdez/conda-express \
  --signer-workflow jezdez/conda-express/.github/workflows/release.yml
```

See {doc}`guides/verify-release-artifacts` for checksum, metadata, lockfile,
and air-gapped transfer checks.

`cxz` is the self-contained variant with the locked package archives embedded. See
{doc}`guides/offline-and-airgapped` for details.

After downloading, make it executable and move it to your `PATH`:

```bash
chmod +x cx-x86_64-unknown-linux-gnu
sudo mv cx-x86_64-unknown-linux-gnu /usr/local/bin/cx
```
::::

::::{tab-item} Docker
A multi-arch image is published to GHCR:

```bash
docker run --rm -v cx-data:/home/nonroot/.conda/express ghcr.io/jezdez/conda-express bootstrap
```

Works on Linux, macOS, and Windows via Docker Desktop. The image runs as
non-root (uid 65532), includes provenance attestations and SBOMs, and can run
with a read-only root filesystem when the managed prefix is mounted as a
writable volume.

```bash
docker run --rm --read-only --tmpfs /tmp \
  -v cx-data:/home/nonroot/.conda/express \
  ghcr.io/jezdez/conda-express status
```

```bash
docker run --rm -v cx-data:/home/nonroot/.conda/express ghcr.io/jezdez/conda-express create -n myenv python=3.12
```

A pre-bootstrapped `cxz` image is also available — conda is already installed,
no bootstrap step needed:

```bash
docker run --rm ghcr.io/jezdez/conda-express:latest-cxz create -n myenv python=3.12
```
::::

::::{tab-item} PyPI
```bash
pip install conda-express
```

The PyPI package installs the `cx` release binary built with conda-ship for
your platform.
::::

:::::

## Bootstrap

![Bootstrap conda, create an environment, and activate it](../demos/quickstart.gif)

If you used the installer script, bootstrap has already been run for you.
Otherwise, run it manually:

```bash
cx bootstrap
```

Bootstrap uses the built-in runtime lock, so it does not solve an environment
at runtime. The prefix is protected with a
[CEP 22](https://conda.org/learn/ceps/cep-0022/) frozen marker to prevent
accidental modification.

## Set up your PATH

This step is optional. Add the managed `condabin` directory to your shell
profile only if you want to run the bootstrapped `conda` executable directly:

```bash
export PATH="$HOME/.conda/express/condabin:$PATH"
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

## Use conda commands

![cx delegates conda commands after bootstrap](../demos/passthrough.gif)

Ordinary conda commands can be run through cx:

```bash
cx install -n myenv scipy matplotlib
cx list -n myenv
cx remove -n myenv scipy
cx env list
```

## Auto-bootstrap

If you skip `cx bootstrap` and run a conda command directly, cx bootstraps on
first use:

```bash
# This bootstraps ~/.conda/express automatically, then runs `conda create`
cx create -n myenv python=3.12
```

## Updating

If you used an early `cx` release that bootstrapped into `~/.cx`, read
{doc}`guides/upgrade-from-early-cx` before removing old files. Current releases
bootstrap into `~/.conda/express`.

To update the base conda installation, re-bootstrap:

```bash
cx bootstrap --force
```

:::{note}
The included `conda-self` plugin is intended to make base updates available as
a conda command once that workflow has settled. Until then, use
`cx bootstrap --force`.
:::

## Uninstalling

To remove the conda prefix and all environments managed by cx:

```bash
cx uninstall
```

This shows the paths it plans to remove and asks for confirmation. It also cleans up
PATH entries from shell profiles and prints a hint for removing the `cx` binary
through your original install method. See the
{ref}`CLI reference <cli-cx-uninstall>` for all options.
