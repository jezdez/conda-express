# Project scope

`conda-express` is the distribution repo for `cx` and `cxz`.

It is not the generic builder, and it is not the browser/WASM conda stack.
Those responsibilities now live in separate repositories:

| Project | Role |
|---|---|
| `conda-express` | Opinionated native conda distribution that publishes `cx` and `cxz` |
| {external+conda-pronto:doc}`conda-pronto <index>` | Generic builder/runtime for ready-to-run conda bootstrap binaries |
| {external+conda-wasm:doc}`conda-wasm <index>` | Browser, WebAssembly, Emscripten, and JupyterLite conda tooling |

## What conda-express owns

This repository keeps the pieces that make `cx` a specific conda distribution:

- binary names: `cx` and `cxz`
- default package set
- default channel and package exclusions
- default prefix behavior
- conda-spawn activation policy
- frozen base-prefix policy
- install scripts and user-facing installation docs
- Docker, Homebrew, PyPI, crates.io, and GitHub Release packaging
- release and release-prep workflows for official `cx` / `cxz` artifacts

The generic build implementation is deliberately outside this repository.
`conda-express` consumes artifacts built with conda-pronto; it does not carry the runtime
or builder source.

## What belongs in conda-pronto

Use conda-pronto when you want to build a different bootstrap binary:

- a different package set
- a different binary name
- a different default channel set
- a product-specific distribution that is not `cx`
- an embedded-bundle variant for another distribution

conda-pronto owns lock generation, bundle creation, binary building, artifact
metadata, and the public builder interface. See
{external+conda-pronto:doc}`conda-pronto's project-boundary notes <explanation/project-boundaries>`
for the builder/runtime side of the split. The conda-express release workflows
call conda-pronto with the `cx` distribution defaults.

## What belongs in conda-wasm

Browser and WebAssembly work lives in
{external+conda-wasm:doc}`conda-wasm <index>`:

- WASM crates
- Emscripten conda patches
- JupyterLite integration
- browser package extraction and solving behavior
- emscripten-forge packaging

Those pieces are intentionally not part of the native `cx` distribution repo.

## What this means for users

If you want a fast conda distribution, install `cx` or `cxz` from this project.

If you want to build your own `cx`-like binary, use
{external+conda-pronto:doc}`conda-pronto <index>` directly. The result should have its own
name and release channel unless it is an official conda-express release
artifact.

If you want conda in the browser, use
{external+conda-wasm:doc}`conda-wasm <index>`.

## What this means for contributors

Changes to the package choices, install methods, documentation, release
packaging, or `cx` distribution policy belong here.

Changes to generic runtime behavior, bundle layouts, lockfile derivation,
artifact metadata, or builder interfaces belong in conda-pronto.

Changes to JupyterLite, Emscripten, WebAssembly, or browser-specific package
handling belong in conda-wasm.
