# Use cx in GitHub Actions

Use the `setup-cx` action when a workflow needs a small conda bootstrapper
without installing Miniconda, Miniforge, or a larger distribution first.

The default action path downloads the `cx` release asset, verifies the
published SHA256 checksum, adds `cx` to `PATH`, and runs `cx bootstrap`.

## Add cx to a workflow

Pin the action to a conda-express release tag:

```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: {{ setup_cx_action }}
      - run: cx status
```

When the action ref is a conda-express release tag and `version` is omitted,
the action installs that same `cx` release. This keeps the action code and the
downloaded binary aligned.

After bootstrap, use `cx` the same way you would use conda:

```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: {{ setup_cx_action }}
      - run: cx create -n test python=3.12 pytest
      - run: cx run -n test pytest
```

## Install without bootstrapping

Set `bootstrap: false` when a job only needs to inspect the binary or wants to
bootstrap later with custom options:

```yaml
jobs:
  inspect:
    runs-on: ubuntu-latest
    steps:
      - id: setup-cx
        uses: {{ setup_cx_action }}
        with:
          bootstrap: false
      - run: "${{ steps.setup-cx.outputs.cx-path }}" --version
```

## Install a different cx version

Pass `version` when the action ref and binary version should differ:

```yaml
steps:
  - uses: {{ setup_cx_action }}
    with:
      version: {{ conda_express_release }}
```

This is mainly useful for testing the action from a branch or local checkout.
For normal workflows, prefer pinning the action to the release tag you want to
run.

## Verify artifact attestations

Checksum verification is enabled by default. For stricter provenance checks,
enable GitHub Artifact Attestation verification and grant the job the
additional read permission:

```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      attestations: read
    steps:
      - uses: {{ setup_cx_action }}
        with:
          verify-attestation: true
          github-token: ${{ github.token }}
      - run: cx status
```

`verify-attestation: true` fails closed when `github-token` is missing. The
token is passed to `gh attestation verify`; it is not needed for the default
checksum-only path.

## Use a custom install directory

By default the action installs `cx` into the runner temp directory. Use
`install-dir` if later workflow steps expect a stable location:

```yaml
steps:
  - uses: {{ setup_cx_action }}
    with:
      install-dir: ${{ runner.temp }}/tools
```

The install directory is added to `PATH` unless `add-to-path: false` is set.

See {doc}`../reference/installer` for the complete action input and output
reference.
