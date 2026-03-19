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

# Must survive for the entire session — pyjs invalidates the JS-side proxy
# when the Python handle is garbage collected.
_bridge_refs: list = []

_locks: dict[str, asyncio.Lock] = {}


def _get_lock(name: str) -> asyncio.Lock:
    """Return a named asyncio.Lock, creating it on first access."""
    if name not in _locks:
        _locks[name] = asyncio.Lock()
    return _locks[name]


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

    async with _get_lock("load"):
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
            "jsText",
            "wasmData",
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
            "});",
        )
        _cx = await js._cx_load_module(js_text, wasm_data)
        log.info("cx-wasm-bridge: WASM module loaded")
        return _cx


_BRIDGE_JS = (
    "globalThis._cxPrefetchCache = new Map();"
    "globalThis._cxPrefetchHits = 0;"
    "globalThis._cxSyncFetchBinary = fetchBin;"
    "globalThis.sync_fetch_binary = function(url) {"
    "  url = String(url);"
    "  var c = globalThis._cxPrefetchCache.get(url);"
    "  if (c) {"
    "    globalThis._cxPrefetchCache.delete(url);"
    "    globalThis._cxPrefetchHits++;"
    "    return c;"
    "  }"
    "  return globalThis._cxSyncFetchBinary(url);"
    "};"
    "globalThis.sync_fetch_text = fetchText;"
    "globalThis._cx_solve_raw = cx.cx_fetch_and_solve;"
    "globalThis.fetch_and_solve = function(request) {"
    "  globalThis._cxPrefetchHits = 0;"
    "  var result = globalThis._cx_solve_raw("
    "    String(request), globalThis.sync_fetch_binary, globalThis.sync_fetch_text);"
    "  console.warn('cx-wasm: solve used ' + globalThis._cxPrefetchHits"
    "    + ' prefetch cache hits, ' + globalThis._cxPrefetchCache.size + ' still cached');"
    "  return result;"
    "};"
    "globalThis._cx_extract_raw = cx.cx_extract_package;"
    "globalThis.cx_extract_package = function(bytes, filename, onFile) {"
    "  return globalThis._cx_extract_raw(bytes, String(filename), onFile);"
    "};"
    "globalThis._cx_get_shard_urls_raw = cx.cx_get_shard_urls;"
    "globalThis.get_shard_urls = function(request) {"
    "  return globalThis._cx_get_shard_urls_raw("
    "    String(request), globalThis.sync_fetch_binary);"
    "};"
    "globalThis.decode_shard_deps = function(data) {"
    "  return cx.cx_decode_shard_deps(data);"
    "};"
    "globalThis.clear_repodata_cache = function() {"
    "  return cx.cx_clear_repodata_cache();"
    "};"
    "globalThis._cx_prefetch_batch = function(urlsOrJson) {"
    "  var urls = typeof urlsOrJson === 'string'"
    "    ? JSON.parse(urlsOrJson) : Array.from(urlsOrJson);"
    "  if (!urls || !urls.length) return Promise.resolve();"
    "  var n = urls.length, done = 0;"
    "  console.warn('cx-wasm: prefetching ' + n + ' shards');"
    "  return new Promise(function(resolve) {"
    "    for (var i = 0; i < n; i++) {"
    "      (function(url) {"
    "        fetch(url)"
    "        .then(function(r) { return r.arrayBuffer(); })"
    "        .then(function(buf) {"
    "          globalThis._cxPrefetchCache.set(url, new Uint8Array(buf));"
    "        })"
    "        .catch(function(e) {"
    "          console.warn('cx-wasm: prefetch fail ' + url.slice(-50) + ': ' + e);"
    "        })"
    "        .finally(function() {"
    "          if (++done >= n) {"
    "            console.warn('cx-wasm: prefetch done, '"
    "              + globalThis._cxPrefetchCache.size + ' cached');"
    "            resolve();"
    "          }"
    "        });"
    "      })(urls[i]);"
    "    }"
    "  });"
    "};"
)

_DEFAULT_CHANNELS = [
    {
        "url": "https://repo.prefix.dev/emscripten-forge-4x",
        "subdirs": ["emscripten-wasm32", "noarch"],
    },
    {
        "url": "https://conda.anaconda.org/conda-forge",
        "subdirs": ["emscripten-wasm32", "noarch"],
    },
]


