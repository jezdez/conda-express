# Configuration

## Build-Time Configuration

conda-express is a downstream distribution built by
{external+conda-ship:doc}`conda-ship <index>`. The package set is maintained in
the `runtime` source environment in [pyproject.toml](../pyproject.toml), and
`[tool.conda-ship]` tells conda-ship which solved environment becomes the
runtime:

```toml
[tool.conda-ship]
runtime = "cx"
delegate = "conda"
layout = "online"
source-environment = "runtime"
exclude = ["conda-libmamba-solver"]
docs-url = "https://jezdez.github.io/conda-express/"
install-scheme = "conda-home"
install-name = "express"
```

The release workflows override only `layout`:

- `layout = "online"` builds `cx`
- `layout = "embedded"` builds `cxz`

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
| `conda ==26.5.2` | Package manager |
| `conda-rattler-solver` | Default solver |
| `conda-spawn >=0.1.0` | Subshell activation |
| `conda-completion >=0.2.0` | Shell completion |
| `conda-pypi` | PyPI interoperability |
| `conda-self` | Base environment self-management |
| `conda-global` | Global tool environments |
| `conda-workspaces >=0.5.0` | Workspace manifests and tasks |

`conda-libmamba-solver` is excluded from the derived runtime lock because
conda-express uses `conda-rattler-solver`.

## Versioning Policy

conda-express release versions follow the exact conda package version in the
runtime lock. A `26.5.2` conda-express release bootstraps conda `26.5.2` on
every supported platform.

Use `.postN` releases for conda-express-only rebuilds that keep the same conda
runtime package, for example `26.5.2.post1`.

See {doc}`features` for why the distribution version follows the conda runtime
version.

## Install Location

The stamped install scheme is `conda-home` and the install name is `express`.
That makes the default install path:

```text
~/.conda/express
```

Users can override the install path for a command with the runtime-level
`--path` option:

```bash
cx --path /opt/cx bootstrap
cx --path /opt/cx status
```

`--path` is intentionally a runtime option, not build configuration. The
published `cx` binary stays cross-platform while advanced users, CI jobs, and
installer smoke tests can choose a local path at runtime.

conda-ship owns the generic install scheme behavior and runtime ownership
metadata. See
{external+conda-ship:doc}`install locations and ownership <explanation/install-locations-and-ownership>`
for the builder-level rules behind `conda-home`, `user-data`, install names,
and prefix ownership checks.

## Runtime Metadata

Generated conda-ship runtimes write ownership metadata into each managed
install path. `cx status`, `cx bootstrap --force`, and `cx uninstall` use that
metadata to avoid taking over or deleting unrelated conda installations.

The managed base environment is also protected with a
[CEP 22](https://conda.org/learn/ceps/cep-0022/) frozen marker after bootstrap.
Day-to-day work should happen in named environments:

```bash
cx create -n analysis python=3.12 numpy pandas
cx shell analysis
```

## Runtime Environment Variables

These environment variables control bootstrap behavior at runtime. They are
useful in native installer post-install scripts, container builds, and CI
pipelines.

| Variable | Effect |
|---|---|
| `CX_BUNDLE` | Bundle directory containing `.conda` / `.tar.bz2` archives to pre-populate the package cache from, equivalent to `--bundle` |
| `CX_OFFLINE` | Disable network access during bootstrap when set to any truthy value, equivalent to `--offline`. Values `0` and `false` are treated as unset |

```bash
CX_BUNDLE=/Library/Application\ Support/cx/packages CX_OFFLINE=1 cx bootstrap
```

For custom package sets or new binary distributions, use
{external+conda-ship:doc}`conda-ship <index>` directly instead of treating this
repository as a generic builder.
