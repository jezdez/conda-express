# AGENTS.md — conda-express coding guidelines

## Project structure

- `conda-express` is a Rust binary (with a Python wheel via maturin)
  that bootstraps conda from scratch using rattler.

- The Cargo workspace has two members: the root crate
  (`conda-express` / `cx` binary) and `crates/cx-wasm` (WebAssembly
  build for JupyterLite).

- Tests for Rust live in `tests/` and inline `#[cfg(test)]` modules.
  Integration tests for the Python wheel go in `python/tests/`.

## Lockfile maintenance

- After any change to `Cargo.toml` (root or workspace members), run
  `cargo generate-lockfile` (or `cargo build`) and commit the updated
  `Cargo.lock`. CI builds use `--locked` and will fail if the lockfile
  is out of date.

- After any change to `pixi.toml` that affects pixi metadata
  (dependencies, features, tasks, or workspace settings), always run
  `pixi lock` and commit the updated `pixi.lock` alongside the
  change. CI will fail if the lockfile is out of date.

## Dependencies

- Minimize the dependency graph. Prefer already-required crates over
  adding new ones.

- Pin minimum versions in `Cargo.toml` (e.g., `"rattler = 1.4"`).
  Use exact SHAs for GitHub Actions in CI workflows.

## Typing and linting

- Rust: `cargo clippy` with `-D warnings`. Format with `cargo fmt`.

- Use `from __future__ import annotations` in any Python modules.
