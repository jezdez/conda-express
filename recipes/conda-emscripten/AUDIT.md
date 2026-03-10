# conda Import Chain Audit for Browser/WASM

Audited against conda 26.1.1 (git: `5438ecce7`).

## Import-time Side Effects

### `conda/__init__.py`

- `os.environ["CONDA_ROOT"] = sys.prefix` — env mutation
- `abspath(dirname(__file__))` — path resolution (works on MEMFS)
- `JSONEncoder.default = _default` — monkey-patch (harmless)

### `conda/base/context.py`

Most critical module — the global configuration singleton.

- **Line 33**: `from ..common._os.osx import mac_ver` — loads
  `osx.py` which does `from subprocess import check_output` at top
  level. This import **succeeds** on emscripten (subprocess module
  exists). `mac_ver()` is only called when `platform_name == "Darwin"`,
  which won't happen on emscripten.
- **Line 32**: `from ..common._os.linux import linux_get_libc_version` —
  safe; guarded by `on_linux` check.
- **Lines 96–104**: `os.getcwd()` / `os.chdir(sys.prefix)` — works on
  emscripten MEMFS.
- **Line 131**: `expanduser("~/.condarc")` — works if `HOME` env var
  is set.
- **Line 135–145**: `user_data_dir()` — lazy import of `platformdirs`.
  Only called on Windows for envs_dirs.

### `conda/plugins/manager.py`

Imports ALL plugin submodules at initialization:
- `conda.plugins.virtual_packages` → imports `cuda.py`, `archspec.py`, etc.
- `conda.plugins.package_extractors.conda` → uses `conda_package_handling`
- `conda.plugins.solvers` → references solvers

### `conda/plugins/virtual_packages/cuda.py`

- **Line 10**: `import multiprocessing` — succeeds on emscripten
- **Line 41**: `multiprocessing.get_context("spawn")` then
  `context.Process(...)` — **WILL CRASH** on emscripten. Must be patched.

### `conda/gateways/repodata/jlap/fetch.py`

- **Line 17**: `import zstandard` — **WILL CRASH** if zstandard is not
  installed. Not imported at conda startup but could be imported if
  JLAP repodata fetching is triggered.

## Skippable Dependencies — Import Locations

| Dep                      | Import location                                   | Style       | Import-time? |
|--------------------------|---------------------------------------------------|-------------|--------------|
| `pycosat`                | `conda/common/_logic.py:168`                      | Inside method | No |
| `menuinst`               | `conda/core/initialize.py:89` (Windows only)      | Conditional | Only on Windows |
| `truststore`             | `conda/gateways/connection/session.py:263`         | Inside method | No |
| `distro`                 | `conda/base/context.py:1200`                       | Inside property | No |
| `archspec`               | `conda/core/index.py:651`                          | Inside function | No |
| `conda_package_handling` | `conda/core/package_cache_data.py:70,372`          | Inside methods | No |
|                          | `conda/gateways/disk/read.py:110`                  | Inside function | No |
|                          | `conda/plugins/package_extractors/conda.py:34`     | Inside function | No |
|                          | `conda/base/context.py:1040`                       | Inside property | No |
| `zstandard`              | `conda/gateways/repodata/jlap/fetch.py:17`          | **Top-level** | **Yes** (if JLAP is used) |

## OS/Subprocess Usage in Non-test Code

| File | API | Context |
|------|-----|---------|
| `conda/gateways/subprocess.py:12` | `from subprocess import PIPE, Popen` | Top-level, but functions only called at runtime |
| `conda/gateways/subprocess.py:54,108` | `Popen(...)` | Inside `any_subprocess`, `subprocess_call` |
| `conda/common/_os/osx.py:4` | `from subprocess import check_output` | Top-level (import OK, usage guarded by platform) |
| `conda/gateways/disk/create.py:113` | `os.execv()` | Inside function |
| `conda/cli/conda_argparse.py:289` | `os.execvpe()` | Inside function |
| `conda/plugins/virtual_packages/cuda.py:41` | `multiprocessing.Process` | Inside `cuda_version()` |
| `conda/common/signals.py:5` | `import signal` | Top-level (limited on emscripten) |
| `conda/common/io.py:7` | `import signal` | Top-level (limited on emscripten) |

## Patches Required for Phase 1 (conda info / conda list)

1. **001-stub-subprocess**: Stub `any_subprocess()` and `subprocess_call()` to raise a
   clear error. Safety net — info/list don't call these.

2. **002-skip-unneeded-deps**: Guard the 7 skippable deps:
   - `cuda.py`: Return NULL immediately (skip multiprocessing spawn)
   - `archspec`: Guard `import archspec.cpu` with try/except
   - `zstandard`: Guard top-level import
   - Others are already lazy-imported, but add try/except guards for safety

3. **003-platformdirs-memfs**: Override `user_data_dir` for MEMFS paths on emscripten.

4. **004-linking-memfs**: Placeholder for Phase 2 (no symlinks/hardlinks).
