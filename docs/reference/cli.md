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

`--payload DIR`
: Directory containing pre-downloaded `.conda` and/or `.tar.bz2` package archives.
  When set, cx pre-populates the package cache from this directory before installing.
  Paired with `--offline`, this enables fully air-gapped bootstrap from a bundled
  payload. Can also be set via the `CX_PAYLOAD` environment variable.

`--offline`
: Disable all network access during bootstrap. Packages must already be available in
  the local package cache (from a previous bootstrap) or supplied via `--payload`.
  Incompatible with `--no-lock` (offline mode requires a lockfile). Can also be set
  via the `CX_OFFLINE` environment variable.

:::{note}
When the binary was built with `CX_EMBED_PAYLOAD=1` (i.e. `cxz`), the embedded
package payload is detected automatically. `--payload` and `--offline` are
implied — no flags or environment variables are needed.
:::

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

# Offline bootstrap from cache (after a previous online bootstrap)
cx bootstrap --prefix /opt/conda --offline

# Offline bootstrap from a payload directory
cx bootstrap --payload ./packages/ --offline

# Bootstrap with cxz (embedded payload, auto-detected)
cxz bootstrap
```

---

## `cx status`

Print cx installation status (prefix, channels, packages, excludes).
For conda's own info, use `cx info` which passes through to `conda info`.

```
cx status [OPTIONS]
```

### Options

`--prefix DIR`
: Target prefix directory. Default: `~/.cx`

:::{admonition} Example output
:class: dropdown

```
cx 0.1.0
  prefix:   /Users/you/.cx
  channels: conda-forge
  packages: python >=3.12, conda >=25.1, ...
  excludes: conda-libmamba-solver
  installed: 86 packages
  conda:    /Users/you/.cx/bin/conda
```
:::

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

(cli-cx-uninstall)=
## `cx uninstall`

Completely remove cx: the conda prefix (including all named environments),
the cx binary itself, and PATH entries from shell profiles.

```
cx uninstall [OPTIONS]
```

### Options

`--prefix DIR`
: Target prefix directory. Default: `~/.cx`

`-y, --yes`
: Skip the interactive confirmation prompt.

### What gets removed

1. The conda prefix directory (e.g. `~/.cx`) including all named environments
2. The `cx` binary (detected via the running executable path)
3. PATH entries added by the installer from `~/.bashrc`, `~/.zshrc`, and
   `~/.config/fish/config.fish`

### Examples

```bash
# Interactive uninstall (shows what will be removed, asks for confirmation)
cx uninstall

# Non-interactive uninstall
cx uninstall --yes

# Uninstall a non-default prefix
cx uninstall --prefix /opt/conda
```

:::{admonition} Example output
:class: dropdown

```
! This will permanently remove:
   Conda prefix: /home/user/.cx
   Named environments (2): myenv, data-science
   cx binary: /home/user/.local/bin/cx

   Continue? [y/N] y

>> Removing conda prefix at /home/user/.cx
>> Removing cx binary at /home/user/.local/bin/cx
>> Cleaned PATH entry from /home/user/.zshrc

✔ cx has been uninstalled.
```
:::

---

## Disabled commands

The following conda commands are intentionally disabled in cx because they
conflict with the conda-spawn activation model:

:::{admonition} `cx activate` / `cx deactivate`
:class: dropdown

Prints a message directing users to `cx shell` instead:

```
! `conda activate` is not available in cx.

  cx uses conda-spawn for environment activation.
  Instead of `conda activate myenv`, run:

    cx shell myenv

  To leave the environment, exit the subshell (Ctrl+D or `exit`).
```
:::

:::{admonition} `cx init`
:class: dropdown

Prints a message explaining that shell profile modifications are unnecessary:

```
! `conda init` is not needed with cx.

  cx uses conda-spawn, which does not require shell
  profile modifications. Just add condabin to your PATH:

    export PATH="$HOME/.cx/condabin:$PATH"

  Then activate environments with:

    cx shell myenv
```
:::

---

## `cx help`

Show a getting-started guide with cx-specific commands, common workflows,
and links to documentation.

```
cx help
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
