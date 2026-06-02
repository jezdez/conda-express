# Offline and Air-Gapped Installs

`cx` supports two offline deployment styles:

- `cxz`: one binary with the locked package archives embedded.
- `cx` plus a bundle directory: one small binary and a separate directory of
  `.conda` or `.tar.bz2` archives.

Use `cxz` when you want the transfer artifact to be a single file. Use a
bundle directory when an installer system, container build, or enterprise
software distribution process already manages package files separately.

conda-express publishes the `online` `cx` runtime and the `embedded` `cxz`
runtime. It does not currently publish conda-ship `external` layout release
assets such as `cx.bundle.tar.zst`. The `--bundle` option below is for
deployment systems that provide a bundle directory next to `cx`. For custom
external-bundle artifacts, use
{external+conda-ship:doc}`conda-ship's external artifact layout <explanation/artifact-layout-tradeoffs>`.

## Use cxz

Download the `cxz` file for the target platform from the release page, make it
executable on Unix, then bootstrap:

```bash
chmod +x cxz-x86_64-unknown-linux-gnu
./cxz-x86_64-unknown-linux-gnu --path /opt/cx bootstrap --offline
```

The embedded bundle is detected automatically, so you do not need `--bundle`.
Use `--offline` in disconnected environments so the runtime refuses network
access if anything is missing.

## Use a Bundle Directory

When package archives are stored separately by your installer, image build, or
software distribution process, point `cx` at the bundle directory and use
offline mode:

```bash
cx --path /opt/cx bootstrap \
  --bundle /path/to/packages \
  --offline
```

The same settings are available as environment variables for installer scripts
and CI jobs:

```bash
CX_BUNDLE=/path/to/packages CX_OFFLINE=1 cx --path /opt/cx bootstrap
```

## Use the installer script

The shell and PowerShell installers pass `CX_BUNDLE` and `CX_OFFLINE` through
as `cx bootstrap` options:

```bash
curl -fsSL https://jezdez.github.io/conda-express/get-cx.sh \
  | env CX_BUNDLE=/path/to/packages CX_OFFLINE=1 sh
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
binary or bundle directory across the boundary. See
{doc}`verify-release-artifacts` for the full checklist.
