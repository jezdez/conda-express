# conda-express (cx): Feasibility Analysis and Implementation Plan

## 1. Publishing cx to PyPI via maturin (like uv)

### The uv pattern

[uv](https://github.com/astral-sh/uv) has proven that a Rust binary can be distributed on PyPI as platform wheels. The technique:

1. **[maturin](https://github.com/PyO3/maturin)** with `bindings = "bin"` compiles the Rust binary and packages it into a wheel's `scripts/` directory
2. A **tiny Python wrapper** (`python/uv/`) ships alongside with:
   - `_find_uv.py` -- locates the binary via `sysconfig.get_path("scripts")`
   - `__main__.py` -- enables `python -m uv`, finds the binary, and uses `os.execvpe()` (Unix) or `subprocess.run()` (Windows) to hand off
   - `__init__.py` -- exposes `find_uv_bin()` as a public API
3. **Pre-built platform wheels** (~20 MB each) are uploaded to PyPI for every target
4. **sdist fallback** builds from source if no wheel is available (requires Rust toolchain)

### Applying this to cx

cx can use the same approach. This is **much simpler** than publishing conda itself to PyPI:

- No upstream conda changes needed (no pycosat/menuinst optionality, no solver extraction)
- No need to publish conda plugins to PyPI individually
- cx is a single Rust binary -- maturin handles the entire wheel build
- `pip install conda-express` gives you `cx`, which bootstraps conda from conda-forge on first run

```toml
# pyproject.toml for PyPI publishing
[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[project]
name = "conda-express"
description = "A lightweight, single-binary conda bootstrapper — powered by rattler"
requires-python = ">=3.8"

[tool.maturin]
bindings = "bin"
manifest-path = "Cargo.toml"
module-name = "conda_express"
python-source = "python"
strip = true
```

```
python/conda_express/
  __init__.py       # exposes find_cx_bin()
  __main__.py       # python -m conda_express -> exec cx
  _find_cx.py       # locate cx binary in sysconfig paths
```

### Comparison: cx on PyPI vs. conda on PyPI

| Dimension | cx on PyPI (maturin wheel) | conda on PyPI (Python wheel) |
|---|---|---|
| Upstream changes needed | None | pycosat optional, menuinst optional, classic solver extraction, plugin publishing |
| What ships | Single Rust binary (~17 MB wheel) | conda + all deps as Python wheels |
| Solver | rattler (compiled in) + conda-rattler-solver (installed from conda-forge) | conda-rattler-solver (from PyPI, needs py-rattler) |
| conda-forge packages | Full access (bootstraps a real conda env) | Limited to what's on PyPI |
| Install experience | `pip install conda-express && cx bootstrap` | `pip install conda` |
| Time to ship | Weeks (add maturin config + CI) | Months (upstream PRs, coordination) |

**Recommendation**: Publish cx to PyPI now (Phase 1). Pursue upstream conda PyPI publishing as a separate, lower-priority effort that benefits conda's long-term ecosystem health but is not on the critical path.

---

## 2. conda-rattler-solver

**The key enabler for cx's solver strategy.** The [conda-rattler-solver](https://github.com/jaimergp/conda-rattler-solver) project by jaimergp:

- Dependencies: only `conda >=25.5.0` + `py-rattler >=0.21.0`
- [py-rattler](https://pypi.org/project/py-rattler/) is **on PyPI with wheels** for all major platforms
- py-rattler wheels are ~28-31 MB (statically-compiled Rust bindings to rattler/resolvo)
- rattler is the same solver used by pixi, under active development in the conda organization

**Advantages over conda-libmamba-solver:**

- Pure wheel distribution (no libarchive, libsolv, CMake build issues)
- resolvo is the fastest SAT solver in the conda ecosystem
- Single dependency (py-rattler) vs. complex C++ dependency chain

---

## 3. Removing the Classic Solver (upstream -- nice to have)

Not required for cx, but valuable for conda's long-term health.

### Existing PRs by jaimergp (all closed, not merged)

1. **[PR #14131](https://github.com/conda/conda/pull/14131)** (Aug 2024) -- "Refactor classic solver into its own subpackage" -- first attempt, moved code into `conda/plugins/solvers/classic/` as a preview.
2. **[PR #14167](https://github.com/conda/conda/pull/14167)** (Aug 2024) -- "Refactor classic solver out" -- full extraction in favor of `conda-classic-solver` repo. Superseded by #14170.
3. **[PR #14170](https://github.com/conda/conda/pull/14170)** (Aug 2024) -- "Refactor classic solver out" -- most complete attempt. Modified 14 files including `solve.py`, `resolve.py`, `logic.py`, `_logic.py`, `solvers.py`. **Marked stale** Jan 2026.

**Key blocker** (jaimergp, Jan 2025): "We'll need to publish `conda-classic-solver` in both `defaults` and `conda-forge` before we can undo this." Prerequisite PR #14475 (CI fix) was merged.

### What needs to happen

1. Revive PR #14170 or create a new PR based on it
2. Publish conda-classic-solver to both `defaults` and `conda-forge`
3. Move pycosat from conda's core deps to conda-classic-solver only
4. Make solver a required plugin (remove fallback to pycosat)

---

## 4. conda-express (cx) Architecture

### Current implementation (Phase 0 -- complete)

cx is a working Rust binary (~17 MB release) that bootstraps conda from conda-forge. Published as `conda-express` on crates.io, binary name `cx`.

```
cx (Rust binary, ~17 MB release)

First run (cx bootstrap):
  -> parse embedded rattler-lock v6 lockfile (compiled in by build.rs)
  -> download + install 86 packages into ~/.cx/
  -> write .condarc (solver: rattler, auto_activate_base: false)
  -> write conda-meta/frozen (CEP 22 — protects base prefix)
  -> exec: ~/.cx/bin/conda <args>

Subsequent runs (cx <any-conda-command>):
  -> check ~/.cx/ exists
  -> exec: ~/.cx/bin/conda <args>

Updating (conda self update):
  -> conda-self handles via pluggable backend (future)
  -> cx backend delegates to rattler for solve+install with exclusions

Disabled commands (cx activate/deactivate/init):
  -> prints guidance to use conda-spawn instead
```

### Key features implemented

| Feature | Status |
|---|---|
| Single-binary bootstrapper | Done |
| Compile-time lockfile (rattler-lock v6) | Done |
| Post-solve package exclusion (libmamba + 27 deps) | Done |
| conda-rattler-solver as default solver | Done |
| conda-spawn activation model | Done |
| Disabled commands (activate, deactivate, init) | Done |
| Auto-bootstrap on first conda command | Done |
| `.condarc` with recommended settings | Done |
| External lockfile override (`--lockfile`) | Done |
| Live solve fallback (`--no-lock`) | Done |
| Multi-platform CI via pixi | Done |
| Release binary builds via GitHub Actions | Done |
| CEP 22 frozen base prefix | Done |
| Self-update (via conda-self plugin) | Not started |
| PyPI distribution (maturin platform wheels) | Done |
| Reusable GitHub Action | Not started |

### Performance (macOS ARM64)

| Metric | Value |
|---|---|
| Release binary size | ~17 MB |
| Installed packages (base) | 86 |
| Excluded packages (libmamba tree) | 27 |
| Bootstrap time (embedded lockfile) | ~3–5 s |
| Bootstrap time (live solve) | ~7–8 s |

### Architecture

```
pixi.toml              [tool.cx]: packages, channels, excludes
       |
       v
    build.rs           Compile-time: solve + filter + write lockfile
       |
       v
    cx.lock            rattler-lock v6 (embedded via include_str!)
       |
       v
      cx               Single binary (~17 MB release)
       |
       +---> bootstrap -----> install from lockfile (fast path)
       |                       or live solve (fallback)
       |                       write CEP 22 frozen marker
       |
       +---> status -----------> show cx prefix metadata
       |
       +---> activate/deactivate/init --> disabled (guides to conda-spawn)
       |
       +---> <any conda arg> --> hand off to installed conda binary
       |                         (includes `conda self update` via conda-self)
```

### Development environment

cx uses [pixi](https://pixi.sh) to manage the Rust toolchain from conda-forge:

```bash
pixi run build         # cargo build --release
pixi run test          # cargo test
pixi run lint          # fmt-check + clippy
```

CI workflows use `prefix-dev/setup-pixi` to replicate the same environment on all platforms (linux-x64, linux-aarch64, macos-x64, macos-arm64, windows-x64).

### Compile-time lockfile

`build.rs` reads `[tool.cx]` from `pixi.toml`, solves at `cargo build` time, filters exclusions, and writes a rattler-lock v6 lockfile embedded into the binary. A content-hash cache skips the solve when the config hasn't changed.

### Package exclusion

conda on conda-forge hard-depends on `conda-libmamba-solver`. cx removes it and its 27 exclusive native dependencies via a post-solve transitive dependency pruning algorithm, reducing the install from 113 to 86 packages. The `.condarc` is configured with `solver: rattler` so conda never tries to load the missing plugin.

### Rattler crates used

All on crates.io under BSD-3-Clause:

- `rattler` -- package installation engine
- `rattler_conda_types` -- MatchSpec, Channel, Platform, RepoDataRecord
- `rattler_repodata_gateway` -- repodata fetching (sharded/CEP-16)
- `rattler_solve` (resolvo) -- SAT-based dependency solving
- `rattler_lock` -- lockfile read/write (v6 format)
- `rattler_networking` -- auth middleware, OCI support
- `rattler_virtual_packages` -- system capability detection
- `rattler_cache` -- cache directory management

### conda-spawn as the primary activation model

cx ships with conda-spawn and disables `conda activate`, `conda deactivate`, and `conda init`:

- `conda spawn env-name` launches a new shell subprocess with the environment activated
- No `conda init` required, no `.bashrc`/`.zshrc` modifications
- Users add `~/.cx/condabin` to their PATH (one-time setup)

---

## 5. Prior Art: Uniconda (jaimergp)

[uniconda](https://github.com/jaimergp/uniconda) was jaimergp's earlier attempt, using **PyOxidizer** to build a single-binary conda.

### Patches required (conda 22.11.1 era)

1. **`conda/__init__.py`** -- `CONDA_PACKAGE_ROOT`: changed to use `__spec__.origin` instead of `__file__` (PyOxidizer in-memory loading)
2. **`conda/__init__.py`** -- `__version__`: hardcoded (PyOxidizer broke `get_version(__file__)`)
3. **`conda_package_handling`** -- `logging.getLogger(__file__)` to `__name__` (in-memory issue)

### Relevance for cx

- **None of these patches are needed**: The rattler approach installs conda as a real conda package into a real prefix -- `__file__`, `sys.prefix`, all filesystem paths work normally
- The uniconda `cph_logging_name.patch` was a bug that may already be fixed upstream

---

## 6. Self-Update via conda-self (pluggable backend)

### Design principle

cx intentionally does **not** implement its own update command. `conda self update` is the canonical way to update conda across all installation methods (miniconda, miniforge, cx, future pip-installed conda). This requires conda-self to support pluggable updater backends.

### Why cx can't use conda-self's default backend

conda-self currently shells out to `conda install --prefix=sys.prefix conda`. This works in miniconda/miniforge, but in a cx-managed prefix it would **re-introduce conda-libmamba-solver** -- conda on conda-forge hard-depends on it, and the solver has no way to exclude a required dependency. cx's post-solve filtering only works at the rattler level, outside conda's own solver.

Additionally, the base prefix is frozen via CEP 22. conda-self must override the frozen check when updating (it's authorized to do so -- its entire purpose is modifying the base prefix).

### Proposed design: pluggable updater backends

conda-self should define a new plugin hook via conda's existing pluggy system:

```python
class CondaSelfUpdaterSpec:
    @plugins.hookspec
    def conda_self_updaters(self) -> Iterable[plugins.CondaSelfUpdater]:
        """Register a self-update backend."""

@dataclass
class CondaSelfUpdater:
    name: str                    # e.g., "conda", "pip", "cx"
    check_updates: Callable      # check what updates are available
    install_updates: Callable    # perform the update
    priority: int = 0            # higher priority wins if multiple registered
```

### Backend implementations

**conda backend** (default, current behavior):

- `check_updates`: query conda channels for newer versions
- `install_updates`: `conda install --prefix=sys.prefix --override-frozen-env`
- Ships with conda-self itself

**cx backend** (for rattler-bootstrapped installs):

- Detects cx-managed prefix via `.cx.json` marker file
- `check_updates`: two-level check:
  1. Check conda-forge for newer conda/plugin packages
  2. Check GitHub releases (or PyPI) for a newer cx binary
- `install_updates`: shells out to `cx _internal-update` (hidden subcommand) which uses rattler to re-solve with exclusion logic, then optionally downloads and replaces the cx binary itself
- Ships as a small Python package installed into the cx prefix

**pypi backend** (for pip/uv-installed conda):

- `check_updates`: query PyPI for newer versions
- `install_updates`: ideally uses conda-pypi for consistency; falls back to pip/uv
- Could ship with conda-self or as a separate plugin

### User experience

```bash
# All users, regardless of installation method:
conda self update

# cx users specifically — same result:
cx self update          # cx execs to conda, which runs conda-self
```

### Detection logic

Each backend declares a `detect()` method:

- cx backend: checks for `.cx.json` in `sys.prefix`
- conda backend: checks for `conda-meta/conda-*.json` (default fallback)
- pypi backend: checks `importlib.metadata` for PyPI installer origin

### Workaround until the plugin exists

Users can re-bootstrap to get the latest packages:

```bash
cx bootstrap --force
```

---

## 7. Reusable GitHub Action for Building Custom Bootstrapper Binaries

### Concept

A GitHub Action that lets anyone build their own "cx-type" binary with a custom set of packages. This makes the rattler bootstrapper pattern reusable beyond just conda.

### Use cases

- **cx**: `python conda conda-rattler-solver conda-spawn conda-self`
- **nanoforge**: same as cx but with `conda-build rattler-build`
- **custom tool**: `python jupyterlab` -- a single binary that bootstraps a JupyterLab environment
- **ML toolkit**: `python pytorch torchvision` -- one binary to bootstrap a PyTorch env

### Action interface

```yaml
# .github/workflows/build.yml
jobs:
  build:
    uses: jezdez/conda-express/.github/workflows/build.yml@main
    with:
      name: cx
      packages: >
        python >=3.13
        conda >=26.1
        conda-rattler-solver
        conda-spawn
        conda-self
      channels: conda-forge
      platforms: >
        linux-x64
        linux-aarch64
        macos-x64
        macos-arm64
        windows-x64
      entry-point: conda
      self-update-repo: jezdez/conda-express
```

### How it works

1. The action checks out the cx Rust template project
2. Injects the package list, channels, entry point, and self-update config into `pixi.toml`'s `[tool.cx]` section
3. Builds natively on each target platform using GitHub runners and `pixi`
4. Outputs platform-specific binaries as build artifacts or attaches them to a GitHub release

---

## 8. Key Risks and Open Questions

- ~~**First-run time**: Solving + downloading + installing from conda-forge takes ~30-60 seconds on a fast network.~~ **Solved**: Compile-time lockfile reduces bootstrap to ~3–5 s (no solve needed at runtime).
- **Requires network on first run**: No offline-first option without additional work (could pre-populate a package cache in the binary, but adds size).
- ~~**Rust development**: ~500-1000 lines of custom Rust for the bootstrapper.~~ **Done**: ~1200 lines across 5 modules + build.rs.
- **conda-self hook design**: Needs buy-in from conda-self maintainers. The hook API design should be discussed as a proposal before implementation.
- ~~**Cross-compilation**: Building Rust binaries for 5 platforms.~~ **Solved**: Native compilation on each platform using GitHub ARM runners + pixi for toolchain management.
- **PyPI wheel size**: cx platform wheels are ~17 MB each (comparable to uv's ~20 MB). PyPI has a default upload limit of 100 MB per file, so this is well within bounds.
- **maturin + pixi interaction**: The build currently uses pixi for the Rust toolchain. maturin wheel builds may need to happen outside pixi, or pixi can invoke maturin. Needs testing.

### Deprioritized risks (upstream conda, not blocking cx)

- **conda-index dependency** in conda-pypi needs verification for PyPI availability.
- **menuinst on Windows**: `initialize.py` imports `menuinst.knownfolders`/`menuinst.winshortcut` behind `if on_win:` -- needs a try/except guard.

---

## Recommended Implementation Order

### Phase 0: cx Rust prototype -- COMPLETE

~~Build a working cx binary using rattler crates.~~ **Done.** All core functionality implemented and tested:

1. ~~Create a new Rust project with configurable package list~~ -> `pixi.toml` `[tool.cx]` section
2. ~~Implement bootstrap: solve + install packages from conda-forge~~ -> compile-time lockfile + rattler Installer
3. ~~Implement exec: detect existing prefix, exec entry-point binary~~ -> Unix execvp hand-off
4. ~~Package exclusion: remove conda-libmamba-solver and 27 deps~~ -> post-solve transitive pruning
5. ~~Disabled commands: activate, deactivate, init~~ -> guides users to conda-spawn
6. ~~Multi-platform CI/CD~~ -> GitHub Actions with pixi, canary artifacts + release binaries
7. ~~Code restructuring~~ -> 5 modules (cli, config, install, exec, main) + build.rs

### Phase 1: conda-self plugin, PyPI publishing, and GitHub Action

1. **conda-self pluggable updater backend** -- propose hook design to conda-self maintainers, implement hook spec and default conda backend in conda-self, implement cx backend (shells out to `cx _internal-update` for rattler-based solve+install with exclusions). This makes `conda self update` the canonical update command for all conda users.
2. ~~**Publish cx to PyPI via maturin**~~ -- **Done.** `pyproject.toml` with `bindings = "bin"`, Python wrapper in `python/conda_express/`. PyPI and crates.io publishing are consolidated into the unified `release.yml` workflow using trusted publishing (OIDC) for both.
3. Build the reusable GitHub Action for compiling custom bootstrapper binaries from a package list

### Phase 2: Production

1. ~~Documentation and migration guide from miniconda/miniforge~~ **Done.** Sphinx docs using `conda-sphinx-theme` with MyST Markdown, following the `conda-workspaces`/`conda-tasks` pattern. Pages: quickstart, features, configuration, CLI reference, design, changelog. Build via `pixi run -e docs docs`.
2. Explore crate name transfer for `cx` on crates.io (currently published as `conda-express`)

### Upstream work (nice to have -- independent of cx)

These improve conda's ecosystem health but are **not required** for cx:

1. **Make conda-libmamba-solver optional on conda-forge** -- would eliminate the need for cx's post-solve exclusion hack entirely, and make `conda self update` work without a custom backend
2. PR to conda/conda: make pycosat and menuinst optional (revive jaimergp's PR #14170 approach)
3. Publish conda-classic-solver to defaults and conda-forge (unblocks solver extraction)
4. Publish conda-rattler-solver, conda-spawn, conda-self, conda-pypi to PyPI
5. Publish conda itself to PyPI (reclaim the yanked `conda` package name)

---

## Appendix A: Conda itself on PyPI (reference)

This section documents the state of publishing conda as a Python wheel on PyPI. With cx on PyPI (section 1), this is **no longer on the critical path** -- users get conda via `pip install conda-express` (the bootstrapper) rather than `pip install conda` (conda itself). This remains useful context if the conda community decides to pursue direct PyPI publishing independently.

### Conda's dependencies (all already on PyPI with wheels)

| Dependency | Type | PyPI Status |
|---|---|---|
| archspec | Pure Python | Available |
| boltons | Pure Python | Available |
| charset-normalizer | Pure Python | Available |
| conda-package-handling | Pure Python | Available (depends on conda-package-streaming, requests, zstandard) |
| distro | Pure Python | Available |
| frozendict | Pure Python | Available |
| ~~menuinst >=2~~ | Mixed (platform-specific shortcuts) | Available -- **not needed, should be optional** |
| packaging | Pure Python | Available |
| platformdirs | Pure Python | Available |
| pluggy | Pure Python | Available |
| **pycosat** | **C extension** (PicoSAT bindings) | **Should be optional -- belongs in conda-classic-solver only** |
| requests | Pure Python | Available |
| ruamel.yaml | C extension optional | Available with wheels |
| setuptools | Pure Python | Available |
| tqdm | Pure Python | Available |
| truststore | Pure Python | Available |
| zstandard | C extension | Available with platform wheels |

### Critical observation

The conda team has **already commented out** conda-libmamba-solver from the PyPI dependencies:

```python
# Disabled due to conda-libmamba-solver not being available on PyPI
# "conda-libmamba-solver >=25.4.0",
```

Conda also already has a pip-specific entry point at `conda.cli.main_pip:main`.

### What's blocking PyPI publication

1. **No solver on PyPI**: conda-libmamba-solver depends on libmambapy (C++ bindings) -- solved by conda-rattler-solver
2. **pycosat still a hard dependency**: Needs to move to `[project.optional-dependencies]`
3. **menuinst still a hard dependency**: Already runtime-optional but still listed as required
4. **The old conda 4.3.16 package on PyPI is yanked** -- the name is available

### Not yet on PyPI (plugin ecosystem)

- **conda-spawn**: conda-forge only, but pure Python -- easy to publish
- **conda-pypi**: conda-forge only, depends on `conda-index`. Provides `conda pypi install`, `conda pypi convert`, experimental wheel channels in repodata ([draft CEP](https://github.com/conda/ceps/pull/145)), editable/VCS package support, PEP-668 `EXTERNALLY-MANAGED` marker integration
- **conda-rattler-solver**: Not on PyPI yet, but pure Python + py-rattler (which IS on PyPI)
- **conda-self**: conda-forge only, essentially pure Python
