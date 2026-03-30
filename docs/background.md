# Background & rationale

This page explains the ecosystem context behind cx: why it exists, how the
conda solver landscape has evolved, and prior art in single-binary conda
distribution.

## Why cx?

conda is traditionally installed via miniconda or miniforge — large installers
that download and unpack hundreds of megabytes. The process is slow, requires
running a platform-specific installer, and leaves users with a heavyweight base
environment that includes the `conda-libmamba-solver` and its 27 exclusive
native dependencies (libsolv, libarchive, libcurl, spdlog, etc.).

cx takes a different approach: a single ~7 MB Rust binary that bootstraps a
minimal conda installation in seconds using an embedded lockfile. No Python
required to start; no installer framework; no shell profile modifications.

## conda-rattler-solver

The [conda-rattler-solver](https://github.com/jaimergp/conda-rattler-solver)
project is the key enabler for cx's solver strategy:

- Dependencies: only `conda >=25.5.0` + `py-rattler >=0.21.0`
- [py-rattler](https://pypi.org/project/py-rattler/) is on PyPI with wheels
  for all major platforms (~28-31 MB, statically-compiled Rust bindings)
- Uses [resolvo](https://github.com/mamba-org/resolvo), the fastest SAT solver
  in the conda ecosystem
- Same solver used by [pixi](https://pixi.sh), under active development in the
  conda organization

**Advantages over conda-libmamba-solver:**

- Pure wheel distribution — no libarchive, libsolv, CMake build issues
- Single dependency (py-rattler) vs. complex C++ dependency chain
- resolvo outperforms libsolv in benchmarks

Because conda on conda-forge hard-depends on `conda-libmamba-solver`, cx
uses a post-solve transitive dependency pruning algorithm to remove libmamba
and its 27 exclusive dependencies, reducing the install from 113 to 86
packages.

## PyPI distribution (the uv pattern)

cx is distributed on PyPI using the same technique as
[uv](https://github.com/astral-sh/uv):

1. [maturin](https://github.com/PyO3/maturin) with `bindings = "bin"` compiles
   the Rust binary and packages it into a wheel's `scripts/` directory
2. A tiny Python wrapper (`python/conda_express/`) ships alongside with
   `find_cx_bin()` and `__main__.py` for `python -m conda_express`
3. Pre-built platform wheels (~7 MB each) are uploaded to PyPI for every target
4. An sdist fallback builds from source if no wheel is available

### Comparison: cx on PyPI vs. conda on PyPI

| Dimension | cx on PyPI (maturin wheel) | conda on PyPI (Python wheel) |
|---|---|---|
| Upstream changes needed | None | pycosat optional, menuinst optional, solver plugin publishing |
| What ships | Single Rust binary (~7 MB wheel) | conda + all deps as Python wheels |
| Solver | rattler (compiled in) + conda-rattler-solver (from conda-forge) | conda-rattler-solver (from PyPI, needs py-rattler) |
| conda-forge packages | Full access (bootstraps a real conda env) | Limited to what's on PyPI |
| Install experience | `pip install conda-express && cx bootstrap` | `pip install conda` |

### What blocks conda itself on PyPI

The conda community has explored publishing conda directly to PyPI. The
[old conda 4.3.16 package](https://pypi.org/project/conda/) is yanked — the
name is available. The main blockers:

1. **No solver on PyPI**: conda-libmamba-solver depends on libmambapy (C++
   bindings). Solved by conda-rattler-solver.
2. **pycosat still a hard dependency**: Needs to move to optional dependencies.
3. **menuinst still a hard dependency**: Already runtime-optional but still
   listed as required.

conda already has a pip-specific entry point at `conda.cli.main_pip:main` and
has commented out the libmamba dependency in its PyPI config.

### conda's dependencies (all on PyPI)

| Dependency | Type | PyPI Status |
|---|---|---|
| archspec | Pure Python | Available |
| boltons | Pure Python | Available |
| charset-normalizer | Pure Python | Available |
| conda-package-handling | Pure Python | Available |
| distro | Pure Python | Available |
| frozendict | Pure Python | Available |
| packaging | Pure Python | Available |
| platformdirs | Pure Python | Available |
| pluggy | Pure Python | Available |
| **pycosat** | **C extension** | **Should be optional** |
| requests | Pure Python | Available |
| ruamel.yaml | C extension optional | Available with wheels |
| setuptools | Pure Python | Available |
| tqdm | Pure Python | Available |
| truststore | Pure Python | Available |
| zstandard | C extension | Available with platform wheels |

### Plugin ecosystem on PyPI

| Plugin | PyPI status |
|---|---|
| conda-pypi | 0.5.0 on PyPI and conda-forge |
| conda-rattler-solver | Not yet (pure Python + py-rattler) |
| conda-spawn | conda-forge only (pure Python) |
| conda-self | conda-forge only (pure Python) |

## Removing the classic solver (upstream)

Not required for cx, but valuable for conda's long-term health.

### Prior work by jaimergp

Three PRs attempted to extract the classic (pycosat-based) solver:

1. [PR #14131](https://github.com/conda/conda/pull/14131) (Aug 2024) —
   moved code into `conda/plugins/solvers/classic/` as a preview
2. [PR #14167](https://github.com/conda/conda/pull/14167) (Aug 2024) —
   full extraction in favor of `conda-classic-solver` repo
3. [PR #14170](https://github.com/conda/conda/pull/14170) (Aug 2024) —
   most complete attempt (14 files modified). Marked stale Jan 2026.

**Key blocker** (jaimergp, Jan 2025): *"We'll need to publish
`conda-classic-solver` in both `defaults` and `conda-forge` before we can
undo this."*

### What needs to happen

1. Revive PR #14170 or create a new PR based on it
2. Publish conda-classic-solver to both `defaults` and `conda-forge`
3. Move pycosat from conda's core deps to conda-classic-solver only
4. Make solver a required plugin (remove fallback to pycosat)

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
prefix — `__file__`, `sys.prefix`, all filesystem paths work normally.
cx doesn't embed Python or conda into the binary; it bootstraps them into
a standard prefix on first run.
