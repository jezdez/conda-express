#!/usr/bin/env python3
"""Build the JupyterLite demo site.

Usage::

    # Standard build — packages from emscripten-forge only
    python build.py

    # Local build — adds the project's output/ channel and cx-wasm-kernel
    python build.py --with-local

The script writes environment.yml (overwriting it), then calls
``jupyter lite build``.  The generated file is .gitignored when it contains
the machine-specific local channel path.
"""

from __future__ import annotations

import argparse
import pathlib
import subprocess
import sys

HERE = pathlib.Path(__file__).parent.resolve()
ROOT = HERE.parent


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
        # Write to a gitignored file so environment.yml (the base) is never
        # overwritten with machine-specific file:// paths.
        env_yml = HERE / "_local_environment.yml"
        content = LOCAL_ENV_TEMPLATE.format(output=output)
        env_yml.write_text(content)
        print(f"[build.py] Wrote {env_yml} (local channel: file://{output})")
        extra_args = [f"--XeusAddon.environment_file={env_yml}"]
    else:
        env_yml = HERE / "environment.yml"
        env_yml.write_text(BASE_ENV)
        print(f"[build.py] Wrote {env_yml}")
        extra_args = []

    result = subprocess.run(
        ["jupyter", "lite", "build", "--config", "jupyter_lite_config.json", *extra_args],
        cwd=HERE,
    )
    sys.exit(result.returncode)


if __name__ == "__main__":
    main()
