# conda-express

`conda-express` publishes `cx`, a small native bootstrapper for
[conda](https://github.com/conda/conda). It installs a working conda
environment from a built-in runtime lock, then passes commands through to the
installed `conda` executable.

`cxz` is the offline variant. It carries the locked package archives in the
binary so it can bootstrap without network access.

`conda-express` is the distribution project for the official `cx` and `cxz`
binaries. Custom bootstrap binaries are built with
{external+conda-pronto:doc}`conda-pronto <index>`.

## Install `cx`

Homebrew is the shortest path on macOS and Linux:

```bash
brew tap jezdez/conda-express https://github.com/jezdez/conda-express
brew install jezdez/conda-express/cx
cx bootstrap
```

See {doc}`quickstart` for shell scripts, GitHub Releases, Docker, PyPI,
crates.io, `cxz`, and the first environment workflow.

## Choose A Path

::::{grid} 1 1 2 4
:gutter: 3

:::{grid-item-card} Tutorial
:link: quickstart
:link-type: doc

Install `cx`, bootstrap the prefix, create an environment, and activate it.
:::

:::{grid-item-card} Project Scope
:link: scope
:link-type: doc

See what belongs in conda-express and what belongs in conda-pronto.
:::

:::{grid-item-card} Reference
:link: reference/cli
:link-type: doc

Look up `cx bootstrap`, `status`, `shell`, `uninstall`, and pass-through
behavior.
:::

:::{grid-item-card} Explanation
:link: features
:link-type: doc

Understand runtime locks, package exclusions, activation, `cxz`, and release
artifacts.
:::

::::

```{toctree}
:hidden:
:caption: Tutorials

quickstart
```

```{toctree}
:hidden:
:caption: Reference

reference/cli
reference/installer
configuration
```

```{toctree}
:hidden:
:caption: Explanation

features
scope
design
background
```

```{toctree}
:hidden:
:caption: Project

plan
changelog
```
