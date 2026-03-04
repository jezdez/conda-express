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
5. **Updates your shell profile / PATH** so `cx` is available in new shells
6. **Runs `cx bootstrap`** to set up the conda environment

## Options

All options are set via environment variables and work on both platforms.

| Variable | Default | Description |
|---|---|---|
| `CX_INSTALL_DIR` | `~/.local/bin` (Unix) or `%USERPROFILE%\.local\bin` (Windows) | Directory to place the `cx` binary |
| `CX_VERSION` | `latest` | Version to install (without `v` prefix, e.g. `0.1.3`) |
| `CX_NO_PATH_UPDATE` | *(unset)* | Set to any value to skip shell profile / PATH modification |
| `CX_NO_BOOTSTRAP` | *(unset)* | Set to any value to skip running `cx bootstrap` after install |

### Examples

Install a specific version:

```bash
CX_VERSION=0.1.3 curl -fsSL https://jezdez.github.io/conda-express/get-cx.sh | sh
```

Install to a custom directory without bootstrap:

```bash
CX_INSTALL_DIR=/opt/bin CX_NO_BOOTSTRAP=1 curl -fsSL https://jezdez.github.io/conda-express/get-cx.sh | sh
```

PowerShell with options:

```powershell
$Env:CX_VERSION = "0.1.3"; irm https://jezdez.github.io/conda-express/get-cx.ps1 | iex
```

The PowerShell script also accepts named parameters when invoked directly:

```powershell
.\get-cx.ps1 -Version 0.1.3 -InstallDir C:\tools -NoBootstrap
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

The easiest way to uninstall is the built-in command:

```bash
cx uninstall
```

This removes the conda prefix (including all named environments), the cx
binary, and any PATH entries added by the installer. It will show what will
be removed and ask for confirmation (use `--yes` to skip).

See {ref}`cx uninstall <cli-cx-uninstall>` for full details.

### Manual uninstall

If the `cx` command is unavailable, you can remove everything manually:

1. Delete the binary:

   ```bash
   rm ~/.local/bin/cx
   ```

2. Remove the conda prefix:

   ```bash
   rm -rf ~/.cx
   ```

3. Remove the PATH line from your shell profile if the installer added one.
