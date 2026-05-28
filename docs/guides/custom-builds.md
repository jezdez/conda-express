# Build a custom cx binary

This guide shows how to build a Pronto-backed cx binary with your own set of
conda packages. This is useful when you want to distribute a bootstrapper that
includes domain-specific packages (e.g. numpy, pandas) out of the box.

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

### Using cxz

```bash
./cxz bootstrap
```

No `--bundle`, no `--offline`, no environment variables needed. `cxz` detects
its embedded bundle automatically and uses it. All other `cx` flags and
subcommands work identically.

Explicit `--bundle` still takes priority over the embedded bundle, so you can
override the built-in packages at runtime if needed.
