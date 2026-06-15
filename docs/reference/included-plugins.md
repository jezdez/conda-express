# Included Plugins

`cx` bootstraps conda with a selected set of default plugins. These are part of
the conda-express distribution policy, not conda-ship's generic builder
behavior.

## Default plugin set

| Package | What it adds | Typical commands or behavior |
|---|---|---|
| `conda-rattler-solver` | Rattler/resolvo-based solver backend | `solver: rattler` is written to `.condarc` |
| `conda-spawn` | Subprocess-based activation | `cx shell ENV`, `conda spawn ENV` |
| `conda-completion` | Shell completion support | `cx completion status`, `cx completion install --dry-run` |
| `conda-exec` | Ephemeral package execution and PEP 723 scripts | `cx exec ruff --version`, `cx exec --list` |
| `conda-pypi` | PyPI interoperability inside conda workflows | PyPI dependency handling through the conda plugin stack |
| `conda-self` | Base environment self-management workflow | `cx self reset --snapshot installer-exact` |
| `conda-global` | Isolated global tool environments | `cx global install ruff`, `cx global list` |
| `conda-workspaces` | Project workspaces, tasks, and lockfiles | `cx workspace ...`, `cx task ...` |

`cx` also installs Python and conda itself. The full package set is listed in
{doc}`../configuration`.

## Why these plugins are included

The default set is meant to make a fresh conda installation useful without
turning the base prefix into a project environment:

- `conda-rattler-solver` avoids libmamba's native dependency chain in the
  managed base.
- `conda-spawn` gives users an activation workflow without `conda init`.
- `conda-self` can reset the managed base prefix from the initial-state
  snapshot written at bootstrap.
- `conda-global` covers isolated command-line tools.
- `conda-exec` covers one-off package execution without adding tools to the
  managed base prefix.
- `conda-workspaces` covers project-level environments and tasks for users who
  want a conda-native workspace workflow.
- `conda-pypi` and `conda-completion` fill common day-to-day gaps.

## Shell completion

The completion command is available after bootstrap:

```bash
cx completion status
cx completion init bash
cx completion install --dry-run
```

`init` prints the shell integration script. `install` writes the shell hook
that enables command completion for conda commands and installed conda plugin
subcommands. Use `--dry-run` first to inspect the profile changes before
writing them.

conda-ship sets the runtime command name for delegate environments, so the
completion hook can register `cx` rather than the underlying `conda`
executable.

## Activation Workflow

Because `cx` includes `conda-spawn`, the conda-express activation model is:

```bash
cx shell myenv
exit
```

`cx shell` is a runtime shortcut for `conda spawn`. Other conda commands are
passed through to the installed conda executable after bootstrap.

## Workspace and tool commands

After bootstrap, these plugin commands are available through `cx`:

```bash
cx workspace init --name my-project
cx workspace add python numpy
cx workspace install
cx task run test
cx exec ruff --version
cx global install ruff
```

The commands are regular conda plugin subcommands. `cx` passes them through to
the installed conda executable after the prefix exists.
