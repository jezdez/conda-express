from __future__ import annotations

import logging
import sys

log = logging.getLogger(__name__)

_patches_applied = False
_run_command = None

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
    """IPython line magic: ``%cx install zlib``."""
    if not (line := line.strip()) or line in ("-h", "--help", "help"):
        print(_HELP)
        return

    command, *args = line.split()

    if command not in _COMMANDS:
        print(f"Unknown command: {command!r}\n")
        print(_HELP)
        return

    if command in _MUTATING and "--yes" not in args and "-y" not in args:
        args = ["--yes", *args]

    global _patches_applied, _run_command

    try:
        import cx_wasm_bridge

        if not cx_wasm_bridge.is_ready():
            print("Loading cx-wasm…")
            await cx_wasm_bridge.setup()
    except ImportError:
        pass

    if not _patches_applied:
        from .patches import patch_conda_internals, patch_urllib3

        patch_urllib3()
        patch_conda_internals()
        _patches_applied = True

    if _run_command is None:
        try:
            from conda.cli.python_api import run_command

            _run_command = run_command
        except ImportError:
            print(
                "conda is not installed in this kernel.\n"
                "Rebuild with: pixi run -e lite lite-build-local"
            )
            return

    stdout, stderr, rc = _run_command(command, *args, use_exception_handler=True)
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
