# Project scope

`conda-express` is the distribution repo for `cx` and `cxz`.

It is not the generic builder. That responsibility lives in
{external+conda-ship:doc}`conda-ship <index>`.

| Project | Role |
|---|---|
| `conda-express` | Opinionated native conda distribution that publishes `cx` and `cxz` |
| {external+conda-ship:doc}`conda-ship <index>` | Generic builder/runtime for ready-to-run conda bootstrap binaries |

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
- release and release-prep workflows for this repository's `cx` / `cxz` artifacts

The generic build implementation is deliberately outside this repository.
`conda-express` consumes artifacts built with conda-ship; it does not carry the runtime
or builder source.

## When to use conda-ship

Use conda-ship when you want to build a different bootstrap binary:

- a different package set
- a different binary name
- a different default channel set
- a product-specific distribution that is not `cx`
- an embedded-bundle variant for another distribution

conda-ship owns lock generation, bundle creation, binary building, artifact
metadata, and the public builder interface. See
{external+conda-ship:doc}`conda-ship's project-boundary notes <explanation/project-boundaries>`
for the builder/runtime side of the split. The conda-express release workflows
call conda-ship with the `cx` distribution defaults.

## What this means for users

If you want a fast conda distribution, install `cx` or `cxz` from this project.

If you want to build your own `cx`-like binary, use
{external+conda-ship:doc}`conda-ship <index>` directly. The result should have its own
name and release channel unless it is a conda-express release
artifact.

## What this means for contributors

Changes to the package choices, install methods, documentation, release
packaging, or `cx` distribution policy belong here.

Changes to generic runtime behavior, bundle layouts, lockfile derivation,
artifact metadata, or builder interfaces belong in conda-ship.
