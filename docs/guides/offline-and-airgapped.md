# Offline and Air-Gapped Installs

`cx` supports two offline deployment styles:

- `cxz`: one binary with the locked package archives embedded.
- `cx` plus an external package bundle: one small binary and a separate
  directory of `.conda` or `.tar.bz2` archives.

Use `cxz` when the simplest transfer artifact is a single file. Use an
external bundle when an installer system, container build, or enterprise
software distribution process already manages payload files separately.

## Use cxz

Download the `cxz` file for the target platform from the release page, make it
executable on Unix, then bootstrap:

```bash
chmod +x cxz-x86_64-unknown-linux-gnu
./cxz-x86_64-unknown-linux-gnu bootstrap --prefix /opt/cx
```

The embedded bundle is detected automatically. You do not need `--bundle` or
`--offline`.

## Use an external bundle

When package archives are stored separately, point `cx` at the bundle and force
offline mode:

```bash
cx bootstrap \
  --prefix /opt/cx \
  --bundle /path/to/packages \
  --offline
```

The same settings are available as environment variables for installer scripts
and CI jobs:

```bash
CX_BUNDLE=/path/to/packages CX_OFFLINE=1 cx bootstrap --prefix /opt/cx
```

## Use the installer script

The shell and PowerShell installers pass offline settings through to
`cx bootstrap`:

```bash
CX_BUNDLE=/path/to/packages CX_OFFLINE=1 \
  curl -fsSL https://jezdez.github.io/conda-express/get-cx.sh | sh
```

On Windows:

```powershell
$Env:CX_BUNDLE = "C:\packages"
$Env:CX_OFFLINE = "1"
irm https://jezdez.github.io/conda-express/get-cx.ps1 | iex
```

## Verify artifacts before use

Each GitHub Release artifact has a matching checksum and GitHub Artifact
Attestation. For example:

```bash
gh attestation verify ./cx-x86_64-unknown-linux-gnu \
  -R jezdez/conda-express \
  --signer-workflow jezdez/conda-express/.github/workflows/release.yml
```

For disconnected environments, perform verification before transferring the
binary or bundle across the boundary.
