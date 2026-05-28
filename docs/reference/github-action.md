# GitHub Action reference

cx provides a composite GitHub Action and a reusable workflow for building
Pronto-backed `cx` and `cxz` binaries with the conda-express distribution
defaults or a custom package set.

## Composite action

The composite action builds a binary for the current runner's platform. Use a
matrix strategy for multi-platform builds.

```
uses: jezdez/conda-express@<ref>
```

### Inputs

`packages` {bdg-secondary}`optional`
: Comma-separated conda package specs to include in the bootstrapper.
  When empty, uses the conda-express distribution package set.

  Example: `"python >=3.12, conda >=25.1, numpy, pandas"`

`channels` {bdg-secondary}`optional`
: Comma-separated conda channels. When empty, uses the conda-express channel
  set (`conda-forge`).

  Example: `"conda-forge, bioconda"`

`exclude` {bdg-secondary}`optional`
: Comma-separated packages to exclude from the bootstrapper, along with their
  exclusive dependencies. When empty, uses the conda-express exclusions.

  Example: `"conda-libmamba-solver"`

`pronto-ref` {bdg-secondary}`optional`
: Git ref of Pronto to use for the build. Defaults to the pinned Pronto commit
  used by this conda-express release line.

`embed-bundle` {bdg-secondary}`optional` {bdg-info}`default: "false"`
: Embed all locked package archives into the binary for offline bootstrap.
  The output binary uses the `cxz-<target>` artifact name instead of
  `cx-<target>`.

### Outputs

`binary-path`
: Absolute path to the built binary on the runner.

`asset-name`
: Platform-qualified asset name, such as `cx-aarch64-apple-darwin` or
  `cxz-aarch64-apple-darwin` when `embed-bundle` is `"true"`.

`info-path`
: Absolute path to the artifact info JSON.

`lock-path`
: Absolute path to the staged artifact lock.

`package-list-path`
: Absolute path to the package list.

`checksums-path`
: Absolute path to the SHA256 checksum file.

### What it does

1. Resolves the conda-express defaults unless package, channel, or exclude
   inputs are supplied.
2. Invokes the pinned Pronto action.
3. Builds either the online `cx` artifact or the embedded-bundle `cxz` artifact.
4. Stages the binary plus `.sha256`, `.info.json`, `.artifact.lock`, and
   `.packages.txt` metadata.

---

## Reusable workflow

The reusable workflow builds cx for all 5 supported platforms in a single call,
using the composite action internally.

```
uses: jezdez/conda-express/.github/workflows/build.yml@<ref>
```

### Inputs

All inputs from the composite action are supported, plus:

`retention-days` {bdg-secondary}`optional` {bdg-info}`default: 7`
: Number of days to retain build artifacts.

### Artifacts

The workflow uploads one artifact per platform. Each artifact contains the
binary and the Pronto metadata files:

| Artifact | Platform |
|---|---|
| `cx-x86_64-unknown-linux-gnu` | Linux x86_64 |
| `cx-aarch64-unknown-linux-gnu` | Linux ARM64 |
| `cx-x86_64-apple-darwin` | macOS Intel |
| `cx-aarch64-apple-darwin` | macOS Apple Silicon |
| `cx-x86_64-pc-windows-msvc.exe` | Windows x86_64 |

Use `embed-bundle: "true"` to produce `cxz-*` artifacts instead.
