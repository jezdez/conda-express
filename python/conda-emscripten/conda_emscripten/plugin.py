"""Conda plugin registration for Emscripten: solver, package extractor, virtual packages."""

import sys

from conda import plugins
from conda.base.context import context
from conda.plugins.types import CondaSolver, CondaVirtualPackage

from .solver import WasmSolver


@plugins.hookimpl
def conda_solvers():
    yield CondaSolver(
        name="emscripten",
        backend=WasmSolver,
    )


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
def conda_virtual_packages():
    if not context.subdir.startswith("emscripten-"):
        return

    yield CondaVirtualPackage(
        name="unix",
        version=None,
        build=None,
    )
    yield CondaVirtualPackage(
        name="emscripten",
        version="3.1.58",
        build=None,
    )
