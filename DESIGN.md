# Design

`conda-express` is the opinionated native conda distribution that ships the
`cx` binary and its compressed-bundle sibling, `cxz`.

The reusable builder and browser-specific projects now live elsewhere:

- `conda-ship`: generic build system for ready-to-run conda bootstrap binaries
- `conda-wasm`: browser, WebAssembly, Emscripten, and JupyterLite pipeline

This repository keeps the distribution policy for `cx`: the package set,
default prefix behavior, activation policy, release artifacts, and user-facing
documentation.

## Runtime Model

`cx` is a single native Rust binary built by conda-ship. It embeds:

- a runtime lock derived from the committed source-environment lock
- build-time metadata from the conda-express distribution defaults
- optionally, a compressed package bundle for `cxz`

On first use, `cx bootstrap` installs conda into the target prefix, writes
prefix metadata, configures the selected solver and plugins, and freezes the
base prefix. Later invocations delegate to the installed `conda` executable.

## Distribution Policy

`conda-express` intentionally keeps opinions that do not belong in `conda-ship`:

- binary names: `cx` and `cxz`
- default prefix: `~/.conda/express`
- default conda channel: `conda-forge`
- default package set: conda, conda-rattler-solver, conda-spawn, and selected
  conda ecosystem plugins
- frozen base prefix behavior
- `cx shell` as the conda-spawn based activation command
- user-facing install methods such as Homebrew, shell scripts, Docker, PyPI,
  and GitHub Releases

PyPI wheels consume release artifacts built with conda-ship; they do not
rebuild or vendor the runtime source in this repository.

## Build Flow

The distribution flow backed by conda-ship is:

1. `conda-express` supplies distribution defaults: the `runtime` source
   environment, exclusions, artifact names, release policy, and downstream
   packaging.
2. CI, release, and release-prep workflows invoke the pinned conda-ship action.
3. conda-ship owns the lock, bundle, build, inspect, and artifact metadata steps.
4. CI and release jobs build `cx` and `cxz` by invoking conda-ship rather than the
   legacy in-repo builder path.

This repository does not carry the generic runtime or builder source. Changes
to that implementation belong in conda-ship.

## Bootstrap Flow

At runtime, `cx bootstrap`:

1. Determines the target prefix.
2. Validates lockfile, offline, and bundle inputs.
3. Uses the stamped runtime lock.
4. Installs packages through rattler using locked package records.
5. Pre-populates the package cache from an external or embedded bundle when
   requested.
6. Writes runtime ownership metadata (for `cx`, `.cx.json`) and `.condarc`.
7. Writes the CEP 22 `conda-meta/frozen` marker for the base prefix.

After bootstrap, pass-through commands run the installed conda executable.

## Activation

`conda-express` keeps activation as distribution behavior, not builder
behavior. The default user workflow is:

```bash
cx shell myenv
```

That command delegates to conda-spawn and avoids shell-profile initialization.
Other conda commands pass through to the installed conda executable after
bootstrap.

## Repository Boundaries

This repo should not contain:

- WebAssembly crates
- Emscripten conda patches
- JupyterLite extensions or demo sites
- generic builder product naming
- Constructor-style `.sh`, `.pkg`, or `.msi` output generation

Those belong in `conda-wasm`, `conda-ship`, or downstream distribution channels.
