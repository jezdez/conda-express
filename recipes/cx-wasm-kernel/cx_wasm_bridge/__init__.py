"""cx-wasm kernel bridge.

Loads the cx-wasm WebAssembly module from this package's files (which land in
the xeus-python kernel's Emscripten MEMFS at install time) and registers the
bridge functions that conda-emscripten's solver and extractor expect on the
JavaScript global scope (``import js`` in Python).

Usage inside a JupyterLite notebook (xeus-python kernel)::

    import cx_wasm_bridge
    await cx_wasm_bridge.setup()

After ``setup()`` succeeds, ``conda install`` works from the same kernel
session via conda-emscripten's WasmSolver and wasm-extractor plugins.
"""

from __future__ import annotations

import logging
import os

log = logging.getLogger(__name__)

_cx = None  # cached ES-module proxy, set by _load_cx()


# ── Sync XHR helpers ──────────────────────────────────────────────────────────


def _sync_fetch_binary(url: str):
    """Synchronous XHR returning a JS Uint8Array (fetch_binary callback for Rust)."""
    import js  # noqa: PLC0415

    xhr = js.XMLHttpRequest.new()
    xhr.open("GET", str(url), False)
    xhr.responseType = "arraybuffer"
    xhr.send()
    return js.Uint8Array.new(xhr.response)


def _sync_fetch_text(url: str) -> str:
    """Synchronous XHR returning a Python str (fetch_text callback for Rust)."""
    import js  # noqa: PLC0415

    xhr = js.XMLHttpRequest.new()
    xhr.open("GET", str(url), False)
    xhr.send()
    return str(xhr.responseText)


# ── WASM module loader ────────────────────────────────────────────────────────


async def _load_cx():
    """Load cx-wasm from this package's MEMFS files using blob URLs.

    Steps:
    1. Read cx_wasm_bg.wasm -> Blob -> blob URL  (so init() can fetch the bytes)
    2. Read cx_wasm.js text -> Blob -> blob URL  (so dynamic import() works)
    3. dynamic import(js_blob_url) via a helper function on js.*
    4. await cx.default(wasm_blob_url) — pass WASM URL directly to bypass the
       'new URL(cx_wasm_bg.wasm, import.meta.url)' resolution which would fail
       when the module itself was loaded from a blob URL.
    """
    global _cx
    if _cx is not None:
        return _cx

    import js  # noqa: PLC0415
    import pyjs  # noqa: PLC0415

    pkg_dir = os.path.dirname(os.path.abspath(__file__))
    wasm_path = os.path.join(pkg_dir, "cx_wasm_bg.wasm")
    js_path = os.path.join(pkg_dir, "cx_wasm.js")

    log.info("cx-wasm-bridge: loading %s (%d bytes)", wasm_path, os.path.getsize(wasm_path))

    # 1. WASM blob URL
    with open(wasm_path, "rb") as fh:
        wasm_bytes = bytes(fh.read())
    wasm_arr = pyjs.to_js(wasm_bytes)
    wasm_blob = js.Blob.new([wasm_arr], pyjs.to_js({"type": "application/wasm"}))
    wasm_url = str(js.URL.createObjectURL(wasm_blob))

    # 2. JS module blob URL
    with open(js_path, encoding="utf-8") as fh:
        js_src = fh.read()
    js_blob = js.Blob.new([js_src], pyjs.to_js({"type": "text/javascript"}))
    js_url = str(js.URL.createObjectURL(js_blob))

    # 3. Register a tiny helper on the JS global so we can trigger a dynamic
    #    import() from Python without using pyjs.eval directly.
    #    (pyjs.eval is intentionally not used here for security reasons.)
    js._cx_dynamic_import = js.Function.new("url", "return import(url)")

    # 4. Import the ES module and initialise with the WASM blob URL.
    _cx = await js._cx_dynamic_import(js_url)
    await _cx.default(wasm_url)

    log.info("cx-wasm-bridge: cx-wasm loaded")
    return _cx


# ── Patch helpers ─────────────────────────────────────────────────────────────


