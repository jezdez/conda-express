"""cx-wasm kernel bridge.

Loads the cx-wasm WebAssembly module from MEMFS (where jupyterlite-xeus installs
it) and registers the bridge functions on the JS global scope so that the
conda-emscripten solver and extractor can call into Rust/WASM.

Quick start::

    import cx_wasm_bridge          # auto-schedules background WASM loading
    await cx_wasm_bridge.setup()   # wait + register js.fetch_and_solve etc.

The ``%cx`` magic and emscripten compatibility patches live in the
``conda-emscripten`` plugin package, not here.
"""

from __future__ import annotations

import asyncio
import logging
import os
import sys

log = logging.getLogger(__name__)

_cx = None  # cached ES-module proxy after _load_cx()
_setup_done = False  # True once js.fetch_and_solve etc. are registered

_load_lock: asyncio.Lock | None = None
_setup_lock: asyncio.Lock | None = None


def _get_load_lock() -> asyncio.Lock:
    global _load_lock
    if _load_lock is None:
        _load_lock = asyncio.Lock()
    return _load_lock


def _get_setup_lock() -> asyncio.Lock:
    global _setup_lock
    if _setup_lock is None:
        _setup_lock = asyncio.Lock()
    return _setup_lock


def is_ready() -> bool:
    """Return True if the bridge is set up and conda install will work."""
    return _setup_done


def _sync_fetch_binary(url: str):
    """Synchronous XHR → JS Uint8Array (fetch_binary callback for Rust)."""
    import js  # noqa: PLC0415

    xhr = js.XMLHttpRequest.new()
    xhr.open("GET", str(url), False)
    xhr.responseType = "arraybuffer"
    xhr.send()
    return js.Uint8Array.new(xhr.response)


def _sync_fetch_text(url: str) -> str:
    """Synchronous XHR → Python str (fetch_text callback for Rust)."""
    import js  # noqa: PLC0415

    xhr = js.XMLHttpRequest.new()
    xhr.open("GET", str(url), False)
    xhr.send()
    return str(xhr.responseText)


async def _load_cx():
    """Load the cx-wasm ES module from MEMFS via blob URLs.

    Concurrency-safe: a lock prevents duplicate loading if setup() is called
    concurrently.  Blob URLs are revoked after initialisation to free memory.
    """
    global _cx
    if _cx is not None:
        return _cx

    async with _get_load_lock():
        if _cx is not None:  # another coroutine finished while we waited
            return _cx

        import js  # noqa: PLC0415
        import pyjs  # noqa: PLC0415

        pkg_dir = os.path.dirname(os.path.abspath(__file__))
        wasm_path = os.path.join(pkg_dir, "cx_wasm_bg.wasm")
        js_path = os.path.join(pkg_dir, "cx_wasm.js")

        for path in (wasm_path, js_path):
            if not os.path.exists(path):
                raise FileNotFoundError(
                    f"cx-wasm file not found: {path}\n"
                    "Is cx-wasm-kernel installed? "
                    "Rebuild with: pixi run -e lite lite-build-local"
                )

        wasm_size = os.path.getsize(wasm_path)
        log.info("cx-wasm-bridge: loading WASM (%d bytes)", wasm_size)

        with open(wasm_path, "rb") as fh:
            wasm_bytes = fh.read()
        wasm_data = pyjs.to_js(bytes(wasm_bytes))

        with open(js_path, encoding="utf-8") as fh:
            js_text = fh.read()

        # Perform Blob creation, dynamic import, and WASM init entirely in
        # JS to avoid pyjs proxy issues.  wasm-bindgen's glue code uses
        # `typeof x === 'string'` checks that fail on pyjs string proxies,
        # and Blob() needs a native JS Array with native JS String parts.
        js._cx_load_module = js.Function.new(
            "jsText", "wasmData",
            "var jsUrl = URL.createObjectURL("
            "  new Blob([String(jsText)], {type: 'text/javascript'}));"
            "var wasmUrl = URL.createObjectURL("
            "  new Blob([wasmData], {type: 'application/wasm'}));"
            "return import(jsUrl).then(function(m) {"
            "  return m.default(wasmUrl).then(function() {"
            "    URL.revokeObjectURL(jsUrl);"
            "    URL.revokeObjectURL(wasmUrl);"
            "    return m;"
            "  });"
            "});"
        )
        _cx = await js._cx_load_module(js_text, wasm_data)
        log.info("cx-wasm-bridge: WASM module loaded")
        return _cx


async def setup() -> None:
    """Load cx-wasm and register bridge functions on the JS global scope.

    Idempotent and concurrency-safe.  Subsequent or concurrent calls wait for
    the first to complete, then return immediately.
    """
    global _setup_done
    if _setup_done:
        return

    async with _get_setup_lock():
        if _setup_done:
            return

        import js  # noqa: PLC0415
        import pyjs  # noqa: PLC0415

        cx = await _load_cx()

        # pyjs.create_callable converts Python functions to JS-callable
        # objects.  The handles must be kept alive for the session.
        js_fetch_binary, _hnd1 = pyjs.create_callable(_sync_fetch_binary)
        js_fetch_text, _hnd2 = pyjs.create_callable(_sync_fetch_text)
        js.sync_fetch_binary = js_fetch_binary
        js.sync_fetch_text = js_fetch_text

        # All bridge functions are wrapped with JS Function.new() so that
        # arguments crossing the pyjs boundary are coerced to native JS
        # types.  wasm-bindgen's glue uses `typeof x === 'string'` and
        # `passStringToWasm0` which reject pyjs proxy objects.
        js._cx_solve_raw = cx.cx_fetch_and_solve
        js.fetch_and_solve = js.Function.new(
            "request",
            "return globalThis._cx_solve_raw("
            "  String(request),"
            "  globalThis.sync_fetch_binary,"
            "  globalThis.sync_fetch_text"
            ")"
        )

        js._cx_extract_raw = cx.cx_extract_package
        js.cx_extract_package = js.Function.new(
            "bytes", "filename", "onFile",
            "return globalThis._cx_extract_raw(bytes, String(filename), onFile)"
        )

        _setup_done = True
        log.info("cx-wasm-bridge: js.fetch_and_solve registered")


async def _setup_background() -> None:
    """Exception-safe wrapper for background use via asyncio.ensure_future.

    pyjs cannot serialise Python exceptions back through the JS done-callback
    (BindingError: Cannot pass non-string to std::string), so we absorb them.
    """
    try:
        await setup()
    except Exception as exc:  # noqa: BLE001
        log.warning("cx-wasm-bridge: background setup failed: %s", exc)


def _schedule_auto_setup() -> None:
    """Schedule setup() as a background task at import time (emscripten only).

    If an event loop is running (xeus-python kernel), setup starts immediately.
    By the time the user runs ``await setup()``, loading is likely complete.
    """
    if sys.platform != "emscripten":
        return
    try:
        asyncio.ensure_future(_setup_background())
    except RuntimeError:
        pass  # no running event loop; user must call await setup() explicitly


_schedule_auto_setup()
