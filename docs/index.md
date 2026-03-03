# conda-express

A lightweight, single-binary bootstrapper for [conda](https://github.com/conda/conda), powered by [rattler](https://github.com/conda/rattler). The `cx` binary is short for **c**onda e**x**press.

cx replaces the miniconda/constructor installation pattern with a ~10 MB static binary that bootstraps a fully functional conda environment in seconds.

## Install

::::{tab-set}

:::{tab-item} Installer (recommended)
```bash
curl -fsSL https://raw.githubusercontent.com/jezdez/conda-express/main/get-cx.sh | sh
```

or with `wget`:

```bash
wget -qO- https://raw.githubusercontent.com/jezdez/conda-express/main/get-cx.sh | sh
```

This detects your platform, downloads the binary, verifies the checksum,
updates your shell profile, and runs `cx bootstrap`.
:::

:::{tab-item} GitHub Releases
Download the binary for your platform from the
[latest release](https://github.com/jezdez/conda-express/releases/latest):

| Platform | File |
|---|---|
| Linux x86_64 | `cx-x86_64-unknown-linux-gnu` |
| Linux ARM64 | `cx-aarch64-unknown-linux-gnu` |
| macOS Intel | `cx-x86_64-apple-darwin` |
| macOS Apple Silicon | `cx-aarch64-apple-darwin` |
| Windows x86_64 | `cx-x86_64-pc-windows-msvc.exe` |
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

::::

## Quick example

```bash
# Bootstrap a conda installation (~3–5 seconds)
cx bootstrap

# Create and activate an environment
cx create -n science python=3.12 numpy scipy
cx shell science

# Use conda normally — cx delegates transparently
cx install -n science pandas matplotlib
```

On first use, cx automatically installs conda and its plugins into `~/.cx` from an embedded lockfile. Subsequent invocations hand off directly to the installed `conda` binary.

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

The `conda-libmamba-solver` and its 27 exclusive native dependencies are excluded by default, reducing the install from 113 to 86 packages.

## Why conda-express?

::::::{grid} 1 1 2 2
:gutter: 3

:::{grid-item-card} Fast
:link: features
:link-type: doc

Bootstrap a full conda environment in ~3–5 seconds from an embedded lockfile. No repodata fetch, no solve at runtime.
:::

:::{grid-item-card} Small
:link: features
:link-type: doc

~10 MB single binary. Installs 86 packages instead of 113 by excluding unnecessary native dependencies.
:::

:::{grid-item-card} Modern
:link: features
:link-type: doc

Uses conda-rattler-solver (resolvo) instead of libmamba. conda-spawn for activation instead of shell profile hacks.
:::

:::{grid-item-card} Simple
:link: quickstart
:link-type: doc

One binary, one command. No Python, no installer, no shell modifications required.
:::

::::::

::::::{grid} 1 1 3 3
:gutter: 3

:::{grid-item-card} Quick start
:link: quickstart
:link-type: doc

Get up and running in minutes.
:::

:::{grid-item-card} Configuration
:link: configuration
:link-type: doc

Customize packages, channels, and exclusions.
:::

:::{grid-item-card} CLI reference
:link: reference/cli
:link-type: doc

All commands and options.
:::

::::::

```{toctree}
:hidden:

quickstart
features
configuration
reference/cli
design
changelog
```