async def setup() -> None:
    """Load cx-wasm and register bridge functions on the JS global scope.

    Idempotent and concurrency-safe.  Subsequent or concurrent calls wait for
    the first to complete, then return immediately.
    """
    global _setup_done
    if _setup_done:
        return

    async with _get_lock("setup"):
        if _setup_done:
            return

        import js  # noqa: PLC0415
        import pyjs  # noqa: PLC0415

        cx = await _load_cx()

        # pyjs.create_callable returns (js_callable, prevent_gc_handle).
        # Both must be stored in _bridge_refs so pyjs doesn't invalidate
        # the JS-side proxy when the Python handle is garbage collected.
        js_fetch_binary, hnd1 = pyjs.create_callable(_sync_fetch_binary)
        js_fetch_text, hnd2 = pyjs.create_callable(_sync_fetch_text)
        _bridge_refs.extend([js_fetch_binary, hnd1, js_fetch_text, hnd2])

        # Register all bridge functions from pure JS via a single
        # Function.new() call.  Setting globalThis properties through
        # pyjs's js.__setattr__ wraps values in proxy objects that aren't
        # directly callable from JS (typeof returns 'object', not
        # 'function').  By receiving cx/fetchBin/fetchText as Function
        # parameters, JS gets the raw underlying objects and can assign
        # them to globalThis as native callables.
        js._cx_register_bridge = js.Function.new(
            "cx",
            "fetchBin",
            "fetchText",
            _BRIDGE_JS,
        )
        js._cx_register_bridge(cx, js_fetch_binary, js_fetch_text)

        _setup_done = True
        log.info("cx-wasm-bridge: js.fetch_and_solve registered")

        await _prefetch_installed()


async def _prefetch_installed() -> None:
    """Pre-warm the shard cache by traversing installed package dependencies level by level.

    Runs in the async ``setup()`` context.  For each level:
    1. Compute shard URLs for the current set of package names (Rust, cached indices).
    2. Fetch all shards in parallel using async ``fetch()`` from JS.
    3. Decode each shard (Rust, pure computation) to discover dependency names.
    4. Repeat with newly discovered names until no new dependencies remain.

    By the time the user runs ``%conda install``, every shard in the transitive
    dependency closure is already in ``_cxPrefetchCache``.  The solver's
    ``sync_fetch_binary`` returns cached data instantly — zero network I/O.
    """
    import json as _json  # noqa: PLC0415

    import js  # noqa: PLC0415

    conda_meta = os.path.join(sys.prefix, "conda-meta")
    if not os.path.isdir(conda_meta):
        return

    seeds: set[str] = set()
    for fn in os.listdir(conda_meta):
        if not fn.endswith(".json"):
            continue
        try:
            with open(os.path.join(conda_meta, fn)) as f:
                data = _json.load(f)
            name = data.get("name")
            if name:
                seeds.add(name)
        except Exception:  # noqa: BLE001
            pass

    if not seeds:
        return

    try:
        print(f"[cx-prefetch] starting for {len(seeds)} installed packages")

        seen_names: set[str] = set()
        queue = sorted(seeds)
        seen_urls: set[str] = set()
        level = 0
        total_fetched = 0

        while queue:
            new_names = [n for n in queue if n not in seen_names]
            if not new_names:
                break
            seen_names.update(new_names)

            request = _json.dumps(
                {
                    "channels": _DEFAULT_CHANNELS,
                    "seeds": new_names,
                }
            )
            urls_js = js.get_shard_urls(request)
            urls_json = str(js.JSON.stringify(urls_js))
            all_urls: list[str] = _json.loads(urls_json)

            new_urls = [u for u in all_urls if u not in seen_urls]
            seen_urls.update(new_urls)

            if not new_urls:
                break

            print(f"[cx-prefetch] level {level}: {len(new_urls)} shards")
            await js._cx_prefetch_batch(_json.dumps(new_urls))
            total_fetched += len(new_urls)

            next_names: set[str] = set()
            for url in new_urls:
                shard_bytes = js._cxPrefetchCache.get(url)
                if not shard_bytes:
                    continue
                deps_js = js.decode_shard_deps(shard_bytes)
                deps = _json.loads(str(js.JSON.stringify(deps_js)))
                next_names.update(deps)

            queue = sorted(next_names - seen_names)
            level += 1

        print(f"[cx-prefetch] done: {total_fetched} shards across {level} levels")
    except Exception as exc:  # noqa: BLE001
        print(f"[cx-prefetch] FAILED: {type(exc).__name__}: {exc}")
        import traceback  # noqa: PLC0415

        traceback.print_exc()


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
