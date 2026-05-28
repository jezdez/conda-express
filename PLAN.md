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

## Remaining

### Rebuild on Pronto

- Move remaining generic builder behavior out of this repository.
- Replace `cx-build` usage with Pronto once Pronto's public interface is ready.
- Keep `conda-express` configuration as the input to Pronto.
- Preserve `cx` as the online/bootstrap binary name.
- Preserve `cxz` as the embedded compressed-bundle binary name.

### Distribution Policy

- Decide the final default package and plugin set.
- Keep conda-spawn activation as distribution policy.
- Keep frozen base prefix behavior.
- Keep Homebrew, shell script, Docker, PyPI, crates.io, and GitHub Releases as
  distribution channels for `cx` / `cxz`.

### Terminology

- Use "bundle" for compressed package archives.
- Stop adding new `payload` terminology.
- Rename existing payload internals once Pronto owns the builder flow.

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
