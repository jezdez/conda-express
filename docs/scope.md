# Project scope

`conda-express` is the distribution repo for `cx` and `cxz`.

It is not an official conda distribution and it is not the generic builder.
The generic builder responsibility lives in
{external+conda-ship:doc}`conda-ship <index>`.

| Project | Role |
|---|---|
| `conda-express` | Opinionated native conda distribution that publishes `cx` and `cxz` |
| {external+conda-ship:doc}`conda-ship <index>` | Generic builder/runtime for ready-to-run conda bootstrap binaries |

## What conda-express owns

This repository keeps the pieces that make `cx` this specific conda
distribution:

- binary names: `cx` and `cxz`
- default package set
- default channel and package exclusions
- default prefix behavior
- conda-spawn activation policy
- frozen base-prefix policy
- install scripts and user-facing installation docs
- Docker, Homebrew, PyPI, and GitHub Release packaging
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

For concrete builder workflows, start with
{external+conda-ship:doc}`conda-ship's GitHub Actions guide <how-to/build-in-github-actions>`
or
{external+conda-ship:doc}`local build guide <how-to/build-locally>`.

## What this means for users

If you want this managed conda distribution with a small bootstrap artifact,
install `cx` or `cxz` from this project.

If you want to build your own `cx`-like binary, use
{external+conda-ship:doc}`conda-ship <index>` directly. The result should have its own
name and release channel unless it is a conda-express release
artifact.

## What this means for contributors

Changes to the package choices, install methods, documentation, release
packaging, or `cx` distribution policy belong here.

Changes to generic runtime behavior, bundle layouts, lockfile derivation,
artifact metadata, or builder interfaces belong in conda-ship.

Changes to conda-express release policy, artifact verification wording,
Homebrew/PyPI/Docker packaging, or the included plugin set belong here.

## Developer Workflows

### Recording demos

The checked-in GIF demos are generated from `demos/*.tape` with
[VHS](https://github.com/charmbracelet/vhs). Record them against current
conda-express binaries, not an older `cx` installed on `PATH`:

```bash
CX_BIN=/path/to/cx CXZ_BIN=/path/to/cxz pixi run demos
```

To refresh one demo:

```bash
CX_BIN=/path/to/cx pixi run demos quickstart
```

The tapes run in a temporary `HOME` and use the current default install path,
`~/.conda/express`.
