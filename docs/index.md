# conda-express

`conda-express` publishes `cx`, a native bootstrapper for
[conda](https://github.com/conda/conda). It installs a working conda
environment from a built-in runtime lock, then passes commands through to the
installed `conda` executable.

`cxz` is the offline variant. It carries the locked package archives in the
binary so it can bootstrap without network access.

`conda-express` is Jannis Leidel's distribution project for the `cx` and `cxz`
binaries. Custom bootstrap binaries are built with
{external+conda-ship:doc}`conda-ship <index>`.

## Install `cx`

Homebrew is the recommended install path on macOS and Linux:

```bash
brew tap jezdez/conda-express https://github.com/jezdez/conda-express
brew install jezdez/conda-express/cx
cx bootstrap
```

See {doc}`quickstart` for the first environment workflow. See
{doc}`guides/index` for using `cx` alongside Anaconda Distribution, Miniconda,
and Miniforge; offline installs; release artifact verification; included
plugins; and how `cx` fits next to Pixi, uv, and Python package managers.

## Choose A Path

::::{grid} 1 1 2 3
:gutter: 3

:::{grid-item-card} Tutorial
:link: quickstart
:link-type: doc

Install `cx`, bootstrap the prefix, create an environment, and activate it.
:::

:::{grid-item-card} Guides
:link: guides/index
:link-type: doc

Compare `cx` with familiar installers, use offline bootstrap, review included
plugins, verify release artifacts, and place it next to project package
managers.
:::

:::{grid-item-card} Project Scope
:link: scope
:link-type: doc

See what conda-express owns and when to use conda-ship instead.
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
:caption: Guides

guides/index
```

```{toctree}
:hidden:
:caption: Reference

reference/cli
reference/installer
reference/included-plugins
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
