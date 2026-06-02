# CLI Reference

`cx` is a conda runtime produced by conda-ship. It installs the conda-express
base environment from the stamped runtime lock, then delegates ordinary commands
to the installed `conda` executable.

This page covers the `cx` and `cxz` runtime interface. For the generic
conda-ship runtime interface shared by other generated runtimes, see
{external+conda-ship:doc}`conda-ship's runtime CLI reference <reference/runtime-cli>`.

## Global Options

`--path PATH`
: Use a custom install path instead of the default stamped into the binary.
  For conda-express, the default is `~/.conda/express`. The generic
  install-scheme rules are documented in
  {external+conda-ship:doc}`conda-ship's install location notes <explanation/install-locations-and-ownership>`.

`-v, --verbose`
: Show more runtime progress output.

`-q, --quiet`
: Suppress non-essential runtime output.

`-h, --help`
: Show help.

`-V, --version`
: Show the runtime version. conda-express runtime versions follow the conda
  package version in the runtime lock.

:::{note}
`--path` is a runtime override. Put it before the command:

```bash
cx --path /opt/cx bootstrap
cx --path /opt/cx status
```
:::

## `cx bootstrap`

Bootstrap conda into the runtime install path.

```bash
cx bootstrap [OPTIONS]
```

### Options

`--force`
: Re-bootstrap even if the install path already exists. Existing non-empty
  install paths are removed only when they contain conda-express runtime
  ownership metadata.

`--bundle DIR`
: Bundle directory containing pre-downloaded `.conda` and/or `.tar.bz2`
  package archives. The runtime pre-populates the package cache from this
  directory before installing. Can also be set via `CX_BUNDLE`.

`--offline`
: Disable network access during bootstrap. Packages must already be available
  from the local package cache, `--bundle`, or an embedded `cxz` bundle. Can
  also be set via `CX_OFFLINE`.

:::{note}
`cxz` is the embedded-bundle variant. It detects its embedded package bundle
automatically, so `--bundle` is not needed. Use `--offline` when you want the
runtime to refuse any network access.
:::

### Examples

```bash
# Standard bootstrap into ~/.conda/express
cx bootstrap

# Re-bootstrap the managed install path
cx bootstrap --force

# Bootstrap into a custom location
cx --path /opt/cx bootstrap

# Offline bootstrap from a bundle directory
cx bootstrap --bundle ./packages --offline

# Bootstrap with cxz, using its embedded bundle and no network access
cxz bootstrap --offline
```

## `cx status`

Print conda-express runtime status, including the install path, configured
channels, package metadata, installed package count, and delegate executable.
For conda's own environment information, use `cx info`, which is passed through
to `conda info`.

```bash
cx status
```

:::{admonition} Example output
:class: dropdown

```text
cx 26.5.0
  path:      /Users/you/.conda/express
  channels:  https://conda.anaconda.org/conda-forge/
  packages:  python, conda, conda-rattler-solver, ...
  installed: 97 packages
  delegate:  conda (/Users/you/.conda/express/bin/conda)
```
:::

## `cx shell`

Activate an environment by spawning a new subshell. This is an alias for
`conda spawn`.

```bash
cx shell [ENV] [-- CONDA-SPAWN-ARGS...]
```

### Examples

```bash
# Activate an environment
cx shell myenv

# Leave the environment
exit
```

(cli-cx-uninstall)=
## `cx uninstall`

Remove the conda-express install path and named environments managed by that
install path. The command prints a hint for removing the `cx` binary through
the package manager or install method you used.

```bash
cx uninstall [OPTIONS]
```

### Options

`-y, --yes`
: Skip the interactive confirmation prompt.

### What Gets Removed

1. The managed conda install path, for example `~/.conda/express`
2. Named environments stored under that install path
3. PATH entries added by the standalone installer, when present

### Examples

```bash
# Interactive uninstall
cx uninstall

# Non-interactive uninstall
cx uninstall --yes

# Uninstall a custom install path
cx --path /opt/cx uninstall --yes
```

## `cx help`

Show a getting-started guide with conda-express commands, common workflows, and
links to documentation.

```bash
cx help
```

## Pass-Through Commands

![cx delegates conda commands after bootstrap](../../demos/passthrough.gif)

Any command not listed above is passed through to the installed `conda` binary.
If the install path does not exist yet, `cx` bootstraps it first.

```bash
cx create -n myenv python=3.12
cx install -n myenv numpy
cx list -n myenv
cx env list
cx config --show
cx self update
```

Use global runtime options before the pass-through command:

```bash
cx --path /tmp/cx-smoke create -n test python=3.12
```
