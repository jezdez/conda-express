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

:::{grid-item-card} Upgrade from early cx
:link: upgrade-from-early-cx
:link-type: doc

Move from early `~/.cx` installs to the current `~/.conda/express` managed
prefix.
:::

:::{grid-item-card} Offline and air-gapped installs
:link: offline-and-airgapped
:link-type: doc

Choose between `cxz`, bundle directories, and installer-script options.
:::

:::{grid-item-card} Use cx in GitHub Actions
:link: use-cx-in-github-actions
:link-type: doc

Add `cx` to CI, choose whether to bootstrap, and opt into artifact
attestation checks.
:::

:::{grid-item-card} Verify release artifacts
:link: verify-release-artifacts
:link-type: doc

Check downloaded binaries, checksums, attestations, runtime locks, and
metadata before use or transfer.
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
upgrade-from-early-cx
offline-and-airgapped
use-cx-in-github-actions
verify-release-artifacts
package-manager-fit
```
