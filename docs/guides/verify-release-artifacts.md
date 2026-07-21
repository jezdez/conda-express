# Verify Release Artifacts

Use this guide when you download `cx` or `cxz` directly from GitHub Releases,
mirror the files into another system, or want to check an artifact before using
it in an installer, image, or air-gapped transfer.

Package managers add their own verification layer. Homebrew checks the formula
checksum, PyPI installs a platform wheel, and Docker pulls by tag or digest. The
steps below are most useful for direct GitHub Release downloads and downstream
packaging work.

For the generic conda-ship trust model, see
{external+conda-ship:doc}`conda-ship's trust and provenance notes <explanation/trust-and-provenance>`
and
{external+conda-ship:doc}`conda-ship's artifact verification guide <how-to/verify-release-artifacts>`.

## Download The Artifact Set

Download the runtime and its matching metadata from the same release. For a
Linux x86_64 `cx` binary, the files are:

```text
cx-x86_64-unknown-linux-gnu
cx-x86_64-unknown-linux-gnu.info.json
cx-x86_64-unknown-linux-gnu.packages.txt
cx-x86_64-unknown-linux-gnu.runtime.lock
cx-x86_64-unknown-linux-gnu.sha256
```

For `cxz`, use the same target with the `cxz-` prefix:

```text
cxz-x86_64-unknown-linux-gnu
cxz-x86_64-unknown-linux-gnu.info.json
cxz-x86_64-unknown-linux-gnu.packages.txt
cxz-x86_64-unknown-linux-gnu.runtime.lock
cxz-x86_64-unknown-linux-gnu.sha256
```

Windows binaries use `.exe` for the executable. Metadata files keep the same
stem and add their own suffix.

## Check SHA256 Sums

The `.sha256` file is written by conda-ship and covers the staged files for
that runtime output. Use it to catch transfer errors and confirm that the
downloaded files match the release checksum file.

On macOS:

```bash
shasum -a 256 --check cx-x86_64-unknown-linux-gnu.sha256
```

On Linux:

```bash
sha256sum --check cx-x86_64-unknown-linux-gnu.sha256
```

Run the command from the directory containing the runtime and its metadata.
If the checksum file contains paths with a directory prefix, run the command
from the matching parent directory.

## Verify GitHub Artifact Attestations

GitHub Release artifacts are attested by the conda-express release workflow.
Verify the downloaded runtime against that workflow identity:

```bash
gh attestation verify ./cx-x86_64-unknown-linux-gnu \
  -R jezdez/conda-express \
  --signer-workflow jezdez/conda-express/.github/workflows/release.yml
```

For `cxz`, verify the `cxz-*` binary:

```bash
gh attestation verify ./cxz-x86_64-unknown-linux-gnu \
  -R jezdez/conda-express \
  --signer-workflow jezdez/conda-express/.github/workflows/release.yml
```

:::{note}
Artifact attestations identify the workflow that produced the file. They do not
replace reviewing the package set, channels, release notes, or your own
organization's signing policy.
:::

Repeat the attestation check for any metadata file that a downstream process
trusts as release input, such as `.runtime.lock`, `.info.json`, `.packages.txt`,
or `.sha256`.

## Inspect Metadata

The `.info.json` file describes the runtime artifact:

```bash
python -m json.tool cx-x86_64-unknown-linux-gnu.info.json
```

Use it to confirm the layout, platform, runtime name, package count, and staged
checksums before wrapping the binary in another installer or image.

The `.packages.txt` file is a plain package list for quick review:

```bash
less cx-x86_64-unknown-linux-gnu.packages.txt
```

The `.runtime.lock` file is the lock that `cx` or `cxz` uses during bootstrap.
It should match the package set you expect for the release. Do not edit it by
hand; update the source environment and rebuild instead.

## Verify Before Air-Gapped Transfer

For disconnected systems, verify the files before moving them across the
boundary:

1. Download the runtime, metadata, and checksum files on a connected machine.
2. Verify the checksum and GitHub attestation.
3. Transfer the verified files into the disconnected environment.
4. Run `CX_OFFLINE=1 cxz info`, or run
   `CX_BUNDLE=/path/to/packages CX_OFFLINE=1 cx info` when a bundle directory
   is provided by your deployment process.

See {doc}`offline-and-airgapped` for offline install patterns.

## Verify Container Images

The release workflow publishes GHCR images with provenance and SBOM metadata.
For stronger pinning, pull by digest instead of a floating tag:

```bash
docker pull ghcr.io/jezdez/conda-express@sha256:DIGEST
```

When verifying GitHub attestations for container images, use the exact image
reference or digest you deploy and verify it against the conda-express
repository. Keep container admission, vulnerability scanning, and any
organization-specific signing policy as separate downstream controls.
