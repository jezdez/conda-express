"""Conda plugin hooks for Emscripten: solver, extractor, virtual packages, %cx magic."""

from __future__ import annotations

import sys

from conda import plugins
from conda.base.context import context
from conda.plugins.types import CondaPreCommand, CondaSolver, CondaVirtualPackage

from .solver import WasmSolver

_pre_command_init_done = False


def _on_pre_command(_command: str) -> None:
    """Warm up the WASM bridge before the first conda command runs."""
    global _pre_command_init_done

    if sys.platform != "emscripten" or _pre_command_init_done:
        return

    from .magic import register
    from .patches import patch_conda_internals, patch_urllib3

    patch_urllib3()
    patch_conda_internals()
    register()

    try:
        import cx_wasm_bridge  # noqa: F401  — import triggers background load
    except ImportError:
        pass

    _pre_command_init_done = True


def _emscripten_version() -> str | None:
    info = getattr(sys, "_emscripten_info", None)
    if info is None:
        return None
    major, minor, tiny = info.emscripten_version
    return f"{major}.{minor}.{tiny}"


@plugins.hookimpl
def conda_solvers():
    yield CondaSolver(name="emscripten", backend=WasmSolver)


@plugins.hookimpl
def conda_package_extractors():
    if sys.platform == "emscripten":
        from .extractor import extract_wasm

        yield plugins.types.CondaPackageExtractor(
            name="wasm-extractor",
            extensions=[".tar.bz2", ".conda"],
            extract=extract_wasm,
        )


@plugins.hookimpl
def conda_pre_commands():
    yield CondaPreCommand(
        name="cx-wasm-bridge-preload",
        action=_on_pre_command,
        run_for={"install", "update", "create", "remove"},
    )


@plugins.hookimpl
def conda_virtual_packages():
    if not context.subdir.startswith("emscripten-"):
        return
    yield CondaVirtualPackage(name="unix", version=None, build=None)
    yield CondaVirtualPackage(
        name="emscripten", version=_emscripten_version(), build=None
    )
