# CLI reference

## `cx bootstrap`

Bootstrap a fresh conda installation into the prefix.

```
cx bootstrap [OPTIONS]
```

### Options

`--force`
: Re-bootstrap even if the prefix already exists. Removes the existing prefix first.

`--prefix DIR`
: Target prefix directory. Default: `~/.cx`

`-c, --channel CH`
: Channels to use. Can be specified multiple times. Default: `conda-forge`

`-p, --package PKG`
: Additional packages to install beyond the built-in list. Can be specified multiple times.

`-e, --exclude PKG`
: Packages to exclude from installation (along with their exclusive dependencies). Can be specified multiple times. Default: `conda-libmamba-solver`

`--no-exclude`
: Disable all default exclusions. Installs everything including `conda-libmamba-solver`.

`--no-lock`
: Ignore the embedded lockfile and perform a live solve instead. Requires network access for repodata fetching.

`--lockfile PATH`
: Use an external lockfile instead of the embedded one. The file must be in rattler-lock v6 format.

### Examples

```bash
# Standard bootstrap
cx bootstrap

# Force re-bootstrap with latest packages (live solve)
cx bootstrap --force --no-lock

# Bootstrap with additional packages
cx bootstrap --package conda-build --package rattler-build

# Bootstrap into a custom location
cx bootstrap --prefix /opt/conda
```

---

## `cx info`

Print information about the current installation.

```
cx info [OPTIONS]
```

### Options

`--prefix DIR`
: Target prefix directory. Default: `~/.cx`

### Output

```
cx 0.1.0
  prefix:   /Users/you/.cx
  channels: conda-forge
  packages: python >=3.12, conda >=25.1, ...
  excludes: conda-libmamba-solver
  installed: 86 packages
  conda:    /Users/you/.cx/bin/conda
```

---

## `cx shell`

Activate an environment by spawning a new subshell. This is an alias for
`conda spawn`.

```
cx shell [ENV] [OPTIONS]
```

### Arguments

`ENV`
: Name of the environment to activate.

### Examples

```bash
# Activate an environment
cx shell myenv

# Leave the environment
exit    # or Ctrl+D
```

---

## Disabled commands

The following conda commands are intentionally disabled in cx because they
conflict with the conda-spawn activation model:

### `cx activate` / `cx deactivate`

Prints a message directing users to `cx shell` instead:

```
! `conda activate` is not available in cx.

  cx uses conda-spawn for environment activation.
  Instead of `conda activate myenv`, run:

    cx shell myenv

  To leave the environment, exit the subshell (Ctrl+D or `exit`).
```

### `cx init`

Prints a message explaining that shell profile modifications are unnecessary:

```
! `conda init` is not needed with cx.

  cx uses conda-spawn, which does not require shell
  profile modifications. Just add condabin to your PATH:

    export PATH="$HOME/.cx/condabin:$PATH"

  Then activate environments with:

    cx shell myenv
```

---

## Pass-through commands

Any command not listed above is passed through to the installed `conda` binary.
cx replaces its own process with `conda` using `execvp` (Unix) or
`CreateProcess` (Windows), so there is no overhead:

```bash
cx create -n myenv python=3.12
cx install -n myenv numpy
cx list -n myenv
cx env list
cx config --show
cx self update          # handled by conda-self
```
