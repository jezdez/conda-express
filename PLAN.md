# conda-express (cx): Implementation Plan

> For architecture, current features, and file structure see [DESIGN.md](DESIGN.md).
>
> For ecosystem context, prior art, and rationale see the [Background & rationale](https://jezdez.github.io/conda-express/background/) docs page.

---

## Remaining work

### Phase 1: Ecosystem integration

| Task | Status | Issue |
|---|---|---|
| Include conda-workspaces in default package set | Blocked (needs conda-forge feedstock) | [#9] |
| Submit cx-wasm packages to emscripten-forge | Not started | [#2] |
| PyPI wheel support (repodata v3 / conda-pypi) | Not started | [#3] |
| conda-forge feedstock for cx | Not started | [#6] |
| conda-self pluggable updater backend | Not started | [#4] |
| Homebrew-core submission | Not started (needs adoption) | [#7] |

### Phase 2: Production polish

| Task | Status | Issue |
|---|---|---|
| Explore crate name transfer for `cx` on crates.io | Not started | [#10] |

### Upstream work (nice to have) ([#5])

These improve conda's ecosystem health but are **not required** for cx:

1. **Make conda-libmamba-solver optional on conda-forge** — would eliminate cx's post-solve exclusion hack and make `conda self update` work without a custom backend
2. PR to conda/conda: make pycosat and menuinst optional (revive jaimergp's [PR #14170](https://github.com/conda/conda/pull/14170) approach)
3. Publish conda-classic-solver to defaults and conda-forge (unblocks solver extraction)
4. Publish conda-rattler-solver, conda-spawn, conda-self to PyPI
5. Publish conda itself to PyPI (reclaim the yanked `conda` package name)
6. **Publish conda-spawn to Anaconda defaults** — last cx package not on defaults; would unblock a defaults-only cx configuration

[#2]: https://github.com/jezdez/conda-express/issues/2
[#3]: https://github.com/jezdez/conda-express/issues/3
[#4]: https://github.com/jezdez/conda-express/issues/4
[#5]: https://github.com/jezdez/conda-express/issues/5
[#6]: https://github.com/jezdez/conda-express/issues/6
[#7]: https://github.com/jezdez/conda-express/issues/7
[#9]: https://github.com/jezdez/conda-express/issues/9
[#10]: https://github.com/jezdez/conda-express/issues/10

---

## Completed work

### Phase 0: cx Rust prototype

All core functionality implemented and tested. See [DESIGN.md](DESIGN.md) for the full feature table and architecture.

### Phase 1: Ecosystem integration (completed items)

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
| Documentation (Sphinx, Diataxis structure) | Done |

---

## Design proposals

### conda-self pluggable updater backend ([#4])

cx intentionally does **not** implement its own update command. `conda self update` is the canonical way to update conda across all installation methods (miniconda, miniforge, cx, future pip-installed conda). This requires conda-self to support pluggable updater backends.

#### Why cx can't use conda-self's default backend

conda-self currently shells out to `conda install --prefix=sys.prefix conda`. This works in miniconda/miniforge, but in a cx-managed prefix it would **re-introduce conda-libmamba-solver** — conda on conda-forge hard-depends on it, and the solver has no way to exclude a required dependency. cx's post-solve filtering only works at the rattler level, outside conda's own solver.

Additionally, the base prefix is frozen via CEP 22. conda-self must override the frozen check when updating.

#### Proposed design

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

#### Backend implementations

**conda backend** (default, current behavior):

- `check_updates`: query conda channels for newer versions
- `install_updates`: `conda install --prefix=sys.prefix --override-frozen-env`
- Ships with conda-self itself

**cx backend** (for rattler-bootstrapped installs):

- Detects cx-managed prefix via `.cx.json` marker file
- `check_updates`: two-level check — conda-forge for newer packages, GitHub releases (or PyPI) for a newer cx binary
- `install_updates`: shells out to `cx _internal-update` (hidden subcommand) which uses rattler to re-solve with exclusion logic
- Ships as a small Python package installed into the cx prefix

**pypi backend** (for pip/uv-installed conda):

- `check_updates`: query PyPI for newer versions
- `install_updates`: ideally uses conda-pypi for consistency; falls back to pip/uv

#### User experience

```bash
conda self update         # works regardless of installation method
cx self update            # cx execs to conda, which runs conda-self
```

#### Detection logic

Each backend declares a `detect()` method:

- cx backend: checks for `.cx.json` in `sys.prefix`
- conda backend: checks for `conda-meta/conda-*.json` (default fallback)
- pypi backend: checks `importlib.metadata` for PyPI installer origin

#### Workaround until the plugin exists

```bash
cx bootstrap --force
```

---

### PyPI wheel support in cx-wasm ([#3])

Enable `conda install <package>` in the browser to install pure Python wheels from PyPI via the [conda-pypi](https://github.com/conda/conda-pypi) plugin and repodata v3 format.

#### conda-pypi-test channel

- URL: `https://github.com/conda-incubator/conda-pypi-test/releases/download`
- Indexes ~500K pure Python wheels (`-none-any.whl` only) from PyPI
- Not sharded — plain `repodata.json` (+ `.zst` compressed variant)
- Uses the [grayskull PyPI-to-conda mapping](https://raw.githubusercontent.com/regro/cf-graph-countyfair/master/mappings/pypi/grayskull_pypi_mapping.json) for name normalization

#### Repodata v3 format

Wheel entries use a `v3.whl` top-level key, defined by three draft CEPs:

- [CEP 111](https://github.com/conda/ceps/pull/111): Conditional dependencies, extras, and flags
- [CEP 145](https://github.com/conda/ceps/pull/145): Repodata wheel support
- [CEP 146](https://github.com/conda/ceps/pull/146): Backwards-compatible repodata update strategy

Implementation timeline:

| Date | Component | Change |
|------|-----------|--------|
| Feb 26, 2026 | `rattler_conda_types` v0.43.5 | Added `ExperimentalV3Packages` with `v3.whl` key ([PR #2093](https://github.com/conda/rattler/pull/2093)) |
| Mar 9, 2026 | py-rattler v0.23.0 | Exposes v3 support to Python ([PR #2007](https://github.com/conda/rattler/pull/2007)) |
| Mar 17, 2026 | conda-pypi [PR #273](https://github.com/conda/conda-pypi/pull/273) | Merged. v3 repodata with `extra_depends` and `when` conditionals |
| Pending | conda-pypi-test [PR #19](https://github.com/conda-incubator/conda-pypi-test/pull/19) | Blocked on conda-pypi release ([#277](https://github.com/conda/conda-pypi/issues/277)) |

#### conda-pypi plugin hooks

| Hook | Function | WASM-compatible |
|------|----------|:---:|
| `conda_package_extractors` | `.whl` extractor using pure-Python `installer` lib | Yes |
| `conda_subcommands` | `conda pypi` subcommand (subprocess) | No |
| `conda_post_commands` | Post-install hooks (subprocess) | No |

Only the extractor hook is needed for WASM.

#### cx-wasm gaps to bridge

1. **Repodata parsing**: `parse_repodata_text` in `sharded.rs` ignores `experimental_v3` — needs to chain `v3.whl` records
2. **Absolute download URLs**: `WhlPackageRecord` entries point to `files.pythonhosted.org`, not channel-relative paths
3. **Repodata size**: ~500K packages in one `repodata.json` — need `.zst` compression or subsetting
4. **Wheel extraction**: conda-pypi extractor and `installer` library must be available in JupyterLite
5. **Defensive patching**: conda-pypi's `post_command` hook calls subprocess — needs targeted disabling in WASM

---

### emscripten-forge publishing ([#2])

| Package | Type | Notes |
|---|---|---|
| `conda` | noarch | Patched 26.1.1 with emscripten patches |
| `conda-emscripten` | noarch | Solver + extractor + vpkgs + magic plugin |
| `cx-wasm-kernel` | noarch | WASM files + Python bridge |
| `frozendict` | noarch | 2.4.6 pure Python (may already exist on emscripten-forge) |

Once published, `lite/environment.yml` can reference these as dependencies, eliminating `--with-local` builds and simplifying GitHub Pages CI.

---

## Open risks

- **Requires network on first run**: No offline-first option without pre-populating a package cache in the binary (adds size).
- **conda-self hook design**: Needs buy-in from conda-self maintainers ([#4]).
- **conda-index dependency**: conda-pypi depends on conda-index — needs PyPI availability verification.
- **menuinst on Windows**: `initialize.py` imports `menuinst.knownfolders`/`menuinst.winshortcut` behind `if on_win:` — needs a try/except guard (upstream).
