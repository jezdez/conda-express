# Build a custom cx binary

This guide shows how to build a Pronto-backed cx binary with your own set of
conda packages. This is useful when you want to distribute a bootstrapper that
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
          embed-bundle: "false"

      - uses: actions/upload-artifact@v4
        with:
          name: ${{ steps.cx.outputs.asset-name }}
          path: |
            ${{ steps.cx.outputs.binary-path }}
            ${{ steps.cx.outputs.checksums-path }}
            ${{ steps.cx.outputs.info-path }}
            ${{ steps.cx.outputs.lock-path }}
            ${{ steps.cx.outputs.package-list-path }}
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
      embed-bundle: "false"
```

Binary artifacts for all platforms are uploaded automatically.

## Building locally

Use Pronto directly when building locally:

```bash
git clone https://github.com/jezdez/pronto.git
cd pronto

cargo run -p pronto -- configure \
  --packages "python >=3.12, conda >=25.1, conda-rattler-solver, conda-spawn, numpy"
pixi lock
cargo run -p pronto -- build --layout none --name cx
```

The staged binary and metadata files are written to `dist/`.

## Choosing packages

When specifying packages, keep in mind:

- Always include the core set: `python`, `conda`, `conda-rattler-solver`,
  and `conda-spawn` (cx depends on these at runtime)
- Use [MatchSpec](https://conda.io/projects/conda/en/latest/user-guide/concepts/pkg-specs.html)
  syntax for version constraints (e.g. `numpy >=1.26`)
- Pronto performs a full dependency solve at build time, so all transitive
  dependencies are resolved and locked
- The resulting binary is self-contained with an embedded lockfile

## Pairing a bundle for offline bootstrap

A custom cx binary can be paired with a pre-downloaded set of package archives
for fully offline, air-gapped installation. This is useful for native installers
(macOS PKG, Windows MSI) and restricted-network deployments.

### Preparing the bundle

Run an online bootstrap once to populate the rattler package cache, then
copy the archives into a bundle directory:

```bash
# Bootstrap online to populate the cache
cx bootstrap --prefix /tmp/seed

# Collect the cached archives
mkdir bundle
cp ~/Library/Caches/rattler/cache/pkgs/*.conda bundle/
cp ~/Library/Caches/rattler/cache/pkgs/*.tar.bz2 bundle/ 2>/dev/null || true
```

:::{note}
The rattler cache location varies by platform: `~/Library/Caches/rattler`
on macOS, `~/.cache/rattler` on Linux, and `%LOCALAPPDATA%\rattler` on
Windows.
:::

### Using the bundle

Bundle the cx binary and the bundle directory in your installer. In the
post-install script, bootstrap from the bundle:

```bash
cx bootstrap --bundle /path/to/bundle --offline
```

Or using environment variables (convenient for installer scripts):

```bash
CX_BUNDLE=/path/to/bundle CX_OFFLINE=1 cx bootstrap
```

### Bundle with network fallback

If you want the bundle to cover most packages but allow network fallback for
any missing ones, omit `--offline`:

```bash
cx bootstrap --bundle /path/to/bundle
```

This pre-populates the cache from the bundle, then downloads anything not
found locally.

## Building cxz (self-contained binary)

`cxz` is a variant of `cx` that embeds all package archives directly into the
binary. The result is a single 50-95 MB file (varies by platform) that bootstraps conda with zero
network access — drop it on any machine and run `cxz bootstrap`.

### Building locally

```bash
cargo run -p pronto -- build --layout embedded --name cx
```

The build downloads all locked packages and stores them as a zstd-compressed
tar archive inside the binary.

### Building via the GitHub Action

```yaml
- uses: jezdez/conda-express@main
  with:
    packages: "python >=3.12, conda, conda-rattler-solver, conda-spawn"
    embed-bundle: "true"
```

The action produces a `cxz-<target>` artifact instead of `cx-<target>`.

### Building via the reusable workflow

```yaml
jobs:
  build-cxz:
    uses: jezdez/conda-express/.github/workflows/build.yml@main
    with:
      packages: "python >=3.12, conda, conda-rattler-solver, conda-spawn"
      embed-bundle: "true"
```

### Using cxz

```bash
./cxz bootstrap
```

No `--bundle`, no `--offline`, no environment variables needed. `cxz` detects
its embedded bundle automatically and uses it. All other `cx` flags and
subcommands work identically.

Explicit `--bundle` still takes priority over the embedded bundle, so you can
override the built-in packages at runtime if needed.
