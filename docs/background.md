# Why cx Exists

This page explains the ecosystem context behind cx: why it exists, how the
conda solver landscape has evolved, and prior art in single-binary conda
distribution.

## Why cx?

conda is traditionally installed via Anaconda Distribution, Miniconda, or
Miniforge: widely used installer distributions. That path is well-established,
but it also makes installation depend on a platform-specific installer and can
leave users with a heavier base environment. `cx` also excludes
`conda-libmamba-solver` and its 27 exclusive native dependencies (libsolv,
libarchive, libcurl, spdlog, etc.) because it configures
`conda-rattler-solver` instead.

cx evolves that bootstrap path: a single native Rust binary (7-11 MB) that
bootstraps a managed conda base environment using a built-in runtime lock. No
local Python installation or installer framework is required before bootstrap.

For air-gapped or restricted-network environments, `cxz` embeds the locked
package archives directly into the binary (50-95 MB depending on platform).
One file, no separate package directory; `cxz bootstrap --offline` installs
conda from the embedded bundle. See {doc}`features` for details.

## conda-rattler-solver

The [conda-rattler-solver](https://github.com/jaimergp/conda-rattler-solver)
project is the key enabler for cx's solver strategy:

- Dependencies: only `conda >=25.5.0` + `py-rattler >=0.21.0`
- [py-rattler](https://pypi.org/project/py-rattler/) is on PyPI with platform
  wheels (13-33 MB depending on platform, statically-compiled Rust bindings)
- Uses [resolvo](https://github.com/mamba-org/resolvo), the Rust solver also
  used by [pixi](https://pixi.sh)

**Why it fits cx:**

- avoids the libmamba/libsolv native dependency chain in the bootstrapped base
- keeps the solver choice aligned with the rattler-based bootstrap path
- still uses conda's plugin mechanism instead of replacing conda's CLI

Because conda on conda-forge hard-depends on `conda-libmamba-solver`, cx
uses a post-solve transitive dependency pruning algorithm to remove libmamba
and its exclusive dependencies, reducing the install from roughly 130-140
packages to about 103-109 packages, depending on platform.

## What blocks conda itself on PyPI

The conda community has explored publishing conda directly to PyPI. That work
has separate constraints from cx: conda's own dependency set, solver plugin
availability, and how conda should behave when installed by `pip`.

cx does not try to solve that upstream packaging question. It uses conda-forge
packages to create a conda-managed base prefix, then delegates to the installed
conda executable.

## Removing the classic solver (upstream)

Not required for cx, but valuable for conda's long-term health.

### Prior work by jaimergp

Three PRs attempted to extract the classic (pycosat-based) solver:

1. [PR #14131](https://github.com/conda/conda/pull/14131) (Aug 2024) —
   moved code into `conda/plugins/solvers/classic/` as a preview
2. [PR #14167](https://github.com/conda/conda/pull/14167) (Aug 2024) —
   full extraction in favor of `conda-classic-solver` repo
3. [PR #14170](https://github.com/conda/conda/pull/14170) (Aug 2024) —
   broad extraction attempt touching the classic solver path.

**Key blocker** (jaimergp, Jan 2025): *"We'll need to publish
`conda-classic-solver` in both `defaults` and `conda-forge` before we can
undo this."*

### Likely upstream work

The exact plan belongs upstream in conda. At a high level, classic-solver
extraction would likely require a packaged `conda-classic-solver`, dependency
changes around `pycosat`, and a clear plugin requirement for solver behavior.

## Prior art: uniconda

[uniconda](https://github.com/jaimergp/uniconda) was jaimergp's earlier
attempt at a single-binary conda, using **PyOxidizer** to build a static
binary embedding Python and conda together.

### Patches required (conda 22.11.1 era)

1. `conda/__init__.py` — `CONDA_PACKAGE_ROOT`: changed to use
   `__spec__.origin` instead of `__file__` (PyOxidizer in-memory loading)
2. `conda/__init__.py` — `__version__`: hardcoded (PyOxidizer broke
   `get_version(__file__)`)
3. `conda_package_handling` — `logging.getLogger(__file__)` to `__name__`
   (in-memory issue)

### Why cx doesn't need these patches

The rattler approach installs conda as a real conda package into a real
prefix, so conda sees standard filesystem paths such as `__file__` and
`sys.prefix`.
cx doesn't embed Python or conda into the binary; it bootstraps them into
a standard prefix on first run. The `cxz` variant embeds the compressed
package *archives* (not an unpacked Python), so the installed prefix is
still a standard conda environment — just sourced from the binary instead
of the network.
