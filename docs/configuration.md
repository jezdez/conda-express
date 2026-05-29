# Configuration

## Build-time configuration

The conda-express release and release-prep workflows pass this distribution
package set to conda-pronto when building `cx` and `cxz` artifacts:

```toml
channels = ["conda-forge"]
packages = [
    "python >=3.12",
    "conda >=25.1",
    "conda-rattler-solver",
    "conda-spawn >=0.1.0",
    "conda-completion >=0.2.0",
    "conda-pypi",
    "conda-self",
    "conda-global",
    "conda-workspaces >=0.4.0",
]
exclude = ["conda-libmamba-solver"]
```

### `channels`

List of conda channels to solve against. Defaults to `conda-forge`.

### `packages`

List of [MatchSpec](https://conda.io/projects/conda/en/latest/user-guide/concepts/pkg-specs.html)
strings defining the packages to install in the base prefix.

### `exclude`

List of package names to exclude from the installation. conda-pronto also removes any
dependencies that are *exclusively* required by the excluded packages.

### Where this configuration lives

There is no public `conda-express` build manifest and no `[tool.cx]` section in
`pyproject.toml`. The official distribution defaults are maintained in the
release workflows and mirrored in the docs.

`pyproject.toml` is used for Python packaging metadata and Pixi maintenance
tasks for this repository. Its `cx-env` environment is a development aid that
tracks the intended conda-express package set; it is not a runtime
configuration file consumed by `cx`.

Custom package sets and new binary distributions should be built with conda-pronto
directly.

## Runtime configuration

### `.condarc`

cx writes a `.condarc` into the prefix with these settings:

```yaml
solver: rattler
auto_activate_base: false
notify_outdated_conda: false
show_channel_urls: true
default_channels:
  - conda-forge
```

### `.cx.json`

cx writes metadata about the installation into `.cx.json` at the prefix root:

```json
{
  "version": "0.6.0",
  "channels": ["conda-forge"],
  "packages": ["python >=3.12", "conda >=25.1", "conda-rattler-solver"],
  "excludes": ["conda-libmamba-solver"]
}
```

This is used by `cx status` and by distribution tooling to detect cx-managed
prefixes.

### `conda-meta/frozen`

A [CEP 22](https://conda.org/learn/ceps/cep-0022/) frozen marker file prevents
accidental modification of the base prefix:

```json
{
  "message": "This base environment is managed by cx.\nCreate a new environment instead: conda create -n myenv\nTo re-bootstrap: cx bootstrap --force\nTo override: pass --override-frozen-env"
}
```

## Customizing the build

To change the official conda-express package set, update the distribution
defaults in the release and release-prep workflows. For custom package sets or
new distributions, use conda-pronto directly instead of treating this repository as a
generic builder.

### Runtime environment variables

These environment variables control bootstrap behavior at runtime. They are
particularly useful in native installer post-install scripts and CI pipelines.

| Variable | Effect |
|---|---|
| `CX_BUNDLE` | Directory of `.conda` / `.tar.bz2` archives to pre-populate the package cache from (equivalent to `--bundle`) |
| `CX_OFFLINE` | Disable network access during bootstrap when set to any truthy value (equivalent to `--offline`). Values `0` and `false` are treated as unset |

```bash
# Native installer post-install script example
CX_BUNDLE=/Library/Application\ Support/cx/packages CX_OFFLINE=1 cx bootstrap
```

## Default prefix

The default installation prefix is `~/.cx`. Override it per-command with the
`--prefix` flag:

```bash
cx bootstrap --prefix /opt/cx
cx status --prefix /opt/cx
```
