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
- Switched the conda-express composite action, reusable build workflow, CI
  canaries, and release binary jobs to build `cx` / `cxz` through Pronto.
- Removed the legacy in-repo Cargo workspace, `cx-build`, runtime source,
  checked-in `cx.lock`, and maturin/PyPI wrapper path now that Pronto owns the
  runtime and builder implementation.
- Removed legacy `payload`, `cx.lock`, and `cx.lock.hash` runtime/build
  surfaces from this repo; remaining references are historical notes.
- Rebuilt PyPI and crates.io distribution around Pronto-built release binaries
  instead of local runtime source builds.

## Remaining

### Rebuild on Pronto

- Keep `conda-express` configuration as the input to Pronto.
- Preserve `cx` as the online/bootstrap binary name.
- Preserve `cxz` as the embedded compressed-bundle binary name.

### Distribution Policy

- Decide the final default package and plugin set.
- Keep conda-spawn activation as distribution policy.
- Keep frozen base prefix behavior.
- Keep Homebrew, shell script, Docker, and GitHub Releases as distribution
  channels for `cx` / `cxz`.
- Keep PyPI and crates.io as distribution channels backed by Pronto artifacts.

### Documentation

- Keep this documentation user-facing and distribution-focused.
- Keep browser/WASM docs in `conda-wasm`.
- Keep builder docs in `pronto`.
- Follow the `conda-workspaces` / `conda-exec` documentation pattern across all
  three repositories.

## Tracking Issues

- Umbrella split: <https://github.com/jezdez/conda-express/issues/81>
- Rebuild this repo on Pronto: <https://github.com/jezdez/conda-express/issues/82>
- Finish Pronto builder migration: <https://github.com/jezdez/pronto/issues/1>
- Complete conda-wasm migration: <https://github.com/jezdez/conda-wasm/issues/1>
