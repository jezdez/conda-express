# CLI Reference

`cx` is a conda runtime produced by conda-ship. It installs the conda-express
base environment from the stamped runtime lock when needed, then delegates to
the installed `conda` executable with the original arguments.

The bootstrapper does not reserve a separate command namespace. `--help`,
`--version`, and every subcommand are conda arguments. For the generic runtime
behavior shared by other generated runtimes, see
{external+conda-ship:doc}`conda-ship's runtime CLI reference <reference/runtime-cli>`.

## First Invocation

Run any conda command:

```bash
cx info
```

If `~/.conda/express` does not exist, `cx` installs the stamped package set and
then runs `conda info`. Later invocations delegate directly to the conda
executable in that prefix.

A bare invocation, `cx --help`, and `cx --version` follow the same rule. They
bootstrap when needed, then show conda's normal output.

The embedded `cxz` artifact has the same interface. It reads its package
archives from the binary during automatic bootstrap:

```bash
cxz info
```

## Bootstrap Controls

Automatic bootstrap is controlled with environment variables. These values
do not consume or rewrite conda arguments.

`CX_PREFIX`
: Override the managed prefix path. The default is `~/.conda/express`.

`CX_BUNDLE`
: Read package archives from a local bundle directory while bootstrapping.

`CX_OFFLINE`
: Disable network access while bootstrapping. Empty, `0`, and `false` leave
  offline mode disabled. Other non-empty values enable it.

Example:

```bash
CX_PREFIX=/opt/cx \
CX_BUNDLE=/opt/cx-packages \
CX_OFFLINE=1 \
cx info
```

Keep `CX_PREFIX` set on later invocations that should use the same non-default
prefix:

```bash
CX_PREFIX=/opt/cx cx create -n myenv python=3.12
```

`cxz` detects its embedded bundle automatically, so it does not need
`CX_BUNDLE`:

```bash
CX_PREFIX=/opt/cx CX_OFFLINE=1 cxz info
```

## Conda Commands

Use `cx` as the conda command name:

```bash
cx create -n myenv python=3.12
cx install -n myenv numpy
cx list -n myenv
cx env list
cx config --show
cx info
cx --help
cx --version
```

`cx info` is the status command. It reports the conda version, root prefix,
environment directories, channels, and other conda configuration.

## Included Plugin Commands

The installed conda environment supplies additional commands through the
plugins included by conda-express.

### `cx spawn`

Open a subshell with an environment activated through conda-spawn:

```bash
cx spawn myenv
exit
```

The currently released conda-spawn command is `spawn`. The `shell` alias in
[conda-spawn PR #59](https://github.com/conda/conda-spawn/pull/59) is not part
of a released conda-spawn version yet.

### `cx self`

conda-self manages the installed base environment. conda-ship writes
`conda-meta/initial-state.explicit.txt` during the first bootstrap, which lets
conda-self restore that installer snapshot:

```bash
cx self reset --snapshot installer-exact
```

That snapshot records the package set used to create the existing prefix. If
you later install a newer `cx` binary, reset still uses the snapshot already in
the prefix. It does not read or apply the newer binary's stamped runtime lock.

The current conda-self integration manages the installed prefix, not the `cx`
binary. Update or remove that binary through the method that installed it.

To bootstrap the package set from a newly installed binary, first export any
named environments you need, remove the managed prefix, and run any `cx`
command. The next invocation creates a new prefix from that binary's lock.

### `cx completion`

conda-completion can generate and install completion hooks. Pass
`--command-name cx` to commands that register a command name because the
runtime does not inject one into the conda process:

```bash
cx completion status
cx completion init bash --command-name cx
cx completion install --dry-run --command-name cx
```

## Removal

There is no runtime-owned `cx uninstall` command. Remove the binary through
the method that installed it, such as Homebrew or PyPI. Standalone script
installations are removed by deleting the installed binary and the PATH entry
added by the script.

Removing the binary does not remove `~/.conda/express` or its named
environments. Export anything you need before deleting that prefix manually.
This remains installation-method-specific until conda-self provides a
conda-express adapter.
