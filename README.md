# conda-express (cx)

[![CI](https://github.com/jezdez/conda-express/actions/workflows/ci.yml/badge.svg)](https://github.com/jezdez/conda-express/actions/workflows/ci.yml)
[![Docs](https://github.com/jezdez/conda-express/actions/workflows/docs.yml/badge.svg)](https://jezdez.github.io/conda-express/)
[![License](https://img.shields.io/github/license/jezdez/conda-express)](https://github.com/jezdez/conda-express/blob/main/LICENSE)
[![PyPI](https://img.shields.io/pypi/v/conda-express)](https://pypi.org/project/conda-express/)

A single-binary bootstrapper for [conda](https://github.com/conda/conda), powered by [rattler](https://github.com/conda/rattler). The `cx` binary is short for **c**onda e**x**press.

conda-express is the distribution project for the `cx` and `cxz` binaries. It
is not an official conda distribution.

cx offers an alternative to the Anaconda Distribution, Miniconda, and Miniforge constructor-style installer pattern: a 7-11 MB native binary that bootstraps a managed conda base environment from a locked package set.

## Quick start

![Bootstrap conda, create an environment, and activate it](demos/quickstart.gif)

```bash
# The first conda command bootstraps the managed prefix
cx info

# Run conda commands through cx
cx create -n myenv python=3.12 numpy pandas
cx create -n science python=3.12 scipy

# Activate environments using conda-spawn, without conda init
cx spawn myenv
```

On first use, cx automatically installs conda and its plugins into `~/.conda/express` from
the built-in runtime lock. Subsequent invocations hand off to the installed
`conda` binary.

## What gets installed

cx installs a managed conda stack from conda-forge:

| Package | Role |
|---|---|
| [python](https://docs.python.org/3/) >= 3.12 | Runtime |
| [conda](https://docs.conda.io/projects/conda/en/stable/) | Package manager |
| [conda-rattler-solver](https://github.com/conda/conda-rattler-solver) | Rust-based solver without libmamba's native dependency chain |
| [conda-spawn](https://conda.github.io/conda-spawn/) >= 0.1.0 | Subprocess-based environment activation |
| [conda-completion](https://conda-incubator.github.io/conda-completion/) >= 0.3.0 | Shell completion support |
| [conda-exec](https://conda-incubator.github.io/conda-exec/) >= 0.3.0 | Ephemeral package execution and PEP 723 script workflows |
| [conda-pypi](https://conda.github.io/conda-pypi/) | PyPI interoperability |
| [conda-self](https://conda.github.io/conda-self/) | Base environment self-management |
| [conda-global](https://conda-incubator.github.io/conda-global/) | Global tool installation and PATH management |
| [conda-workspaces](https://conda-incubator.github.io/conda-workspaces/) >= 0.7.0 | Multi-environment workspace and task management |

See the [included plugins reference](https://jezdez.github.io/conda-express/reference/included-plugins/)
for the commands and workflows these packages add.

Shell completion is available through the included `conda-completion` plugin:

```bash
cx completion status
cx completion install --dry-run --command-name cx
```

Pass `--command-name cx` when installing or generating a completion hook so it
registers `cx` instead of the underlying `conda` executable.

Ad hoc package commands can run through the included `conda-exec` plugin
without adding tools to the managed base prefix:

```bash
cx exec ruff --version
```

The `conda-libmamba-solver` and its 27 exclusive native dependencies (libsolv, libarchive, libcurl, spdlog, etc.) are excluded by default because cx configures `conda-rattler-solver`.

## Versioning

conda-express versions follow the conda version in the runtime lock. If
conda-express needs a packaging-only rebuild without changing the bundled
conda version, it uses a post-release version.

## Installation

### Homebrew (recommended)

Homebrew is the recommended install path on macOS and Linux:

```bash
brew tap jezdez/conda-express https://github.com/jezdez/conda-express
brew install jezdez/conda-express/cx
```

Update later with `brew upgrade cx`.

### Shell script

**macOS / Linux:**

```bash
curl -fsSL https://jezdez.github.io/conda-express/get-cx.sh | sh
```

**Windows (PowerShell):**

```powershell
powershell -ExecutionPolicy ByPass -c "irm https://jezdez.github.io/conda-express/get-cx.ps1 | iex"
```

The script detects your platform, downloads the right binary, verifies the checksum, updates your shell profile / PATH, and runs `cx info` to bootstrap the managed prefix. Customize with environment variables:

- `CX_INSTALL_DIR` — where to place the binary (default: `~/.local/bin` or `%USERPROFILE%\.local\bin`)
- `CX_VERSION` — specific version to install without a `v` prefix (default: `latest`)
- `CX_NO_PATH_UPDATE` — set to skip shell profile / PATH modification
- `CX_NO_BOOTSTRAP` — set to skip the installer's eager bootstrap command
- `CX_SKIP_VERIFY` — set to skip checksum verification
- `CX_PREFIX` — managed prefix used by automatic bootstrap (default: `~/.conda/express`)
- `CX_BUNDLE` — bundle directory used during automatic bootstrap
- `CX_OFFLINE` — set to a truthy value to force offline bootstrap

### GitHub Actions

Use the setup action from a pinned conda-express release tag:

```yaml
steps:
  - uses: jezdez/conda-express/.github/actions/setup-cx@<release-tag>
  - run: cx info
```

The action downloads the matching `cx` release asset, verifies its checksum,
adds `cx` to `PATH`, and runs `cx info` by default. That first conda command
automatically bootstraps the managed prefix. Artifact Attestation
verification is available with `verify-attestation: true`.

See the [GitHub Actions guide](https://jezdez.github.io/conda-express/guides/use-cx-in-github-actions/)
for automatic bootstrap, deferred bootstrap, and attestation examples.

### From GitHub Releases

Download the binary for your platform from the
[latest release](https://github.com/jezdez/conda-express/releases/latest):

| Platform | File |
|---|---|
| Linux x86_64 | `cx-x86_64-unknown-linux-gnu` |
| Linux ARM64 | `cx-aarch64-unknown-linux-gnu` |
| macOS x86_64 (Intel) | `cx-x86_64-apple-darwin` |
| macOS ARM64 (Apple Silicon) | `cx-aarch64-apple-darwin` |
| Windows x86_64 | `cx-x86_64-pc-windows-msvc.exe` |

Windows ARM64 is not published for conda-express yet. conda-ship publishes
Windows ARM64 builder assets, but full runtime bootstrap support is still gated
by the conda package ecosystem.

Each file has matching `.sha256`, `.info.json`, `.packages.txt`, and
`.runtime.lock` files. Release artifacts are also covered by GitHub Artifact
Attestations:

```bash
gh attestation verify ./cx-x86_64-unknown-linux-gnu \
  -R jezdez/conda-express \
  --signer-workflow jezdez/conda-express/.github/workflows/release.yml
```

See the [artifact verification guide](https://jezdez.github.io/conda-express/guides/verify-release-artifacts/)
for checksum, metadata, runtime lock, and air-gapped transfer checks.

### Docker

A multi-arch image is published to GHCR on every release:

```bash
docker run --rm -v cx-data:/home/nonroot/.conda/express ghcr.io/jezdez/conda-express info
```

The image runs as non-root (uid 65532), can run with a read-only root
filesystem when the managed prefix is mounted as a writable volume, and
includes provenance attestations and SBOMs. Docker Desktop on macOS and
Windows automatically selects the right architecture (linux/amd64 or
linux/arm64).

```bash
docker run --rm --read-only --tmpfs /tmp \
  -v cx-data:/home/nonroot/.conda/express \
  ghcr.io/jezdez/conda-express info
```

```bash
# Run a conda command through cx
docker run --rm -v cx-data:/home/nonroot/.conda/express ghcr.io/jezdez/conda-express create -n myenv python=3.12
```

Use as a container in CI (GitHub Actions):

```yaml
jobs:
  test:
    container: ghcr.io/jezdez/conda-express:latest
    steps:
      - run: cx create -n test python numpy
```

Use as a base for multi-stage application builds:

```dockerfile
FROM ghcr.io/jezdez/conda-express:latest AS conda-builder
RUN cx create -n app python numpy pandas
FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=conda-builder /home/nonroot/.conda/express/envs/app /opt/conda
```

### PyPI

```bash
pip install conda-express
```

The PyPI package installs the `cx` release binary built with conda-ship for
your platform.

## Upgrading from early releases

Current `cx` releases bootstrap into `~/.conda/express`. Early releases used
`~/.cx`; upgrading the binary does not migrate that prefix automatically. Keep
`~/.cx` until you have recreated or archived any environments you still need.
If an old Cargo-installed `cx` is still on your `PATH`, remove it because
conda-express no longer publishes new crates.io releases.

See the [upgrade guide](https://jezdez.github.io/conda-express/guides/upgrade-from-early-cx/)
for commands to export old environments, bootstrap the new prefix, and remove
the old directory safely.

## Reproducing distribution artifacts

The `cx` and `cxz` artifacts published from this repository are built with
[conda-ship](https://github.com/jezdez/conda-ship). This repository keeps the
conda-express distribution defaults and delegates the generic runtime and
builder implementation to conda-ship.

Use this repository's release workflow to reproduce these conda-express
artifacts. For custom package sets, binary names, or release channels, use
[conda-ship](https://github.com/jezdez/conda-ship) directly.

## Configuration

The conda-express runtime is defined in `pyproject.toml`. Pixi solves the
`runtime` source environment, and conda-ship derives the runtime lock from that
committed lockfile:

```toml
[tool.conda-ship]
runtime-name = "cx"
runtime-version = { from = "project-metadata" }
delegate-executable = "conda"
artifact-layout = "online"
source-environment = "runtime"
exclude-packages = ["conda-libmamba-solver"]
condarc-file = "runtime.condarc"
freeze-base = true
docs-url = "https://jezdez.github.io/conda-express/"
install-scheme = "conda-home"
install-name = "express"
```

## CLI reference

```
cx <conda-args>                  Bootstrap if needed, then run conda
cx info                          Show conda and prefix information
cx spawn [ENV]                   Open a subshell through conda-spawn
cx self <command>                Run conda-self commands

CX_PREFIX=DIR cx <conda-args>    Use a different managed prefix
CX_BUNDLE=DIR cx <conda-args>    Seed automatic bootstrap from a bundle
CX_OFFLINE=1 cx <conda-args>     Disable network access during bootstrap
```

`cx --help`, `cx --version`, and every subcommand belong to the installed
conda CLI. The bootstrapper does not reserve its own commands.

### Frozen base prefix

The `~/.conda/express` prefix is protected with a [CEP 22](https://conda.org/learn/ceps/cep-0022/) frozen marker after bootstrap. This prevents accidental modification of the base environment (e.g., `conda install numpy` into base). Users should create named environments for their work:

```bash
cx create -n myenv numpy pandas
cx spawn myenv
```

Bootstrap also writes constructor-compatible prefix metadata:
`conda-meta/history` and `conda-meta/initial-state.explicit.txt`. Conda can
recognize the managed prefix as an environment, and the included `conda-self`
plugin can use the initial-state snapshot:

```bash
cx self reset --snapshot installer-exact
```

The snapshot belongs to the prefix that was created during its first
bootstrap. Installing a newer `cx` binary does not replace that snapshot, and
`conda self reset` does not apply the newer binary's stamped runtime lock.

## Building custom binaries

For custom package sets or new distributions, use
[conda-ship](https://github.com/jezdez/conda-ship) directly. This repository's build
workflow is release preparation for this repository's `cx` and `cxz`
binaries, not a generic downstream builder interface.

## Uninstalling

Remove `cx` with the same method that installed it. For example, use
`brew uninstall cx` for Homebrew or `python -m pip uninstall conda-express`
for PyPI. A standalone script installation can be removed by deleting the
installed binary and its PATH entry.

Those operations do not delete `~/.conda/express` or its named environments.
Export anything you need before removing that directory manually. Until
conda-self gains a conda-express adapter, there is no `cx uninstall` command.

## How it works

1. **Build time**: Pixi solves the `runtime` source environment into
   `pixi.lock`; the conda-express release workflow asks conda-ship to derive a
   runtime lock, filter excluded packages, and stamp that lock into the staged
   binary.

2. **First run**: cx reads the stamped runtime lock, downloads packages from
   conda-forge, installs them into the prefix, and writes conda-compatible
   prefix metadata. No repodata fetch or solve needed at runtime.

3. **Subsequent runs**: cx detects the existing prefix and replaces its own process with the installed `conda` binary, passing all arguments through.

## Activation model

![cx delegates conda commands after bootstrap](demos/passthrough.gif)

cx ships with conda-spawn instead of traditional `conda activate`. There is no need to run `conda init` or modify shell profiles.

```bash
# Optional: expose the managed conda executable directly
export PATH="$HOME/.conda/express/condabin:$PATH"

# Activate an environment (spawns a subshell)
cx spawn myenv

# Deactivate by exiting the subshell
exit
```

## Lockfile format

The stamped runtime lock uses the [rattler-lock v6](https://github.com/conda/rattler/tree/main/crates/rattler_lock) format (same as `pixi.lock`). It can be:

- Read by pixi
- Imported by conda-lockfiles
- Checked into version control for reproducibility auditing

## License

BSD 3-Clause. See [LICENSE](LICENSE).
