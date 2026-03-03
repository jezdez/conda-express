# Quick start

## Installation

::::{tab-set}

:::{tab-item} GitHub Releases
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

**Linux / macOS one-liner** (replace `TARGET` with your platform):

```bash
TARGET=aarch64-apple-darwin  # adjust for your platform
curl -fsSL "https://github.com/jezdez/conda-express/releases/latest/download/cx-${TARGET}" -o cx
chmod +x cx
sudo mv cx /usr/local/bin/
```
:::

:::{tab-item} PyPI
```bash
pip install conda-express
```
:::

:::{tab-item} crates.io
```bash
cargo install conda-express
```
:::

:::{tab-item} Build from source
[pixi](https://pixi.sh) manages the Rust toolchain from conda-forge:

```bash
git clone https://github.com/jezdez/conda-express.git
cd conda-express
pixi run build
# Binary is at target/release/cx
```
:::

::::

## Bootstrap

Run `cx bootstrap` to install conda into `~/.cx`:

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
