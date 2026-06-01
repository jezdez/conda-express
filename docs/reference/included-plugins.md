# Included Plugins

`cx` bootstraps conda with a small set of default plugins. These are part of
the conda-express distribution policy, not conda-ship's generic builder
behavior.

## Default plugin set

| Package | What it adds | Typical commands or behavior |
|---|---|---|
| `conda-rattler-solver` | Rattler/resolvo-based solver backend | `solver: rattler` is written to `.condarc` |
| `conda-spawn` | Subprocess-based activation | `cx shell ENV`, `conda spawn ENV` |
| `conda-completion` | Shell completion support | `cx completion status`, `cx completion install --dry-run` |
| `conda-pypi` | PyPI interoperability inside conda workflows | PyPI dependency handling through the conda plugin stack |
| `conda-self` | Base environment self-management | `cx self update`, `conda self update` |
| `conda-global` | Isolated global tool environments | `cx global install ruff`, `cx global list` |
| `conda-workspaces` | Project workspaces, tasks, and lockfiles | `cx workspace ...`, `cx task ...` |

`cx` also installs Python and conda itself. The full package set is listed in
{doc}`../configuration`.

## Why these plugins are included

The default set is meant to make a fresh conda installation useful without
turning the base prefix into a project environment:

- `conda-rattler-solver` keeps the base small by avoiding libmamba's native
  dependency chain.
- `conda-spawn` gives users an activation workflow without `conda init`.
- `conda-self` gives the managed base prefix an intended update path.
- `conda-global` covers isolated command-line tools.
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

## Activation policy

Because `cx` includes `conda-spawn`, the conda-express activation model is:

```bash
cx shell myenv
exit
```

`cx activate`, `cx deactivate`, and `cx init` are disabled with guidance. See
{doc}`cli` for the exact command behavior.

## Workspace and tool commands

After bootstrap, these plugin commands are available through `cx`:

```bash
cx workspace init --name my-project
cx workspace add python numpy
cx workspace install
cx task run test
cx global install ruff
```

The commands are regular conda plugin subcommands. `cx` passes them through to
the installed conda executable after the prefix exists.
