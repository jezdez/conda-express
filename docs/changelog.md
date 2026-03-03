# Changelog

## 0.1.0 (unreleased)

Initial release.

- Single-binary conda bootstrapper powered by rattler
- Compile-time lockfile (rattler-lock v6) for deterministic bootstraps
- Post-solve exclusion of `conda-libmamba-solver` and 27 native dependencies
- conda-rattler-solver as default solver
- conda-spawn activation model with `cx shell` alias
- Disabled `activate`, `deactivate`, and `init` commands
- Auto-bootstrap on first conda command
- CEP 22 frozen base prefix protection
- Multi-platform CI/CD (linux-x64, linux-aarch64, macos-x64, macos-arm64, windows-x64)
- PyPI distribution via maturin platform wheels
- crates.io distribution
- Trusted publishing (OIDC) for both PyPI and crates.io
