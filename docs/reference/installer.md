# Installer reference

:::{tip}
On macOS and Linux, [Homebrew](../quickstart.md) is the recommended
installation method. The shell scripts below are an alternative for
environments without Homebrew (CI, containers, Windows).
:::

cx provides standalone installer scripts for macOS, Linux, and Windows. These
scripts automate downloading the correct binary, verifying its checksum,
setting up your PATH, and optionally running `cx info` to trigger automatic
bootstrap.

## Usage

**macOS / Linux:**

```bash
curl -fsSL https://jezdez.github.io/conda-express/get-cx.sh | sh
```

or with `wget`:

```bash
wget -qO- https://jezdez.github.io/conda-express/get-cx.sh | sh
```

**Windows (PowerShell):**

```powershell
powershell -ExecutionPolicy ByPass -c "irm https://jezdez.github.io/conda-express/get-cx.ps1 | iex"
```

:::{tip}
You can inspect the scripts before running them:

```bash
curl -fsSL https://jezdez.github.io/conda-express/get-cx.sh | less
```

```powershell
irm https://jezdez.github.io/conda-express/get-cx.ps1 | more
```
:::

## What the installer does

1. **Detects your platform** — operating system and CPU architecture
2. **Downloads the binary** from [GitHub Releases](https://github.com/jezdez/conda-express/releases)
3. **Verifies the SHA256 checksum** against the published `.sha256` file
4. **Installs the binary** to the install directory
5. **Updates your shell profile / PATH** so the installed `cx` binary is found
   in new shells
6. **Warns about early-release state** if `~/.cx` still exists
7. **Runs `cx info`** to bootstrap the conda environment and print conda information

If the installer finds `~/.cx`, it leaves that directory alone. Current
releases bootstrap into `~/.conda/express`; see
{doc}`../guides/upgrade-from-early-cx` before removing old early-release
environments.

For manual downloads or mirrored installer inputs, see
{doc}`../guides/verify-release-artifacts` for checksum, attestation, and
metadata checks.

## GitHub Actions

For task-oriented examples, see {doc}`../guides/use-cx-in-github-actions`.
Use the setup action when a workflow needs `cx` on `PATH`:

```yaml
steps:
  - uses: jezdez/conda-express/.github/actions/setup-cx@<release-tag>
  - run: cx info
```

Pin the action to a conda-express release tag for reproducible CI. When
`version` is omitted and the action ref is a release tag, the action installs
that same version. If the action ref is not a release tag, it resolves and
installs the latest release.

The action downloads the platform-specific `cx` asset, checks the published
`.sha256` file, optionally verifies GitHub Artifact Attestations from
`.github/workflows/release.yml`, optionally bootstraps the managed conda
prefix by running a conda command, and adds the install directory to `PATH`.

Useful inputs:

| Input | Default | Description |
|---|---|---|
| `version` | action ref when it is a release tag, otherwise `latest` | conda-express release version to install |
| `github-token` | empty | GitHub token used for latest-release lookup and Artifact Attestation verification |
| `install-dir` | runner temp directory | Directory to place the `cx` binary |
| `bootstrap` | `true` | Run `cx info` after installation to trigger automatic bootstrap |
| `add-to-path` | `true` | Add the install directory to `GITHUB_PATH` |
| `verify-attestation` | `false` | Verify the downloaded binary with GitHub Artifact Attestations |

Set `bootstrap: false` for workflows that only need the downloaded binary or
want to defer prefix creation until a later `cx` command.

Enable provenance verification for stricter CI:

```yaml
permissions:
  contents: read
  attestations: read

steps:
  - uses: jezdez/conda-express/.github/actions/setup-cx@<release-tag>
    with:
      verify-attestation: true
      github-token: ${{ github.token }}
```

## Options

All options are set via environment variables and work on both platforms.

| Variable | Default | Description |
|---|---|---|
| `CX_INSTALL_DIR` | `~/.local/bin` (Unix) or `%USERPROFILE%\.local\bin` (Windows) | Directory to place the `cx` binary |
| `CX_VERSION` | `latest` | Version to install (without `v` prefix, e.g. `{{ conda_express_release }}`) |
| `CX_NO_PATH_UPDATE` | *(unset)* | Set to any value to skip shell profile / PATH modification |
| `CX_NO_BOOTSTRAP` | *(unset)* | Set to any value to skip the installer's eager bootstrap command |
| `CX_SKIP_VERIFY` | *(unset)* | Set to any value to skip checksum verification |
| `CX_PREFIX` | `~/.conda/express` | Managed prefix used by runtime invocations while the variable is set |
| `CX_BUNDLE` | *(unset)* | Bundle directory containing `.conda`/`.tar.bz2` archives used during automatic bootstrap |
| `CX_OFFLINE` | *(unset)* | Set to any truthy value to disable network during bootstrap |

`CX_PREFIX`, `CX_BUNDLE`, and `CX_OFFLINE` are runtime bootstrap controls that
the installer passes to `cx info`. The installer does not persist
`CX_PREFIX`. Set it again for later commands that should use a non-default
prefix.

`CX_VERSION` selects a conda-express release. Release versions follow the conda
runtime version in the lock; `{{ conda_express_release }}` bootstraps conda
`{{ conda_runtime_version }}`, while
`{{ conda_express_post_release_example }}` would be a conda-express-only
rebuild with the same conda runtime package.

:::{tip}
For disconnected deployments, consider using `cxz` instead of `cx` with
`CX_BUNDLE` / `CX_OFFLINE`. The `cxz` binary embeds the locked package
archives and requires no separate bundle directory. See
{doc}`../guides/offline-and-airgapped` for details.
:::

### Examples

Install a specific version:

```bash
curl -fsSL https://jezdez.github.io/conda-express/get-cx.sh | env CX_VERSION={{ conda_express_release }} sh
```

Install to a custom directory without eagerly bootstrapping the prefix:

```bash
curl -fsSL https://jezdez.github.io/conda-express/get-cx.sh | env CX_INSTALL_DIR=/opt/bin CX_NO_BOOTSTRAP=1 sh
```

PowerShell with options:

```powershell
$Env:CX_VERSION = "{{ conda_express_release }}"; irm https://jezdez.github.io/conda-express/get-cx.ps1 | iex
```

The PowerShell script also accepts named parameters when invoked directly:

```powershell
.\get-cx.ps1 -Version {{ conda_express_release }} -InstallDir C:\tools -NoBootstrap
```

For an offline automatic bootstrap with a bundle directory:

```powershell
.\get-cx.ps1 -Bundle C:\mirror\cx-packages -Offline
```

## Platform detection

The shell script uses `uname -s` and `uname -m` to detect the platform.
The PowerShell script uses .NET's `RuntimeInformation.OSArchitecture`.

| Detected | Target | Binary |
|---|---|---|
| Linux x86_64 | `x86_64-unknown-linux-gnu` | `cx-x86_64-unknown-linux-gnu` |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | `cx-aarch64-unknown-linux-gnu` |
| macOS x86_64 | `x86_64-apple-darwin` | `cx-x86_64-apple-darwin` |
| macOS ARM64 | `aarch64-apple-darwin` | `cx-aarch64-apple-darwin` |
| Windows x86_64 | `x86_64-pc-windows-msvc` | `cx-x86_64-pc-windows-msvc.exe` |

Windows ARM64 is not published for conda-express yet. conda-ship has Windows
ARM64 builder assets, but full runtime bootstrap support still depends on the
conda package ecosystem. The PowerShell installer reports that architecture as
unsupported instead of downloading an incompatible binary.

## Shell profile updates

The shell script appends a `PATH` export to your shell configuration file:

| Shell | File |
|---|---|
| bash | `~/.bashrc` |
| zsh | `~/.zshrc` |
| fish | `~/.config/fish/config.fish` |

The PowerShell script updates the user `PATH` in the Windows registry
(`HKCU:\Environment`) and broadcasts a `WM_SETTINGCHANGE` message so
running applications pick up the change.

Set `CX_NO_PATH_UPDATE` to skip this step if you prefer to manage your
PATH manually.

## Uninstalling

Remove the binary through the method that installed it:

```bash
brew uninstall cx
```

For a PyPI installation:

```bash
python -m pip uninstall conda-express
```

For a standalone script installation, delete the installed `cx` binary and
remove the PATH line that the script added to your shell profile. On Windows,
remove the install directory from the user PATH.

These methods do not remove `~/.conda/express` or its named environments.
Export anything you need before removing that prefix manually. There is no
runtime-owned uninstall command until conda-self provides a conda-express
adapter.

### Manual uninstall

If the `cx` command is unavailable, you can remove everything manually:

1. Delete the binary:

   ```bash
   rm ~/.local/bin/cx
   ```

2. Remove the conda prefix:

   ```bash
   rm -rf ~/.conda/express
   ```

3. Remove the PATH line from your shell profile if the installer added one.
