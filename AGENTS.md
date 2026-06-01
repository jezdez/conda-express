# AGENTS.md - conda-express coding guidelines

## Project structure

- `conda-express` is the opinionated `cx` / `cxz` distribution repo.
  It builds release artifacts through `conda-ship`; the generic Rust runtime
  and builder implementation live in the separate `conda-ship` repository.

- Keep this repo focused on distribution defaults, GitHub Actions,
  Homebrew/Docker/release packaging, docs, and installer entry points.
  Do not reintroduce a local Cargo workspace for the runtime or builder.

- Browser, WebAssembly, Emscripten, and JupyterLite work belongs in the
  separate `conda-wasm` repository, not here.

## Lockfile maintenance

- After any change to Pixi metadata in `pyproject.toml`
  (dependencies, features, tasks, or workspace settings), always run
  `pixi lock` and commit the updated `pixi.lock` alongside the
  change. CI will fail if the lockfile is out of date.

## Dependencies

- Minimize the dependency graph. Prefer existing Pixi dependencies and
  GitHub Actions over adding new tooling.

- Use exact SHAs for GitHub Actions in CI workflows. The conda-ship action is
  the exception: use its immutable release tag so the action and downloaded
  release assets share the same version.

## Typing and linting

- Validate docs with `pixi run -e docs docs`.

- Run `pixi run security` for the local security/audit sweep. It validates
  workflow YAML, runs actionlint, zizmor, ShellCheck, Bandit, Python compile
  checks, `git diff --check`, and `conda-ship inspect` when `cs` is available.
  The individual checks are also available as `security-workflows`,
  `security-installers`, `security-python`, and `security-conda-ship`.
