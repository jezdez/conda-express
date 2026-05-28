# Plan

`conda-express` is being narrowed to the `cx` / `cxz` distribution. The split
creates two companion repositories:

- `pronto`: the generic builder for ready-to-run conda bootstrap binaries
- `conda-wasm`: the browser, WASM, Emscripten, and JupyterLite stack

## Done

- Created `jezdez/pronto`.
- Created `jezdez/conda-wasm`.
- Moved the browser/JupyterLite code path to `conda-wasm`.
- Removed JupyterLite demo publishing from this repo's docs workflow.
- Added migration notes that point browser/WASM users to `conda-wasm`.
- Removed WebAssembly, Emscripten, JupyterLite, and demo source files from this
  repo.
- Kept `conda-express` focused on native `cx` and `cxz` binaries.
- Switched CI canaries, release prep, and release binary jobs to build `cx` /
  `cxz` through Pronto.
- Removed the legacy in-repo Cargo workspace, `cx-build`, runtime source,
  checked-in `cx.lock`, and maturin/PyPI wrapper path now that Pronto owns the
  runtime and builder implementation.
- Removed legacy `payload`, `cx.lock`, and `cx.lock.hash` runtime/build
  surfaces from this repo; remaining references are historical notes.
- Rebuilt PyPI and crates.io distribution around Pronto-built release binaries
  instead of local runtime source builds.
- Removed the `conda-express` composite GitHub Action; this repo's
  `.github/workflows/build.yml` is release preparation for official `cx` /
  `cxz` binaries, not a downstream builder interface.
- Moved Pixi metadata and Python package metadata into `pyproject.toml`.

## Remaining

### Verification

- Run the release-prep workflow on GitHub runners once to verify the direct
  Pronto action path for all `cx` and `cxz` platforms.
- Exercise the full release workflow before the next public release.

### Distribution Policy

- Keep the default package set synchronized across release workflows,
  `pyproject.toml`'s `cx-env`, and docs.
- Add `conda-exec` to the default package set once the intended new release is
  available on conda-forge.
- Keep Homebrew, shell script, Docker, GitHub Releases, PyPI, and crates.io as
  distribution channels backed by Pronto artifacts.

## Tracking Issues

- Umbrella split: <https://github.com/jezdez/conda-express/issues/81>
- Add `conda-exec` once released: <https://github.com/jezdez/conda-express/issues/85>
- Channel presets follow-up: <https://github.com/jezdez/pronto/issues/2>
- Complete conda-wasm migration: <https://github.com/jezdez/conda-wasm/issues/1>
