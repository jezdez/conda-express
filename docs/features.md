# Features

## Single-binary bootstrapper

cx is a single ~10 MB static binary written in Rust. It requires no Python,
no installer framework, and no shell modifications. Download it, run it, and
you have a working conda installation.

## Compile-time lockfile

`build.rs` performs a full dependency solve at `cargo build` time using rattler
crates, producing a [rattler-lock v6](https://github.com/conda/rattler/tree/main/crates/rattler_lock)
lockfile that is embedded into the binary. At runtime, bootstrap skips repodata
fetching and solving entirely — it downloads and installs packages directly from
the locked URLs.

This gives cx deterministic, reproducible bootstraps with ~3–5 second install
times.

## Package exclusion

conda on conda-forge hard-depends on `conda-libmamba-solver`, which pulls in
27 native dependencies (libsolv, libarchive, libcurl, spdlog, etc.). Since cx
uses `conda-rattler-solver` instead, these are unnecessary.

cx removes them via a post-solve transitive dependency pruning algorithm:
after the solver produces a complete solution, cx identifies packages that are
*exclusively* required by the excluded packages and removes them. This reduces
the install from 113 to 86 packages.

## conda-rattler-solver

cx configures [conda-rattler-solver](https://github.com/jaimergp/conda-rattler-solver)
as the default solver via `.condarc`. This solver is based on
[resolvo](https://github.com/mamba-org/resolvo), the fastest SAT solver in the
conda ecosystem, and ships as a pure Python package with
[py-rattler](https://pypi.org/project/py-rattler/) wheels.

## conda-spawn activation

cx ships with [conda-spawn](https://github.com/conda-incubator/conda-spawn)
and disables traditional `conda activate`/`deactivate`/`init`. Instead:

```bash
cx shell myenv          # spawns a subshell with myenv activated
exit                    # leaves the environment
```

No `.bashrc`/`.zshrc` modifications required. Just add `~/.cx/condabin` to
your `PATH`.

## `cx shell` alias

`cx shell` is a convenience alias for `conda spawn`. It works identically:

```bash
cx shell myenv          # same as: conda spawn myenv
```

## Frozen base prefix (CEP 22)

After bootstrap, cx writes a `conda-meta/frozen` marker file per
[CEP 22](https://conda.org/learn/ceps/cep-0022/). This protects the base
prefix from accidental modification. Users should create named environments
for their work:

```bash
cx create -n myenv numpy pandas
cx shell myenv
```

## Auto-bootstrap

If the prefix doesn't exist when you run a conda command, cx automatically
bootstraps before executing:

```bash
# First time: bootstraps ~/.cx, then creates the environment
cx create -n myenv python=3.12
```

## External lockfile support

For custom deployments, you can override the embedded lockfile:

```bash
cx bootstrap --lockfile /path/to/custom.lock
```

Or skip the lockfile entirely for a live solve:

```bash
cx bootstrap --no-lock
```

## Multi-platform support

cx builds and tests on 6 platforms via GitHub Actions:

| Platform | Runner |
|---|---|
| linux-x64 | `ubuntu-latest` |
| linux-aarch64 | `ubuntu-24.04-arm` |
| macos-x64 | `macos-15-large` |
| macos-arm64 | `macos-latest` |
| windows-x64 | `windows-latest` |
| windows-arm64 | `windows-11-arm` |

## PyPI and crates.io distribution

cx is published as `conda-express` on both
[PyPI](https://pypi.org/project/conda-express/) and
[crates.io](https://crates.io/crates/conda-express):

```bash
pip install conda-express       # from PyPI
cargo install conda-express     # from crates.io
```

Both use trusted publishing (OIDC) for secure, tokenless releases.
