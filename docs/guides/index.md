# Guides

These guides explain where `cx` fits when you already know conda, already use
another project manager, or need to deploy conda in a constrained environment.

::::{grid} 1
:gutter: 3

:::{grid-item-card} Alternative to conda installers
:link: conda-installers
:link-type: doc

Try `cx` side by side with existing conda installations and understand what
changes.
:::

:::{grid-item-card} Offline and air-gapped installs
:link: offline-and-airgapped
:link-type: doc

Choose between `cxz`, external package bundles, and installer-script options.
:::

:::{grid-item-card} Pixi, uv, and Python package managers
:link: package-manager-fit
:link-type: doc

Decide when `cx` should own the conda prefix and when another tool should own
the project environment.
:::

:::{grid-item-card} Included conda plugins
:link: ../reference/included-plugins
:link-type: doc

See which plugin commands and workflows are available after bootstrap.
:::

::::

```{toctree}
:hidden:

conda-installers
offline-and-airgapped
package-manager-fit
```
