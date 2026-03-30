# Build a custom cx binary

This guide shows how to build a cx binary with your own set of conda packages
baked in. This is useful when you want to distribute a bootstrapper that
includes domain-specific packages (e.g. numpy, pandas) out of the box.

## Using the GitHub Action

The simplest approach. Add a workflow to your repo that calls the cx
composite action:

```yaml
name: Build custom cx
on: [push]

jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}

    steps:
      - uses: jezdez/conda-express@main
        id: cx
        with:
          packages: "python >=3.12, conda >=25.1, conda-rattler-solver, conda-spawn, numpy, pandas"

      - uses: actions/upload-artifact@v4
        with:
          name: ${{ steps.cx.outputs.asset-name }}
          path: ${{ steps.cx.outputs.binary-path }}
```

The action builds cx for the runner's platform and outputs the path to the
binary. Use a matrix to build for multiple platforms.

See the [GitHub Action reference](../reference/github-action.md) for all
inputs and outputs.

## Using the reusable workflow

If you want all 5 platforms built without managing a matrix yourself:

```yaml
name: Build custom cx
on: [push]

jobs:
  build-cx:
    uses: jezdez/conda-express/.github/workflows/build.yml@main
    with:
      packages: "python >=3.12, conda >=25.1, conda-rattler-solver, conda-spawn, numpy, pandas"
      channels: "conda-forge"
      exclude: "conda-libmamba-solver"
```

Binary artifacts for all platforms are uploaded automatically.

## Building locally

Set environment variables to override the default package list, then build:

```bash
git clone https://github.com/jezdez/conda-express.git
cd conda-express

CX_PACKAGES="python >=3.12, conda >=25.1, conda-rattler-solver, conda-spawn, numpy" \
  pixi run build
```

The binary is at `target/release/cx`.

### Available overrides

| Variable | Effect |
|---|---|
| `CX_PACKAGES` | Replace the default package list |
| `CX_CHANNELS` | Replace the default channels |
| `CX_EXCLUDE` | Replace the default exclusions |

Empty values are ignored. See the {ref}`configuration reference <env-var-overrides>`
for details on how overrides interact with the lockfile cache.

## Choosing packages

When specifying packages, keep in mind:

- Always include the core set: `python`, `conda`, `conda-rattler-solver`,
  and `conda-spawn` (cx depends on these at runtime)
- Use [MatchSpec](https://conda.io/projects/conda/en/latest/user-guide/concepts/pkg-specs.html)
  syntax for version constraints (e.g. `numpy >=1.26`)
- The build performs a full dependency solve at compile time, so all
  transitive dependencies are resolved and locked
- The resulting binary is self-contained with an embedded lockfile

## Bundling a payload for offline bootstrap

A custom cx binary can be paired with a pre-downloaded set of package archives
for fully offline, air-gapped installation. This is useful for native installers
(macOS PKG, Windows MSI) and restricted-network deployments.

### Preparing the payload

Run an online bootstrap once to populate the rattler package cache, then
copy the archives into a payload directory:

```bash
# Bootstrap online to populate the cache
cx bootstrap --prefix /tmp/seed

# Collect the cached archives
mkdir payload
cp ~/Library/Caches/rattler/cache/pkgs/*.conda payload/
cp ~/Library/Caches/rattler/cache/pkgs/*.tar.bz2 payload/ 2>/dev/null || true
```

:::{note}
The rattler cache location varies by platform: `~/Library/Caches/rattler`
on macOS, `~/.cache/rattler` on Linux, and `%LOCALAPPDATA%\rattler` on
Windows.
:::

### Using the payload

Bundle the cx binary and the payload directory in your installer. In the
post-install script, bootstrap from the payload:

```bash
cx bootstrap --payload /path/to/payload --offline
```

Or using environment variables (convenient for installer scripts):

```bash
CX_PAYLOAD=/path/to/payload CX_OFFLINE=1 cx bootstrap
```

### Payload with network fallback

If you want the payload to cover most packages but allow network fallback for
any missing ones, omit `--offline`:

```bash
cx bootstrap --payload /path/to/payload
```

This pre-populates the cache from the payload, then downloads anything not
found locally.

## Building cxz (self-contained binary)

`cxz` is a variant of `cx` that embeds all package archives directly into the
binary. The result is a single ~60 MB file that bootstraps conda with zero
network access — drop it on any machine and run `cxz bootstrap`.

### Building locally

```bash
CX_EMBED_PAYLOAD=1 pixi run build
cp target/release/cx cxz
```

The build downloads all locked packages and bundles them as a zstd-compressed
tar archive inside the binary. This composes with `CX_PACKAGES`, `CX_CHANNELS`,
and `CX_EXCLUDE` — you can build a custom package set and embed it in one step:

```bash
CX_PACKAGES="python >=3.12, conda, numpy" CX_EMBED_PAYLOAD=1 pixi run build
```

### Building via the GitHub Action

```yaml
- uses: jezdez/conda-express@main
  with:
    packages: "python >=3.12, conda, conda-rattler-solver, conda-spawn"
    embed-payload: "true"
```

The action produces a `cxz-<target>` artifact instead of `cx-<target>`.

### Building via the reusable workflow

```yaml
jobs:
  build-cxz:
    uses: jezdez/conda-express/.github/workflows/build.yml@main
    with:
      packages: "python >=3.12, conda, conda-rattler-solver, conda-spawn"
      embed-payload: "true"
```

### Using cxz

```bash
./cxz bootstrap
```

No `--payload`, no `--offline`, no environment variables needed. `cxz` detects
its embedded payload automatically and uses it. All other `cx` flags and
subcommands work identically.

Explicit `--payload` still takes priority over the embedded payload, so you can
override the built-in packages at runtime if needed.
