# Configuration

## Build-time configuration

Package specs, channels, and exclusions are defined in the `[tool.cx]` section
of `pixi.toml`. These are read both by `build.rs` (at compile time) and
embedded into the binary.

```toml
[tool.cx]
channels = ["conda-forge"]
packages = [
    "python >=3.12",
    "conda >=25.1",
    "conda-rattler-solver",
    "conda-spawn",
    "conda-pypi",
    "conda-self",
]
exclude = ["conda-libmamba-solver"]
```

### `channels`

List of conda channels to solve against. Defaults to `conda-forge`.

### `packages`

List of [MatchSpec](https://conda.io/projects/conda/en/latest/user-guide/concepts/pkg-specs.html)
strings defining the packages to install in the base prefix.

### `exclude`

List of package names to exclude from the installation. cx will also remove
any dependencies that are *exclusively* required by the excluded packages.

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
  "version": "0.1.0",
  "channels": ["conda-forge"],
  "packages": ["python >=3.12", "conda >=25.1", "conda-rattler-solver"],
  "excludes": ["conda-libmamba-solver"]
}
```

This is used by `cx status` and will be used by the future conda-self update
backend to detect cx-managed prefixes.

### `conda-meta/frozen`

A [CEP 22](https://conda.org/learn/ceps/cep-0022/) frozen marker file that
prevents accidental modification of the base prefix:

```json
{
  "message": "This base environment is managed by cx (conda-express).\nCreate a new environment instead: conda create -n myenv\nTo re-bootstrap: cx bootstrap --force\nTo override: pass --override-frozen-env"
}
```

## Customizing the build

To change what cx installs, edit the `[tool.cx]` section in `pixi.toml` and
rebuild:

```bash
pixi run build
```

The first build after a config change triggers a compile-time re-solve. A
content hash ensures the solve is skipped when the config hasn't changed.

## Default prefix

The default installation prefix is `~/.cx`. Override it per-command with the
`--prefix` flag:

```bash
cx bootstrap --prefix /opt/cx
cx status --prefix /opt/cx
```
