# Configuration

## Build-Time Configuration

conda-express is a downstream distribution built by
{external+conda-ship:doc}`conda-ship <index>`. The package set is maintained in
the `runtime` source environment in [pyproject.toml](../pyproject.toml), and
`[tool.conda-ship]` tells conda-ship which solved environment becomes the
runtime:

```toml
[tool.conda-ship]
runtime-name = "cx"
runtime-version = { from = "project-metadata" }
delegate-executable = "conda"
artifact-layout = "online"
source-environment = "runtime"
exclude-packages = ["conda-libmamba-solver"]
condarc-file = "runtime.condarc"
freeze-base = true
docs-url = "https://jezdez.github.io/conda-express/"
install-scheme = "conda-home"
install-name = "express"
```

conda-ship installs `runtime.condarc` as `<root-prefix>/.condarc`:

```yaml
solver: rattler
auto_activate_base: false
notify_outdated_conda: false
show_channel_urls: true
channels:
  - "https://conda.anaconda.org/conda-forge/"
```

`freeze-base = true` writes the CEP 22 marker after bootstrap. These are cx
defaults, not conda-ship defaults.

The release workflows override `artifact-layout` and `artifact-name`:

- `artifact-layout = "online"` with `artifact-name = "cx"` builds `cx`
- `artifact-layout = "embedded"` with `artifact-name = "cxz"` builds `cxz`

The source environment is solved and locked by Pixi. conda-ship derives the
runtime lock from that committed source lock rather than accepting ad hoc
package and channel lists in the workflow. See
{external+conda-ship:doc}`source locks and runtime locks <explanation/source-locks-and-runtime-locks>`
for the generic distinction between project input and runtime output.

:::{note}
This page documents conda-express's distribution configuration. To build a
different runtime, use
{external+conda-ship:doc}`conda-ship's configuration reference <reference/configuration>`
instead of copying conda-express's defaults.
:::

## Runtime Package Set

The `runtime` source environment installs:

| Package | Role |
|---|---|
| `python >=3.12` | Python runtime for conda |
| `conda =={{ conda_runtime_version }}` | Package manager |
| `conda-rattler-solver` | Default solver |
| `conda-spawn >=0.1.0` | Subshell activation |
| `conda-completion >=0.3.0` | Shell completion for `cx` and conda plugin commands |
| `conda-exec >=0.3.0` | Ephemeral package execution and PEP 723 script workflows |
| `conda-pypi` | PyPI interoperability |
| `conda-self` | Base environment self-management and reset |
| `conda-global` | Global tool environments |
| `conda-workspaces >=0.7.0` | Workspace manifests and tasks |

`conda-libmamba-solver` is excluded from the derived runtime lock because
conda-express uses `conda-rattler-solver`.

## Versioning Policy

conda-express release versions follow the exact conda package version in the
runtime lock. A `{{ conda_express_release }}` conda-express release bootstraps
conda `{{ conda_runtime_version }}` on every supported platform.

Use `.postN` releases for conda-express-only rebuilds that keep the same conda
runtime package, for example `{{ conda_express_post_release_example }}`.

See {doc}`features` for why the distribution version follows the conda runtime
version.

## Install Location

The stamped install scheme is `conda-home` and the install name is `express`.
That makes the default install path:

```text
~/.conda/express
```

Users can override the install path with `CX_PREFIX`:

```bash
CX_PREFIX=/opt/cx cx info
CX_PREFIX=/opt/cx cx create -n analysis python=3.12
```

The published `cx` binary stays cross-platform while advanced users, CI jobs,
and installer smoke tests can choose a local path at runtime. Keep
`CX_PREFIX` set for every command that should use the non-default prefix.

conda-ship owns the generic install scheme behavior and runtime ownership
metadata. See
{external+conda-ship:doc}`install locations and ownership <explanation/install-locations-and-ownership>`
for the builder-level rules behind `conda-home`, `user-data`, install names,
and prefix ownership checks.

## Runtime Metadata

Generated conda-ship runtimes write ownership metadata into each managed
install path. Automatic bootstrap and interrupted-bootstrap recovery use that
metadata to avoid taking over unrelated conda installations.

Bootstrap also writes standard conda prefix metadata:

- `conda-meta/history`
- `conda-meta/initial-state.explicit.txt`

`history` lets conda recognize `~/.conda/express` as an environment. The
initial-state file records the exact packages installed from the stamped
runtime lock, including package URLs and checksums. The included `conda-self`
plugin can use that snapshot to reset the existing managed base prefix:

```bash
cx self reset --snapshot installer-exact
```

That snapshot belongs to the existing prefix. Installing a newer `cx` binary
does not replace it, and reset does not apply the newer binary's stamped
runtime lock.

The managed base environment is also protected with a
[CEP 22](https://conda.org/learn/ceps/cep-0022/) frozen marker after bootstrap.
Day-to-day work should happen in named environments:

```bash
cx create -n analysis python=3.12 numpy pandas
cx spawn analysis
```

## Runtime Environment Variables

These environment variables control bootstrap behavior at runtime. They are
useful in native installer post-install scripts, container builds, and CI
pipelines.

| Variable | Effect |
|---|---|
| `CX_PREFIX` | Override the managed prefix path. The default is `~/.conda/express` |
| `CX_BUNDLE` | Bundle directory containing `.conda` / `.tar.bz2` archives used to pre-populate the package cache |
| `CX_OFFLINE` | Disable network access during bootstrap when set to any truthy value. Values `0` and `false` are treated as unset |

```bash
CX_BUNDLE=/Library/Application\ Support/cx/packages CX_OFFLINE=1 cx info
```

For custom package sets or new binary distributions, use
{external+conda-ship:doc}`conda-ship <index>` directly instead of treating this
repository as a generic builder.
