"""Emscripten compatibility patches applied at runtime inside xeus-python."""

from __future__ import annotations

import logging
import sys

log = logging.getLogger(__name__)


def patch_urllib3() -> None:
    """Replace urllib3's async Emscripten backend with synchronous XHR.

    conda uses urllib3 for HTTP.  In the xeus-python worker there is no async
    fetch API, so we redirect every request through synchronous XMLHttpRequest.
    Skips if the emscripten backend is not importable (not running in browser).
    """
    if sys.platform != "emscripten":
        return
    try:
        import urllib3.contrib.emscripten.fetch  # noqa: F401
    except ImportError:
        return

    from email.parser import Parser

    import js
    import pyjs

    _IGNORE = {"user-agent"}

    def _pyjs_send_request(request):
        from urllib3.contrib.emscripten.response import EmscriptenResponse

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

    import urllib3.contrib.emscripten.connection as _ec
    import urllib3.contrib.emscripten.fetch as _ef

    _ef.send_request = _pyjs_send_request
    _ec.send_request = _pyjs_send_request
    log.debug("conda-emscripten: urllib3 patched (sync XHR)")


def patch_conda_internals() -> None:
    """Stub conda internals that break under Emscripten MEMFS.

    These are belt-and-suspenders runtime patches; the conda recipe already
    applies equivalent patches at build time (patches/007 and 008).  Kept here
    as a fallback for unpatched conda builds.
    """
    if sys.platform != "emscripten":
        return
    try:
        from conda.core import solve as _solve

        _solve.Solver._notify_conda_outdated = lambda self, link_precs: None

        from conda.gateways.repodata import RepodataCache

        _orig_save = RepodataCache.save

        def _safe_save(self, raw_repodata):
            try:
                return _orig_save(self, raw_repodata)
            except (AttributeError, OSError):
                pass

        RepodataCache.save = _safe_save
    except ImportError:
        pass  # conda not installed; patches not needed