def _patch_urllib3() -> None:
    """Replace urllib3's async emscripten backend with sync XHR via pyjs."""
    from email.parser import Parser  # noqa: PLC0415

    import js  # noqa: PLC0415
    import pyjs  # noqa: PLC0415

    _IGNORE = {"user-agent"}

    def _pyjs_send_request(request):
        from urllib3.contrib.emscripten.response import EmscriptenResponse  # noqa: PLC0415

        headers = {k: v for k, v in request.headers.items() if k.lower() not in _IGNORE}
        body = request.body
        if isinstance(body, bytes):
            body = body.decode("latin-1")

        xhr = js.XMLHttpRequest.new()
        xhr.open(request.method, request.url, False)
        xhr.responseType = "arraybuffer"
        for k, v in headers.items():
            xhr.setRequestHeader(k, v)
        xhr.send(body)

        status = int(str(xhr.status))
        raw_headers = str(xhr.getAllResponseHeaders())
        resp_headers = dict(Parser().parsestr(raw_headers))
        resp_body = bytes(pyjs.to_py(js.Uint8Array.new(xhr.response)))

        return EmscriptenResponse(
            status_code=status,
            headers=resp_headers,
            body=resp_body,
            request=request,
        )

    import urllib3.contrib.emscripten.connection as _ec  # noqa: PLC0415
    import urllib3.contrib.emscripten.fetch as _ef  # noqa: PLC0415

    _ef.send_request = _pyjs_send_request
    _ec.send_request = _pyjs_send_request
    log.info("cx-wasm-bridge: urllib3 Emscripten backend patched (sync XHR)")


def _patch_conda() -> None:
    """Stub conda internals that break under Emscripten MEMFS."""
    import sys  # noqa: PLC0415

    if sys.platform != "emscripten":
        return

    import fcntl  # noqa: PLC0415

    if not hasattr(fcntl, "lockf"):
        fcntl.lockf = lambda fd, op, *a, **kw: None
    if not hasattr(fcntl, "flock"):
        fcntl.flock = lambda fd, op: None

    from conda.core import solve as _solve  # noqa: PLC0415

    _solve.Solver._notify_conda_outdated = lambda self, link_precs: None

    from conda.gateways.repodata import RepodataCache  # noqa: PLC0415

    _orig_save = RepodataCache.save

    def _safe_save(self, raw_repodata):
        try:
            return _orig_save(self, raw_repodata)
        except (AttributeError, OSError):
            pass

    RepodataCache.save = _safe_save
    log.info("cx-wasm-bridge: conda Emscripten patches applied")


# ── Public API ────────────────────────────────────────────────────────────────


async def setup() -> None:
    """Load cx-wasm and register all bridge functions on the JS global scope.

    Must be awaited once per kernel session before running ``conda install``::

        import cx_wasm_bridge
        await cx_wasm_bridge.setup()

    Registers on ``import js``:

    * ``js.fetch_and_solve(request_json)``   — used by conda-emscripten WasmSolver
    * ``js.cx_extract_package(bytes, fn, cb)``— used by wasm-extractor plugin
    * ``js.sync_fetch_binary(url)``           — sync XHR → Uint8Array
    * ``js.sync_fetch_text(url)``             — sync XHR → str
    """
    import js  # noqa: PLC0415
    import pyjs  # noqa: PLC0415

    cx = await _load_cx()

    js_fetch_binary = pyjs.to_js(_sync_fetch_binary)
    js_fetch_text = pyjs.to_js(_sync_fetch_text)

    def _fetch_and_solve(request_json):
        return cx.cx_fetch_and_solve(request_json, js_fetch_binary, js_fetch_text)

    js.fetch_and_solve = pyjs.to_js(_fetch_and_solve)
    js.cx_extract_package = cx.cx_extract_package
    js.sync_fetch_binary = js_fetch_binary
    js.sync_fetch_text = js_fetch_text

    _patch_urllib3()
    _patch_conda()

    print("[cx-wasm-bridge] ready — fetch_and_solve and cx_extract_package registered")
