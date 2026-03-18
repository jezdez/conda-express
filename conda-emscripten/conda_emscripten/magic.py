from __future__ import annotations

import logging
import os
import sys

log = logging.getLogger(__name__)

_patches_applied = False
_run_command = None

_HELP = """\
%cx / %conda — real conda in the browser (accelerated by cx-wasm)

Calls conda.cli directly — all conda subcommands are supported:

  %cx install <pkg> [pkg2 ...]   Install packages
  %cx remove  <pkg> [pkg2 ...]   Remove packages
  %cx list                       List installed packages
  %cx search  <query>            Search available packages
  %cx info                       Show conda info
  %cx env list                   List environments

Examples:
  %cx install zlib
  %conda install numpy scipy matplotlib
  %cx list
"""

_MUTATING = {"install", "update", "upgrade", "remove", "uninstall", "create"}

_CONDARC = """\
solver: cx-wasm
subdir: emscripten-wasm32
auto_activate_base: false
notify_outdated_conda: false
show_channel_urls: true
channels:
  - https://repo.prefix.dev/emscripten-forge-4x
  - conda-forge
"""


def _is_shared_lib(filename: str) -> bool:
    """True for ``.so``, ``.so.1``, ``.so.1.10.0``, etc."""
    parts = filename.split(".")
    return "so" in parts


def _find_shared_libs(prefix: str) -> set[str]:
    """Walk *prefix* and return all shared library paths.

    Matches both bare ``.so`` extensions and versioned variants like
    ``liblz4.so.1.10.0`` — emscripten-forge packages use both.
    """
    libs: set[str] = set()
    for dirpath, _dirnames, filenames in os.walk(prefix):
        for fn in filenames:
            if _is_shared_lib(fn):
                libs.add(os.path.join(dirpath, fn))
    return libs


def _load_shared_lib(path: str) -> None:
    """Load a single WASM shared library into Emscripten's dynamic linker.

    Uses ``ctypes.CDLL`` with ``RTLD_GLOBAL`` which calls Emscripten's
    ``dlopen`` → ``loadDynamicLibrary`` under the hood.  This is more
    reliable than calling ``Module.loadDynamicLibrary`` via JS because
    ``Module`` may not be on ``globalThis`` in the xeus-python worker.
    """
    import ctypes  # noqa: PLC0415

    ctypes.CDLL(path, mode=ctypes.RTLD_GLOBAL)


def _load_new_shared_libs(before: set[str], prefix: str) -> None:
    """Register newly installed shared libraries with Emscripten's dynamic linker.

    Compares the current set of shared libraries under *prefix* against the
    *before* snapshot taken prior to the conda command.  New libraries are
    loaded shallowest-first (C runtime libs before Python extension modules)
    with retry to handle inter-library dependencies.
    """
    after = _find_shared_libs(prefix)
    new_libs = sorted(after - before, key=lambda p: p.count(os.sep))

    if not new_libs:
        return

    log.info("conda-emscripten: %d new shared libraries to load", len(new_libs))

    failed: list[tuple[str, Exception]] = []

    for so_path in new_libs:
        try:
            _load_shared_lib(so_path)
            log.debug("conda-emscripten: loaded %s", so_path)
        except Exception as exc:  # noqa: BLE001
            failed.append((so_path, exc))

    # Retry once — a library's dependencies may have been loaded in the first pass.
    still_failed: list[tuple[str, Exception]] = []
    for so_path, _prev_exc in failed:
        try:
            _load_shared_lib(so_path)
            log.debug("conda-emscripten: loaded %s (retry)", so_path)
        except Exception as exc:  # noqa: BLE001
            still_failed.append((so_path, exc))
            log.warning("conda-emscripten: failed to load %s: %s", so_path, exc)

    loaded = len(new_libs) - len(still_failed)
    log.info("conda-emscripten: loaded %d/%d shared libraries", loaded, len(new_libs))


def _bootstrap_prefix():
    """One-time setup of the conda prefix in MEMFS.

    Mirrors what ``cx bootstrap`` does natively: creates conda-meta/
    with a history file and writes a .condarc that configures channels
    and the cx-wasm solver. Also sets env vars since conda's platform
    detection and config search don't handle emscripten.
    """
    prefix = sys.prefix
    conda_meta = os.path.join(prefix, "conda-meta")
    os.makedirs(conda_meta, exist_ok=True)

    history = os.path.join(conda_meta, "history")
    if not os.path.exists(history):
        with open(history, "w") as f:
            f.write("")

    condarc = os.path.join(prefix, ".condarc")
    if not os.path.exists(condarc):
        with open(condarc, "w") as f:
            f.write(_CONDARC)

    os.environ.setdefault("CONDA_ROOT_PREFIX", prefix)
    os.environ.setdefault("CONDA_PREFIX", prefix)
    os.environ.setdefault("CONDARC", condarc)
    if sys.platform == "emscripten":
        os.environ.setdefault("CONDA_SUBDIR", "emscripten-wasm32")


def cx_magic(line: str) -> None:
    """IPython line magic: ``%cx install zlib`` or ``%conda install zlib``."""
    if not (line := line.strip()) or line in ("-h", "--help", "help"):
        print(_HELP)
        return

    command, *args = line.split()

    if command in _MUTATING and "--yes" not in args and "-y" not in args:
        args = ["--yes", *args]

    global _patches_applied, _run_command

    try:
        import cx_wasm_bridge

        if not cx_wasm_bridge.is_ready():
            print("cx-wasm is still loading — please run the cell again in a moment")
            return
    except ImportError:
        pass

    if not _patches_applied:
        _bootstrap_prefix()

        from .patches import patch_conda_internals, patch_urllib3

        patch_urllib3()
        patch_conda_internals()
        _patches_applied = True

    if _run_command is None:
        try:
            from conda.cli.main import main

            _run_command = main
        except ImportError:
            print(
                "conda is not installed in this kernel.\n"
                "Rebuild with: pixi run -e lite lite-build-local"
            )
            return

    is_mutating = command in _MUTATING
    prefix = sys.prefix
    before_libs = _find_shared_libs(prefix) if is_mutating else None

    _run_command(command, *args)

    if is_mutating and before_libs is not None:
        _load_new_shared_libs(before_libs, prefix)


def register(ip=None) -> None:
    """Register ``%cx`` and ``%conda`` with the active IPython instance."""
    if ip is None:
        try:
            ip = get_ipython()  # type: ignore[name-defined]  # noqa: F821
        except NameError:
            return
    if ip is None:
        return
    ip.register_magic_function(cx_magic, magic_kind="line", magic_name="cx")
    ip.register_magic_function(cx_magic, magic_kind="line", magic_name="conda")
    log.debug("conda-emscripten: %%cx and %%conda magics registered")
