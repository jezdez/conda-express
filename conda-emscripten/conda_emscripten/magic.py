"""IPython %cx line magic — conda-express in the browser."""

from __future__ import annotations

import logging
import sys

from .patches import patch_conda_internals, patch_urllib3

log = logging.getLogger(__name__)

_HELP = """\
%cx — conda-express in the browser (powered by cx-wasm)

Same commands as the cx CLI:

  %cx install <pkg> [pkg2 ...]   Install one or more packages
  %cx update  <pkg> [pkg2 ...]   Update packages
  %cx remove  <pkg> [pkg2 ...]   Remove packages
  %cx list                       List installed packages
  %cx search  <query>            Search available packages

Examples:
  %cx install zlib
  %cx install numpy scipy matplotlib
  %cx list
"""

_MUTATING = {"install", "update", "remove", "create"}
_COMMANDS = _MUTATING | {"list", "search"}


async def cx_magic(line: str) -> None:
    """IPython line magic: ``%cx install zlib``.

    Browser-side equivalent of the ``cx`` CLI.  Loads the cx-wasm bridge on
    first use; ``--yes`` is injected for mutating commands so conda never
    blocks on interactive prompts.
    """
    line = line.strip()
    if not line or line in ("-h", "--help", "help"):
        print(_HELP)
        return

    parts = line.split()
    command, args = parts[0], parts[1:]

    if command not in _COMMANDS:
        print(f"Unknown command: {command!r}\n")
        print(_HELP)
        return

    if command in _MUTATING and "--yes" not in args and "-y" not in args:
        args = ["--yes", *args]

    try:
        import cx_wasm_bridge

        if not cx_wasm_bridge.is_ready():
            print("Loading cx-wasm…")
            await cx_wasm_bridge.setup()
            patch_urllib3()
            patch_conda_internals()
    except ImportError:
        pass  # cx_wasm_bridge not installed (e.g. cx-worker.js context)

    try:
        from conda.cli.python_api import run_command
    except ImportError:
        print(
            "conda is not installed in this kernel.\nRebuild with: pixi run -e lite lite-build-local"
        )
        return

    stdout, stderr, rc = run_command(command, *args, use_exception_handler=True)
    if stdout:
        print(stdout, end="")
    if stderr:
        print(stderr, end="", file=sys.stderr)
    if rc != 0:
        print(f"\ncx {command} exited with code {rc}", file=sys.stderr)


def register(ip=None) -> None:
    """Register ``%cx`` with the active IPython instance, if any."""
    if ip is None:
        try:
            ip = get_ipython()  # type: ignore[name-defined]  # noqa: F821
        except NameError:
            return
    if ip is None:
        return
    ip.register_magic_function(cx_magic, magic_kind="line", magic_name="cx")
    log.debug("conda-emscripten: %%cx magic registered")
