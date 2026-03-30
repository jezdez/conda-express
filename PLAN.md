# conda-express (cx): Feasibility Analysis and Implementation Plan

> For architecture, current features, and file structure see [DESIGN.md](DESIGN.md).
>
> For the browser/WASM work (cx-wasm, conda-emscripten, JupyterLite), see the [cx-wasm section](#6-cx-wasm-conda-in-the-browser) below.

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

### Applying this to cx -- DONE

cx uses the same approach. `pip install conda-express` gives you `cx`, which bootstraps conda from conda-forge on first run. Published via trusted publishing (OIDC) in the `release.yml` workflow.

### Comparison: cx on PyPI vs. conda on PyPI

| Dimension | cx on PyPI (maturin wheel) | conda on PyPI (Python wheel) |
|---|---|---|
| Upstream changes needed | None | pycosat optional, menuinst optional, classic solver extraction, plugin publishing |
| What ships | Single Rust binary (~17 MB wheel) | conda + all deps as Python wheels |
| Solver | rattler (compiled in) + conda-rattler-solver (installed from conda-forge) | conda-rattler-solver (from PyPI, needs py-rattler) |
| conda-forge packages | Full access (bootstraps a real conda env) | Limited to what's on PyPI |
| Install experience | `pip install conda-express && cx bootstrap` | `pip install conda` |
| Time to ship | ~~Weeks~~ Done | Months (upstream PRs, coordination) |

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

## 4. Prior Art: Uniconda (jaimergp)

[uniconda](https://github.com/jaimergp/uniconda) was jaimergp's earlier attempt, using **PyOxidizer** to build a single-binary conda.

### Patches required (conda 22.11.1 era)

1. **`conda/__init__.py`** -- `CONDA_PACKAGE_ROOT`: changed to use `__spec__.origin` instead of `__file__` (PyOxidizer in-memory loading)
2. **`conda/__init__.py`** -- `__version__`: hardcoded (PyOxidizer broke `get_version(__file__)`)
3. **`conda_package_handling`** -- `logging.getLogger(__file__)` to `__name__` (in-memory issue)

### Relevance for cx

- **None of these patches are needed**: The rattler approach installs conda as a real conda package into a real prefix -- `__file__`, `sys.prefix`, all filesystem paths work normally
- The uniconda `cph_logging_name.patch` was a bug that may already be fixed upstream

---

## 5. Self-Update via conda-self (pluggable backend) ([#4](https://github.com/jezdez/conda-express/issues/4))

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

## 6. cx-wasm: conda in the Browser

cx-wasm compiles the rattler-based solver and package extractor to WebAssembly, enabling `conda install` to run entirely client-side. Combined with the `conda-emscripten` plugin and a JupyterLite deployment, users can run `%cx install numpy` in a browser notebook with no server.

### Architecture

1. **cx-wasm crate** (`crates/cx-wasm/`) — Rust crate compiled to `wasm32-unknown-unknown` via `wasm-pack`. Exports:
   - `cx_fetch_and_solve` — combined repodata fetch + resolvo solve. Reads shards from the JS prefetch cache when available, falling back to sync XHR.
   - `cx_extract_package` — streaming `.conda`/`.tar.bz2` extraction to MEMFS.
   - `cx_get_shard_urls` — computes shard URLs for a set of package names (used by the shard prefetch).
   - `cx_decode_shard_deps` — decodes raw zstd+msgpack shard bytes and extracts dependency package names (used by the shard prefetch).

2. **conda-emscripten plugin** (`conda-emscripten/`) — Python conda plugin providing:
   - `CxWasmSolver` (`CONDA_SOLVER=cx-wasm`) — delegates to `js.fetch_and_solve`, round-trips solutions through JSON
   - WASM-based package extractor — calls `js.cx_extract_package` with explicit `Uint8Array` conversion; Python streaming tarfile fallback for `.tar.bz2`
   - Virtual packages (`__unix`, `__emscripten`)
   - `%cx` and `%conda` IPython magics (via `%load_ext conda_emscripten`)
   - MEMFS bootstrap (creates `conda-meta/`, `.condarc`, sets env vars)
   - Shared library loading after install (`ctypes.CDLL` with `RTLD_GLOBAL`)
   - Runtime patches: urllib3 sync XHR, no-seek `download_inner`, WASM `ExtractPackageAction`, subprocess no-op, MEMFS stubs

3. **cx-jupyterlite** (`cx-jupyterlite/`) — TypeScript JupyterLab federated extension that intercepts `execute_request` messages on the main thread and rewrites bare `conda` commands to `%cx` so the IPython magic handles them. Also catches `%conda` and `!conda` forms.

4. **cx-wasm-kernel** (`recipes/cx-wasm-kernel/`) — conda package that places the WASM files and `cx_wasm_bridge` Python module into a xeus-python kernel prefix. The bridge:
   - Loads WASM via blob URLs and registers JS bridge functions on the global scope using `js.Function.new()` to avoid pyjs proxy wrapping
   - Runs an **async shard prefetch** at startup — traverses the dependency graph of installed packages, fetching all repodata shards in parallel via JavaScript `fetch()` into a JS-side `Map` cache
   - The prefetch uses Rust functions (`cx_get_shard_urls`, `cx_decode_shard_deps`) for efficient URL computation and dependency extraction

5. **JupyterLite demo** (`lite/`) — builds a static JupyterLite site with xeus-python + the above packages. `lite/build.py --with-local` includes locally-built packages and builds cx-jupyterlite; `lite/build.py` uses public channels only.

6. **Web Worker demo** (`crates/cx-wasm/www/`) — standalone browser demo using Comlink for RPC, IndexedDB for caching (~50 MB bootstrap cache), and pyjs for Python execution.

### Performance: two-phase fetch/solve

Sharded repodata (CEP-16) requires fetching an individual shard for each package name. In a Web Worker, only synchronous XHR is available during the solve phase (Python blocks the worker thread). Making hundreds of sequential sync XHR requests originally caused solves to take 10-12 seconds.

The solution separates fetching from solving:

- **Phase 1 (async, kernel startup):** `_prefetch_installed()` traverses the dependency graph level by level, fetching all repodata shards in parallel via async `fetch()` and caching them in a JavaScript `Map`.
- **Phase 2 (sync, user command):** When the solver requests shards, it reads from the cache. No network requests during the solve.

Result: solve time dropped from **11.85s to 0.21s** (56x speedup). Total `%conda install lz4` time is ~3.5 seconds.

### Status

| Feature | Status |
|---|---|
| cx-wasm crate (solver + extractor to WASM) | Done |
| Sharded repodata (CEP-16) fetch and decode in Rust | Done |
| Combined fetch-and-solve (`cx_fetch_and_solve`) | Done |
| Async shard prefetch at kernel startup | Done |
| Shard dependency extraction (`cx_decode_shard_deps`) | Done |
| Streaming package extraction (`.conda` + `.tar.bz2`) | Done |
| conda-emscripten plugin (solver, extractor, vpkgs, magic) | Done |
| `%conda` / `%cx` IPython magics (via `%load_ext conda_emscripten`) | Done |
| cx-jupyterlite extension (bare `conda` command interception) | Done |
| MEMFS patches (no-seek download, WASM extractor, subprocess no-op) | Done |
| Shared library loading for C extensions (`ctypes.CDLL` + `RTLD_GLOBAL`) | Done |
| cx-wasm-kernel conda package | Done |
| JupyterLite demo site | Done |
| GitHub Pages deployment (docs + `/demo/`) | Done |
| Web Worker architecture (Comlink, IndexedDB) | Done |
| Submit packages to emscripten-forge | Not started |
| PyPI wheel support (repodata v3 / conda-pypi) | Research complete |
| npm package (`@conda-express/web`) | Not started (deprioritized) |

### Pending: emscripten-forge publishing ([#2](https://github.com/jezdez/conda-express/issues/2))

| Package | Type | Notes |
|---|---|---|
| `conda` | noarch | Patched 26.1.1 with emscripten patches |
| `conda-emscripten` | noarch | Solver + extractor + vpkgs + magic plugin |
| `cx-wasm-kernel` | noarch | WASM files + Python bridge |
| `frozendict` | noarch | 2.4.6 pure Python (may already exist on emscripten-forge) |

Once published, `lite/environment.yml` can add these as dependencies, eliminating the need for `--with-local` builds and simplifying the GitHub Pages CI.

### Future: PyPI wheel support via conda-pypi (repodata v3) ([#3](https://github.com/jezdez/conda-express/issues/3))

The [conda-pypi](https://github.com/conda/conda-pypi) plugin teaches conda how to install Python wheels (`.whl` files) by registering a `CondaPackageExtractor` hook. A companion project, [conda-pypi-test](https://github.com/conda-incubator/conda-pypi-test), provides a development conda channel that indexes ~500K pure Python wheels from PyPI. Together, these would allow `conda install requests` to install the wheel directly from PyPI in the browser -- extending cx-wasm beyond emscripten-forge packages.

#### conda-pypi-test channel

- URL: `https://github.com/conda-incubator/conda-pypi-test/releases/download`
- Indexes ~500K pure Python wheels (`-none-any.whl` only) from PyPI, latest version per package
- Not sharded -- plain `repodata.json` (+ `.zst` compressed variant)
- Only `noarch` packages; uses the [grayskull PyPI-to-conda mapping](https://raw.githubusercontent.com/regro/cf-graph-countyfair/master/mappings/pypi/grayskull_pypi_mapping.json) for name normalization
- Each entry has a `url` field pointing directly to PyPI (e.g., `https://files.pythonhosted.org/.../requests-2.32.3-py3-none-any.whl`); the channel itself does not host wheel files
- Repodata is generated by `generate.py` which fetches package metadata from PyPI's JSON API, converts dependencies using grayskull mapping, and outputs conda-compatible repodata
- A `blocklist.txt` excludes packages that shadow conda names or bundle native code despite being marked noarch

#### Repodata v3 format (in progress)

The wheel entries are transitioning to a new `v3` top-level key in repodata, defined by three draft CEPs:

- [CEP 111](https://github.com/conda/ceps/pull/111): Conditional dependencies, extras, and flags
- [CEP 145](https://github.com/conda/ceps/pull/145): Repodata wheel support
- [CEP 146](https://github.com/conda/ceps/pull/146): Backwards-compatible repodata update strategy

Timeline of implementation:

| Date | Component | Change |
|------|-----------|--------|
| Feb 26, 2026 | `rattler_conda_types` v0.43.5 | Added `ExperimentalV3Packages` struct with `v3` key and `whl` sub-key. **Removed** old `packages.whl` key ([PR #2093](https://github.com/conda/rattler/pull/2093)) |
| Mar 9, 2026 | py-rattler v0.23.0 | Exposes v3 support to Python. Changed conditional syntax from `; if` to `when` key ([PR #2007](https://github.com/conda/rattler/pull/2007)) |
| Mar 17, 2026 | conda-pypi [PR #273](https://github.com/conda/conda-pypi/pull/273) | Merged. Updates plugin to generate v3 repodata with `extra_depends` and `[when="..."]` conditionals |
| Pending | conda-pypi-test [PR #19](https://github.com/conda-incubator/conda-pypi-test/pull/19) | Blocked on conda-pypi release ([#277](https://github.com/conda/conda-pypi/issues/277)). Updates `generate.py` to put entries under `v3.whl` |

The v3 repodata format puts wheel records under `v3.whl` with `WhlPackageRecord` entries that include a `url` field (absolute PyPI URL or channel-relative path):

```json
{
  "packages": { },
  "packages.conda": { },
  "v3": {
    "whl": {
      "requests-2.32.3-py3_none_any_0": {
        "url": "https://files.pythonhosted.org/.../requests-2.32.3-py3-none-any.whl",
        "name": "requests",
        "version": "2.32.3",
        "build": "py3_none_any_0",
        "depends": ["charset-normalizer>=2,<4", "python"],
        "extra_depends": { "socks": ["pysocks>=1.5.6,!=1.5.7"] },
        "subdir": "noarch",
        "noarch": "python"
      }
    }
  }
}
```

#### conda-pypi plugin structure

| Hook | Function | WASM-compatible |
|------|----------|:---:|
| `conda_package_extractors` | Registers `.whl` extractor using pure-Python `installer` lib. Extracts wheels and creates conda metadata (`info/index.json`, `info/paths.json`, `info/link.json`) | Yes |
| `conda_subcommands` | `conda pypi` subcommand (invokes pip via subprocess) | No |
| `conda_post_commands` | Post-install hooks (runs pip via subprocess) | No |

Only the extractor hook is needed for WASM. The `installer` library is pure Python and should work in Emscripten. The subprocess-dependent hooks would be neutralized by the existing subprocess no-op patch.

#### cx-wasm compatibility analysis

cx-wasm already uses `rattler_conda_types` v0.43.5, so the Rust deserialization layer supports the `v3.whl` format. The `RepoData.into_repo_data_records()` method in rattler already handles `WhlPackageRecord` correctly, converting URLs and creating proper `RepoDataRecord` entries.

Gaps to bridge:

1. **Repodata parsing in cx-wasm**: `parse_repodata_text` in `sharded.rs` only iterates `repo.packages` + `repo.conda_packages`. It ignores `experimental_v3` entirely. Needs to also chain `experimental_v3.whl` records, converting `WhlPackageRecord` URLs into `RepoDataRecord` entries.
2. **Absolute download URLs**: `WhlPackageRecord` entries have `url` pointing to PyPI (`https://files.pythonhosted.org/...`), not a channel-relative path. The download logic in `_memfs_download_inner` already handles arbitrary URLs, but the transaction/extractor pipeline needs to recognize `.whl` file extensions.
3. **Repodata size**: ~500K packages in a single `repodata.json` is very large. Sync XHR for a file this big would be extremely slow. Options: use `.zst` compression, subset the repodata, or wait for sharded repodata support on the channel.
4. **Wheel extraction**: The conda-pypi extractor hook and `installer` library need to be available in the JupyterLite environment. Both are noarch/pure Python packages.
5. **Defensive patching**: The conda-pypi `post_command` hook calls `run_pip_install` (subprocess). The existing subprocess no-op patch should prevent errors, but the post_command hook may still attempt to run and could log confusing errors. A targeted patch to disable conda-pypi's post_command hooks in WASM would be cleaner.

---

## 7. Key Risks and Open Questions

- ~~**First-run time**: Solving + downloading + installing from conda-forge takes ~30-60 seconds on a fast network.~~ **Solved**: Compile-time lockfile reduces bootstrap to ~3–5 s (no solve needed at runtime).
- **Requires network on first run**: No offline-first option without additional work (could pre-populate a package cache in the binary, but adds size).
- ~~**Rust development**: ~500-1000 lines of custom Rust for the bootstrapper.~~ **Done**: ~1200 lines across 5 modules + build.rs.
- **conda-self hook design**: Needs buy-in from conda-self maintainers. The hook API design should be discussed as a proposal before implementation.
- ~~**Cross-compilation**: Building Rust binaries for 5 platforms.~~ **Solved**: Native compilation on each platform using GitHub ARM runners + pixi for toolchain management.
- **PyPI wheel size**: cx platform wheels are ~17 MB each (comparable to uv's ~20 MB). PyPI has a default upload limit of 100 MB per file, so this is well within bounds.
- ~~**maturin + pixi interaction**: The build currently uses pixi for the Rust toolchain. maturin wheel builds may need to happen outside pixi, or pixi can invoke maturin. Needs testing.~~ **Solved**: maturin builds run outside pixi in the release workflow; pixi manages the Rust toolchain for local development only.

### Deprioritized risks (upstream conda, not blocking cx)

- **conda-index dependency** in conda-pypi needs verification for PyPI availability.
- **menuinst on Windows**: `initialize.py` imports `menuinst.knownfolders`/`menuinst.winshortcut` behind `if on_win:` -- needs a try/except guard.

---

## Recommended Implementation Order

### Phase 0: cx Rust prototype -- COMPLETE

All core functionality implemented and tested. See [DESIGN.md](DESIGN.md) for the full feature table and architecture.

### Phase 1: Ecosystem integration -- IN PROGRESS

| Task | Status |
|---|---|
| Publish cx to PyPI via maturin | Done |
| Publish cx to crates.io | Done |
| Reusable GitHub Action (composite action + workflow) | Done |
| Build-time env var overrides (`CX_PACKAGES`, etc.) | Done |
| `cx uninstall` subcommand | Done |
| Homebrew formula (same-repo tap) | Done |
| Installer scripts (get-cx.sh, get-cx.ps1) | Done |
| cx-wasm crate (browser solver + extractor) | Done |
| Async shard prefetch (two-phase fetch/solve) | Done |
| conda-emscripten plugin (solver, extractor, magics, patches) | Done |
| cx-jupyterlite extension (conda command interception) | Done |
| Shared library loading for C extensions | Done |
| MEMFS compatibility patches (download, extract, subprocess) | Done |
| JupyterLite demo + GitHub Pages deployment | Done |
| ~~Include conda-tasks in default package set~~ | ~~Removed (conda-tasks archived)~~ |
| Include conda-workspaces in default package set ([#9]) | Blocked (needs conda-forge feedstock for conda-workspaces; conda-lockfiles already on conda-forge) |
| Submit cx-wasm packages to emscripten-forge ([#2]) | Not started |
| PyPI wheel support (repodata v3 / conda-pypi) ([#3]) | Not started |
| Homebrew-core submission ([#7]) | Not started (needs adoption first) |
| conda-forge feedstock for cx ([#6]) | Not started |
| conda-self pluggable updater backend ([#4]) | Not started |

[#2]: https://github.com/jezdez/conda-express/issues/2
[#3]: https://github.com/jezdez/conda-express/issues/3
[#4]: https://github.com/jezdez/conda-express/issues/4
[#5]: https://github.com/jezdez/conda-express/issues/5
[#6]: https://github.com/jezdez/conda-express/issues/6
[#7]: https://github.com/jezdez/conda-express/issues/7
[#8]: https://github.com/jezdez/conda-express/issues/8
[#9]: https://github.com/jezdez/conda-express/issues/9
[#10]: https://github.com/jezdez/conda-express/issues/10

### Phase 2: Production polish

| Task | Status |
|---|---|
| Documentation (Sphinx, Diataxis structure) | Done |
| Explore crate name transfer for `cx` on crates.io ([#10]) | Not started |

### Upstream work (nice to have -- independent of cx) ([#5](https://github.com/jezdez/conda-express/issues/5))

These improve conda's ecosystem health but are **not required** for cx:

1. **Make conda-libmamba-solver optional on conda-forge** -- would eliminate the need for cx's post-solve exclusion hack entirely, and make `conda self update` work without a custom backend
2. PR to conda/conda: make pycosat and menuinst optional (revive jaimergp's PR #14170 approach)
3. Publish conda-classic-solver to defaults and conda-forge (unblocks solver extraction)
4. Publish conda-rattler-solver, conda-spawn, conda-self to PyPI (conda-pypi 0.5.0 already on PyPI and conda-forge, pending defaults)
5. Publish conda itself to PyPI (reclaim the yanked `conda` package name)
6. **Publish conda-spawn to Anaconda defaults** -- conda-spawn is the only cx package not yet on defaults. All others (python, conda, conda-rattler-solver, conda-self) are already available; conda-pypi 0.5.0 is pending on defaults. Publishing conda-spawn to defaults would unblock a defaults-only cx configuration (no conda-forge dependency).

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
- **conda-pypi**: 0.5.0 on PyPI and conda-forge (pending defaults). Depends on `conda-index`. Provides `conda pypi install`, `conda pypi convert`, experimental wheel channels in repodata ([draft CEP PR #145](https://github.com/conda/ceps/pull/145)), editable/VCS package support, PEP-668 `EXTERNALLY-MANAGED` marker integration
- **conda-rattler-solver**: Not on PyPI yet, but pure Python + py-rattler (which IS on PyPI)
- **conda-self**: conda-forge only, essentially pure Python
