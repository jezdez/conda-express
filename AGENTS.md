# AGENTS.md - conda-express coding guidelines

## Project structure

- `conda-express` is the opinionated `cx` / `cxz` distribution repo.
  It builds release artifacts through `pronto`; the generic Rust runtime
  and builder implementation live in the separate `pronto` repository.

- Keep this repo focused on distribution defaults, GitHub Actions,
  Homebrew/Docker/release packaging, docs, and installer entry points.
  Do not reintroduce a local Cargo workspace for the runtime or builder.

- Browser, WebAssembly, Emscripten, and JupyterLite work belongs in the
  separate `conda-wasm` repository, not here.

## Lockfile maintenance

- After any change to `pixi.toml` that affects pixi metadata
  (dependencies, features, tasks, or workspace settings), always run
  `pixi lock` and commit the updated `pixi.lock` alongside the
  change. CI will fail if the lockfile is out of date.

## Dependencies

- Minimize the dependency graph. Prefer existing Pixi dependencies and
  GitHub Actions over adding new tooling.

- Use exact SHAs for GitHub Actions in CI workflows.

## Typing and linting

- Validate docs with `pixi run -e docs docs`.
