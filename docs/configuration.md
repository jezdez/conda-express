# Configuration

## Build-time configuration

The conda-express GitHub Action passes this distribution package set to Pronto
when building official `cx` and `cxz` artifacts:

```toml
channels = ["conda-forge"]
packages = [
    "python >=3.12",
    "conda >=25.1",
    "conda-rattler-solver",
    "conda-spawn >=0.1.0",
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

List of package names to exclude from the installation. Pronto also removes any
dependencies that are *exclusively* required by the excluded packages.

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

To change what cx installs in CI, pass package, channel, and exclude inputs to
the conda-express action or reusable workflow:

```yaml
- uses: jezdez/conda-express@main
  with:
    packages: "python >=3.12, conda >=25.1, conda-rattler-solver, conda-spawn, numpy"
    channels: "conda-forge"
    exclude: "conda-libmamba-solver"
```

Pronto resolves the package set, writes the runtime lock, and embeds that lock
into the staged binary.

### Action inputs

| Input | Format |
|---|---|
| `packages` | Comma-separated [MatchSpec](https://conda.io/projects/conda/en/latest/user-guide/concepts/pkg-specs.html) strings |
| `channels` | Comma-separated channel names |
| `exclude` | Comma-separated package names |
| `embed-bundle` | `"true"` to build the `cxz` embedded-bundle variant |

Empty values use the conda-express defaults.

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
