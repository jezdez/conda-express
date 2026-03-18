#!/usr/bin/env python3
"""Build the JupyterLite demo site.

Usage::

    # Standard build — packages from emscripten-forge only
    python build.py

    # Local build — adds the project's output/ channel and cx-wasm-kernel
    python build.py --with-local

The script creates the conda environment directly (keeping .pyc bytecode
for faster browser startup), then calls ``jupyter lite build`` with
``--XeusAddon.prefix`` pointing to the pre-built environment.
"""

from __future__ import annotations

import argparse
import pathlib
import shutil
import subprocess
import sys

HERE = pathlib.Path(__file__).parent.resolve()
ROOT = HERE.parent

ENV_NAME = "xeus-python-kernel"

BASE_CHANNELS = [
    "https://repo.prefix.dev/emscripten-forge-4x",
    "https://conda.anaconda.org/conda-forge",
]

BASE_SPECS = ["xeus-python", "numpy", "matplotlib", "ipywidgets"]

LOCAL_EXTRA_SPECS = ["cx-wasm-kernel", "conda", "conda-emscripten"]


# These templates are only written for documentation/reference purposes.
BASE_ENV = """\
name: xeus-python-kernel
channels:
  - https://repo.prefix.dev/emscripten-forge-4x
  - https://conda.anaconda.org/conda-forge
dependencies:
  - xeus-python
  - numpy
  - matplotlib
  - ipywidgets
"""

LOCAL_ENV_TEMPLATE = """\
name: xeus-python-kernel
channels:
  - file://{output}
  - https://repo.prefix.dev/emscripten-forge-4x
  - https://conda.anaconda.org/conda-forge
dependencies:
  - xeus-python
  - numpy
  - matplotlib
  - ipywidgets
  # cx-wasm WASM bridge (loads cx_wasm_bg.wasm into xeus-python kernel)
  - cx-wasm-kernel
  # patched conda + solver plugin (enables conda install from kernel)
  - conda
  - conda-emscripten
"""

PLATFORM = "emscripten-wasm32"


def _create_prefix(channels: list[str], specs: list[str]) -> pathlib.Path:
    """Create the emscripten conda environment, keeping .pyc bytecode.

    Unlike jupyterlite-xeus's default, this does NOT pass ``--no-pyc``
    to micromamba, so pre-compiled bytecode from conda packages is
    preserved.  Combined with the custom empack config that allows
    ``*.pyc`` files, this avoids expensive runtime compilation of
    every Python module on first import in the browser.
    """
    root_prefix = HERE / "_env"
    prefix_path = root_prefix / "envs" / ENV_NAME

    if prefix_path.exists():
        shutil.rmtree(prefix_path)

    root_prefix.mkdir(parents=True, exist_ok=True)

    micromamba = shutil.which("micromamba")
    if not micromamba:
        raise RuntimeError(
            "micromamba is needed for creating the emscripten environment.\n"
            "Install it with: conda install micromamba -c conda-forge"
        )

    channels_args = []
    for ch in channels:
        channels_args.extend(["-c", ch])

    cmd = [
        micromamba,
        "create",
        "--yes",
        "--prefix",
        str(prefix_path),
        "--relocate-prefix",
        "",
        "--root-prefix",
        str(root_prefix),
        f"--platform={PLATFORM}",
        *channels_args,
        *specs,
    ]

    print(f"[build.py] Creating environment at {prefix_path}")
    subprocess.run(cmd, check=True)
    print(f"[build.py] Environment created ({prefix_path})")

    return prefix_path


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--with-local",
        action="store_true",
        help="Include locally-built conda packages (cx-wasm-kernel) from output/",
    )
    args = parser.parse_args()

    output = ROOT / "output"

    if args.with_local:
        if not output.exists():
            print(f"ERROR: {output} not found.")
            print("Build the local packages first:")
            print("  pixi run -e web wasm-build")
            print("  pixi run -e recipes build-cx-wasm-kernel")
            print("  pixi run -e recipes build-conda-emscripten-plugin")
            sys.exit(1)
        channels = [f"file://{output}"] + BASE_CHANNELS
        specs = BASE_SPECS + LOCAL_EXTRA_SPECS

        # Write reference YAML (gitignored — contains machine-specific path)
        env_yml = HERE / "_local_environment.yml"
        env_yml.write_text(LOCAL_ENV_TEMPLATE.format(output=output))
        print(f"[build.py] Wrote {env_yml} (local channel: file://{output})")
    else:
        channels = list(BASE_CHANNELS)
        specs = list(BASE_SPECS)

        env_yml = HERE / "environment.yml"
        env_yml.write_text(BASE_ENV)
        print(f"[build.py] Wrote {env_yml}")

    prefix_path = _create_prefix(channels, specs)

    cx_jl = ROOT / "cx-jupyterlite"
    if (cx_jl / "package.json").exists():
        print("[build.py] Building cx-jupyterlite extension …")
        subprocess.run(["jlpm", "install"], cwd=cx_jl, check=True)
        subprocess.run(["jlpm", "run", "build"], cwd=cx_jl, check=True)
        print("[build.py] cx-jupyterlite extension built")

    result = subprocess.run(
        [
            "jupyter",
            "lite",
            "build",
            "--config",
            "jupyter_lite_config.json",
            f"--XeusAddon.prefix={prefix_path}",
        ],
        cwd=HERE,
    )
    sys.exit(result.returncode)


if __name__ == "__main__":
    main()
