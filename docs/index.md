# conda-express

A lightweight, single-binary bootstrapper for [conda](https://github.com/conda/conda), powered by [rattler](https://github.com/conda/rattler). The `cx` binary is short for **c**onda e**x**press.

cx replaces the miniconda/constructor installation pattern with a ~17 MB static binary that bootstraps a fully functional conda environment in seconds.

## Install

::::{tab-set}

:::{tab-item} Homebrew
```bash
brew tap jezdez/conda-express https://github.com/jezdez/conda-express
brew install jezdez/conda-express/cx
```

Works on macOS and Linux. Upgrades via `brew upgrade cx`.
:::

:::{tab-item} Shell script
**macOS / Linux:**

```bash
curl -fsSL https://jezdez.github.io/conda-express/get-cx.sh | sh
```

**Windows (PowerShell):**

```powershell
powershell -ExecutionPolicy ByPass -c "irm https://jezdez.github.io/conda-express/get-cx.ps1 | iex"
```
:::

:::{tab-item} pip / cargo
```bash
pip install conda-express
```

```bash
cargo install conda-express
```
:::

::::

See the {doc}`quickstart <quickstart>` for all installation methods.

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

:::{grid-item-card} {octicon}`zap` Fast
:link: features
:link-type: doc

Bootstrap a full conda environment in ~3–5 seconds from an embedded lockfile. No repodata fetch, no solve at runtime.
:::

:::{grid-item-card} {octicon}`package` Small
:link: features
:link-type: doc

~17 MB single binary. Installs 86 packages instead of 113 by excluding unnecessary native dependencies.
:::

:::{grid-item-card} {octicon}`rocket` Modern
:link: features
:link-type: doc

Uses conda-rattler-solver (resolvo) instead of libmamba. conda-spawn for activation instead of shell profile hacks.
:::

:::{grid-item-card} {octicon}`check-circle` Simple
:link: quickstart
:link-type: doc

One binary, one command. No Python, no installer, no shell modifications required.
:::

::::::

::::::{grid} 1 1 2 3
:gutter: 3

:::{grid-item-card} {octicon}`rocket` Quick start
:link: quickstart
:link-type: doc

Get up and running in minutes.
:::

:::{grid-item-card} {octicon}`tools` Build custom cx binaries
:link: guides/custom-builds
:link-type: doc

Bake your own packages into a cx binary.
:::

:::{grid-item-card} {octicon}`terminal` CLI reference
:link: reference/cli
:link-type: doc

All commands and options.
:::

:::{grid-item-card} {octicon}`play` GitHub Action reference
:link: reference/github-action
:link-type: doc

Inputs, outputs, and behavior of the composite action and reusable workflow.
:::

:::{grid-item-card} {octicon}`gear` Configuration
:link: configuration
:link-type: doc

Build-time and runtime settings.
:::

:::{grid-item-card} {octicon}`cpu` Architecture
:link: design
:link-type: doc

Design decisions and internals.
:::

::::::

```{toctree}
:hidden:
:caption: Tutorials

quickstart
```

```{toctree}
:hidden:
:caption: How-to guides

guides/custom-builds
```

```{toctree}
:hidden:
:caption: Reference

reference/cli
reference/github-action
reference/installer
configuration
```

```{toctree}
:hidden:
:caption: Explanation

features
design
```

```{toctree}
:hidden:
:caption: Project

changelog
```
