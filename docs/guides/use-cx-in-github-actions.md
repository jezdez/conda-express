# Use cx in GitHub Actions

Use the `setup-cx` action when a workflow needs a small conda bootstrapper
without installing Miniconda, Miniforge, or a larger distribution first.

The default action path downloads the `cx` release asset, verifies the
published SHA256 checksum, adds `cx` to `PATH`, and runs `cx info`. That first
conda command automatically bootstraps the managed prefix.

## Add cx to a workflow

Pin the action to a conda-express release tag:

```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: {{ setup_cx_action }}
      - run: cx info
```

When the action ref is a conda-express release tag and `version` is omitted,
the action installs that same `cx` release. This keeps the action code and the
downloaded binary aligned.

After automatic bootstrap, use `cx` the same way you would use conda:

```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: {{ setup_cx_action }}
      - run: cx create -n test python=3.12 pytest
      - run: cx run -n test pytest
```

## Install without eager bootstrap

Set `bootstrap: false` when a job only needs to inspect the binary or wants to
defer automatic bootstrap:

```yaml
jobs:
  inspect:
    runs-on: ubuntu-latest
    steps:
      - id: setup-cx
        uses: {{ setup_cx_action }}
        with:
          bootstrap: false
      - run: test -x "${{ steps.setup-cx.outputs.cx-path }}"
```

Any later `cx` command bootstraps the prefix before conda handles its
arguments. Set `CX_PREFIX`, `CX_BUNDLE`, or `CX_OFFLINE` on that command when
the job needs non-default bootstrap controls.

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
      - run: cx info
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
