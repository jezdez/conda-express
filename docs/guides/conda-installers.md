# Alternative to Conda Installers

[Anaconda Distribution](https://docs.conda.io/projects/conda/en/stable/user-guide/install/),
Miniconda, and Miniforge are the familiar, trusted ways many users get conda.
`cx` is not a demand to move away from them. It is an alternative bootstrap
path for users who want a smaller native executable, a locked first install,
and an opinionated base prefix.

## What stays familiar

After bootstrap, `cx` delegates ordinary commands to the installed `conda`
executable:

```bash
cx create -n analysis python=3.12 numpy pandas
cx install -n analysis scipy matplotlib
cx list -n analysis
cx env list
```

The packages still come from conda channels. The default channel is
`conda-forge`, and the installed prefix is a normal conda prefix on disk.

## What changes

The base prefix is managed more strictly than a typical Anaconda Distribution,
Miniconda, or Miniforge base:

- `cx` bootstraps into `~/.cx` by default.
- The base prefix is frozen after bootstrap, so day-to-day work should happen
  in named environments.
- Activation uses `cx shell ENV`, powered by conda-spawn, instead of
  `conda activate`.
- `conda init`, `conda activate`, and `conda deactivate` are intercepted with
  guidance because they do not match the `cx` activation model.

## Try cx side by side

You can try `cx` without changing an existing Anaconda Distribution,
Miniconda, or Miniforge installation:

```bash
cx bootstrap
cx create -n cx-test python=3.12
cx shell cx-test
python --version
exit
```

`cx` manages its own prefix. Do not point it at an existing Anaconda
Distribution, Miniconda, or Miniforge base prefix.

## Recreate an existing environment

If you have an `environment.yml`, pass the command through `cx`:

```bash
cx env create -f environment.yml
cx shell my-environment
```

If the file does not name the environment, choose one explicitly:

```bash
cx env create -n analysis -f environment.yml
cx shell analysis
```

## When to keep using an installer distribution

Keep using a traditional installer when that is the trusted path for your
organization, classroom, cluster image, or vendor workflow. `cx` is most useful
when you want the conda CLI with a smaller bootstrap artifact, fast first-run
setup, a frozen base prefix, or an offline `cxz` binary.
