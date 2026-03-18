"""conda-emscripten: Emscripten-specific conda plugins (solver, extractor, virtual packages)."""

__version__ = "0.1.0"

import logging
import sys
import warnings

if sys.platform == "emscripten":
    # Emscripten can't start threads; silence tqdm's monitor warning.
    warnings.filterwarnings(
        "ignore", message="tqdm:disabling monitor", category=FutureWarning
    )
    warnings.filterwarnings(
        "ignore", message="tqdm:disabling monitor", category=UserWarning
    )
    try:
        from tqdm import TqdmMonitorWarning

        warnings.filterwarnings("ignore", category=TqdmMonitorWarning)
    except ImportError:
        pass

    # urllib3 DEBUG-level connection logs are noisy in the browser console.
    logging.getLogger("urllib3").setLevel(logging.WARNING)


def load_ipython_extension(ip):
    """IPython extension entry point: ``%load_ext conda_emscripten``.

    Registers the ``%cx`` and ``%conda`` line magics and kicks off background
    WASM loading so that ``%cx install <pkg>`` works immediately.
    """
    from .magic import register

    register(ip)

    try:
        import cx_wasm_bridge  # noqa: F401 — import triggers background WASM load

        print("%cx / %conda magic ready — WASM bridge loading in background")
    except ImportError:
        print(
            "%cx / %conda magic registered, but cx-wasm-kernel is not installed.\n"
            "Rebuild with: pixi run -e lite lite-build-local"
        )
