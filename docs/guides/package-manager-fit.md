# Pixi, uv, and Python Package Managers

`cx` is a conda distribution bootstrapper. It installs a conda base prefix and
then delegates to the conda CLI. It is not trying to become every project's
package manager.

The practical rule is: one tool should own an environment. Use `cx` when the
environment is meant to be managed as a conda environment. Use a project
manager when that project manager owns the environment and lockfile.

## For conda users

Use `cx` when you want the conda CLI, conda channels, conda packages, and conda
plugins with a locked bootstrap path:

```bash
cx create -n data python=3.12 numpy pandas
cx spawn data
```

The main differences from a traditional base installation are the frozen
`~/.conda/express` base prefix and the `cx spawn` activation model.

## For Pixi users

[Pixi](https://pixi.prefix.dev/latest/) is a project-oriented package and
workflow manager built on the conda ecosystem. Keep using Pixi for projects
that are already centered on `pixi.toml`, `pixi.lock`, and `pixi run`.

`cx` is useful next to Pixi when you want a conda CLI distribution for:

- users who expect `conda create`, `conda install`, and `conda env`
- conda plugin workflows, including `conda-workspaces`
- a small bootstrap artifact for machines that should have conda available
  before any project is selected

Do not manually edit a Pixi-managed environment with `cx install`. Let Pixi own
Pixi environments.

## For uv users

[uv](https://docs.astral.sh/uv/) is a Python package and project manager. It is
a strong fit for Python-only projects, Python tools, virtual environments, and
PyPI-centered lockfiles.

Use `cx` instead when the environment depends on conda packages or conda
channels, for example native libraries, compilers, CUDA stacks, geospatial
packages, or mixed-language scientific environments.

Using both is fine when the boundary is clear:

- `uv` owns Python project environments such as `.venv`.
- `cx` owns conda environments created under the `~/.conda/express` prefix.

## For pip, Poetry, Hatch, PDM, and pipx users

Keep using Python package managers for Python package development and
PyPI-first workflows. `cx` is helpful when you need conda's binary package
ecosystem or want to install Python tooling through isolated conda
environments:

```bash
cx global install ruff
cx global install nox
```

Avoid mixing managers casually inside one environment. If a conda environment
needs a small number of PyPI packages, use the conda plugin workflow available
in the `cx` base. If a Python project is already managed by uv, Poetry, Hatch,
or PDM, let that tool keep control of its project environment.
