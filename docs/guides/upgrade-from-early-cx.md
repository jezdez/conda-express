# Upgrade From Early cx Versions

Use this guide when you installed an early `cx` release before the
conda-ship-based runtime split and want to move to the current conda-express
release.

## What Changed

Current `cx` releases bootstrap the managed conda prefix at:

```text
~/.conda/express
```

Early releases used `~/.cx`. Upgrading the `cx` binary does not rewrite or
migrate that old prefix. The first run of the new binary creates a fresh
managed prefix at `~/.conda/express`; the old `~/.cx` directory remains until
you remove it.

:::{important}
Keep `~/.cx` until you have recreated or archived any environments you still
need. Current runtimes do not provide an uninstall command, and upgrading the
binary does not remove old early-release state.
:::

## Upgrade The Binary

Install a current `cx` binary with one of the supported release channels.
Current conda-express releases are built with conda-ship and are distributed
through Homebrew, the installer scripts, GitHub Releases, Docker, and PyPI.
New releases are no longer published to crates.io. Common local-binary upgrade
paths are below.

::::{tab-set}

:::{tab-item} Homebrew
```bash
brew update
brew upgrade cx
```
:::

:::{tab-item} Installer Script
```bash
curl -fsSL https://jezdez.github.io/conda-express/get-cx.sh | sh
```

On Windows:

```powershell
powershell -ExecutionPolicy ByPass -c "irm https://jezdez.github.io/conda-express/get-cx.ps1 | iex"
```
:::

:::{tab-item} PyPI
```bash
python -m pip install --upgrade conda-express
```
:::

:::{tab-item} Old Cargo Install
If you installed an early release with `cargo install`, remove that old binary
and then choose one of the supported install methods above:

```bash
cargo uninstall conda-express
```

This removes the Cargo-installed `cx` executable. It does not remove the old
`~/.cx` prefix.
:::

::::

On macOS and Linux, check which binary your shell finds:

```bash
command -v cx
```

On Windows PowerShell:

```powershell
Get-Command cx
```

If that still points at an old Cargo install, for example under
`~/.cargo/bin` or `%USERPROFILE%\.cargo\bin`, remove the old binary or adjust
your `PATH` before continuing.

## Run The First Conda Command

Run:

```bash
cx info
```

The command automatically bootstraps the new prefix, then prints conda's
environment information. The root prefix should be `~/.conda/express` unless
you set `CX_PREFIX`.

`cx --version` and `cx --help` also delegate to conda. They report conda's
version and help after automatic bootstrap, not a separate bootstrapper CLI.

## Move Environments You Still Need

If your old `~/.cx` prefix contains named environments you still use, export
and recreate them deliberately.

For a portable environment file:

```bash
~/.cx/bin/conda env export --from-history -n myenv > myenv.yml
cx env create -n myenv -f myenv.yml
```

For an exact same-platform package specification:

```bash
~/.cx/bin/conda list --explicit -n myenv > myenv-spec.txt
cx create -n myenv --file myenv-spec.txt
```

On Windows, the old conda executable is typically under:

```powershell
$HOME\.cx\Scripts\conda.exe
```

Use that path in place of `~/.cx/bin/conda` in the commands above.

:::{note}
Do not point the new `cx` runtime at `~/.cx` as a migration shortcut. Current
releases use runtime ownership metadata to avoid mutating prefixes that were
created by another runtime generation.
:::

## Remove The Old Prefix

After the environments you need are available under the new prefix, remove the
old early-release prefix manually.

macOS and Linux:

```bash
rm -rf ~/.cx
```

Windows PowerShell:

```powershell
Remove-Item -Recurse -Force "$HOME\.cx"
```

Then use the current runtime:

```bash
cx create -n myenv python=3.12
cx spawn myenv
```
