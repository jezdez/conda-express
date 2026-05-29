# Build a custom conda-pronto bootstrapper

This guide shows how to build a `cx`-like binary with your own package set.
That is conda-pronto's job, not conda-express's job.

Use this guide when you want to distribute a separate bootstrapper with
domain-specific packages such as `numpy`, `pandas`, or internal packages
included by default. The output should have its own binary name and release
channel unless it is an official conda-express `cx` or `cxz` artifact.

## Building locally

Use conda-pronto directly when building locally:

```bash
git clone https://github.com/jezdez/conda-pronto.git
cd conda-pronto

pronto configure \
  --packages "python >=3.12, conda >=25.1, conda-rattler-solver, conda-spawn, numpy, pandas" \
  --channels "conda-forge" \
  --exclude "conda-libmamba-solver"
pixi lock
pronto build --layout none --name serpe
```

The staged binary and metadata files are written to `dist/`. In this example,
the binary is `serpe`, not `cx`.

## Choosing packages

When specifying packages, keep in mind:

- Always include the core set: `python`, `conda`, `conda-rattler-solver`,
  and `conda-spawn` if you want conda-express-like behavior
- Use [MatchSpec](https://conda.io/projects/conda/en/latest/user-guide/concepts/pkg-specs.html)
  syntax for version constraints (e.g. `numpy >=1.26`)
- The source lockfile records the solved package set; conda-pronto derives the
  runtime lock from that solved environment
- The resulting binary is stamped with a runtime lock

## Naming the binary

Use a product-specific name:

```bash
pronto build --layout none --name serpe
```

Avoid publishing custom builds as `cx`. In this documentation, `cx` and `cxz`
mean the official conda-express distribution artifacts.

## Pairing a bundle for offline bootstrap

A custom bootstrap binary built with conda-pronto can be paired with a pre-downloaded set of package
archives for fully offline, air-gapped installation. This is useful for native
installers and restricted-network deployments.

### Preparing the bundle

Run an online bootstrap once to populate the rattler package cache, then
copy the archives into a bundle directory:

```bash
# Bootstrap online to populate the cache.
./serpe bootstrap --prefix /tmp/seed

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

Bundle the custom binary and the bundle directory in your installer. In the
post-install script, bootstrap from the bundle:

```bash
serpe bootstrap --bundle /path/to/bundle --offline
```

### Bundle with network fallback

If you want the bundle to cover most packages but allow network fallback for
any missing ones, omit `--offline`:

```bash
serpe bootstrap --bundle /path/to/bundle
```

This pre-populates the cache from the bundle, then downloads anything not
found locally.

## Building an embedded-bundle binary

conda-pronto can also embed all package archives directly into the binary. The result
is a larger single file that bootstraps conda with zero network access.

In conda-express, this variant is named `cxz`. For your own distribution, use
your own binary name.

### Building locally

```bash
pronto build --layout embedded --name serpe
```

The build downloads all locked packages and stores them as a zstd-compressed
tar archive inside the binary.

### Using the embedded binary

```bash
./serpez bootstrap
```

No `--bundle`, no `--offline`, no environment variables are needed. The runtime
detects its embedded bundle automatically and uses it. All other bootstrap flags
and conda pass-through behavior work identically.

Explicit `--bundle` still takes priority over the embedded bundle, so you can
override the built-in packages at runtime if needed.
