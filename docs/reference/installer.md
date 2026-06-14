# Installer reference

:::{tip}
On macOS and Linux, [Homebrew](../quickstart.md) is the recommended
installation method. The shell scripts below are an alternative for
environments without Homebrew (CI, containers, Windows).
:::

cx provides standalone installer scripts for macOS, Linux, and Windows. These
scripts automate downloading the correct binary, verifying its checksum,
setting up your PATH, and optionally running `cx bootstrap`.

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
7. **Runs `cx bootstrap`** to set up the conda environment

If the installer finds `~/.cx`, it leaves that directory alone. Current
releases bootstrap into `~/.conda/express`; see
{doc}`../guides/upgrade-from-early-cx` before removing old early-release
environments.

For manual downloads or mirrored installer inputs, see
{doc}`../guides/verify-release-artifacts` for checksum, attestation, and
metadata checks.

## GitHub Actions

Use the setup action when a workflow needs `cx` on `PATH`:

```yaml
permissions:
  contents: read
  attestations: read

steps:
  - uses: jezdez/conda-express/.github/actions/setup-cx@<release-tag>
    with:
      github-token: ${{ github.token }}
  - run: cx status
```

Pin the action to a conda-express release tag for reproducible CI. When
`version` is omitted and the action ref is a release tag, the action installs
that same version. If the action ref is not a release tag, it resolves and
installs the latest release.

The action downloads the platform-specific `cx` asset, checks the published
`.sha256` file, verifies GitHub Artifact Attestations from
`.github/workflows/release.yml`, optionally bootstraps the managed conda
prefix, and adds the install directory to `PATH`.

Useful inputs:

| Input | Default | Description |
|---|---|---|
| `version` | action ref when it is a release tag, otherwise `latest` | conda-express release version to install |
| `github-token` | empty | GitHub token used for latest-release lookup and Artifact Attestation verification |
| `install-dir` | runner temp directory | Directory to place the `cx` binary |
| `bootstrap` | `true` | Run `cx bootstrap` after installation |
| `add-to-path` | `true` | Add the install directory to `GITHUB_PATH` |
| `verify-attestation` | `true` | Verify the downloaded binary with GitHub Artifact Attestations |

Set `bootstrap: false` for workflows that only need to inspect the binary or
defer bootstrap to a later step. When `verify-attestation` is enabled, pass
`github-token: ${{ github.token }}` and grant `attestations: read` to the job.

## Options

All options are set via environment variables and work on both platforms.

| Variable | Default | Description |
|---|---|---|
| `CX_INSTALL_DIR` | `~/.local/bin` (Unix) or `%USERPROFILE%\.local\bin` (Windows) | Directory to place the `cx` binary |
| `CX_VERSION` | `latest` | Version to install (without `v` prefix, e.g. `26.5.2`) |
| `CX_NO_PATH_UPDATE` | *(unset)* | Set to any value to skip shell profile / PATH modification |
| `CX_NO_BOOTSTRAP` | *(unset)* | Set to any value to skip running `cx bootstrap` after install |
| `CX_SKIP_VERIFY` | *(unset)* | Set to any value to skip checksum verification |
| `CX_BUNDLE` | *(unset)* | Bundle directory containing `.conda`/`.tar.bz2` archives used by `cx bootstrap` |
| `CX_OFFLINE` | *(unset)* | Set to any truthy value to disable network during bootstrap |

`CX_VERSION` selects a conda-express release. Release versions follow the conda
runtime version in the lock; `26.5.2` bootstraps conda `26.5.2`, while
`26.5.2.post1` would be a conda-express-only rebuild with the same conda
runtime package.

:::{tip}
For disconnected deployments, consider using `cxz` instead of `cx` with
`CX_BUNDLE` / `CX_OFFLINE`. The `cxz` binary embeds the locked package
archives and requires no separate bundle directory. See
{doc}`../guides/offline-and-airgapped` for details.
:::

### Examples

Install a specific version:

```bash
curl -fsSL https://jezdez.github.io/conda-express/get-cx.sh | env CX_VERSION=26.5.2 sh
```

Install to a custom directory without bootstrap:

```bash
curl -fsSL https://jezdez.github.io/conda-express/get-cx.sh | env CX_INSTALL_DIR=/opt/bin CX_NO_BOOTSTRAP=1 sh
```

PowerShell with options:

```powershell
$Env:CX_VERSION = "26.5.2"; irm https://jezdez.github.io/conda-express/get-cx.ps1 | iex
```

The PowerShell script also accepts named parameters when invoked directly:

```powershell
.\get-cx.ps1 -Version 26.5.2 -InstallDir C:\tools -NoBootstrap
```

For an offline `cx` bootstrap with a bundle directory:

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

Windows ARM64 is not published for conda-express yet. conda-ship 0.3.0 has
Windows ARM64 builder assets, but full runtime bootstrap support still depends
on the conda package ecosystem. The PowerShell installer reports that
architecture as unsupported instead of downloading an incompatible binary.

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

Use the built-in command to remove the managed prefix:

```bash
cx uninstall
```

This removes the conda prefix (including all named environments) and any
PATH entries added by the installer. It shows the paths it plans to remove and
asks for confirmation (use `--yes` to skip).

After completion, cx prints a hint for removing the binary itself through the
install method it detects, such as Homebrew or PyPI.

See {ref}`cx uninstall <cli-cx-uninstall>` for full details.

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
