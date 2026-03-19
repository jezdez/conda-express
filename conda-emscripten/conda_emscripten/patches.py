"""Emscripten compatibility patches applied at runtime inside xeus-python."""

from __future__ import annotations

import logging
import sys
import time

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


_conda_internals_patched = False


def patch_conda_internals() -> None:
    """Stub conda internals that break under Emscripten MEMFS.

    These are belt-and-suspenders runtime patches; the conda recipe already
    applies equivalent patches at build time (patches/007 and 008).  Kept here
    as a fallback for unpatched conda builds.

    Idempotent — safe to call from both the ``%cx`` magic and the
    ``conda_pre_commands`` plugin hook.
    """
    global _conda_internals_patched
    if sys.platform != "emscripten" or _conda_internals_patched:
        return
    _conda_internals_patched = True
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

        # conda's download pipeline (download_inner + download_partial_file)
        # uses seek() on the target file for partial downloads and checksum
        # verification.  MEMFS doesn't support seek at all, so replace
        # download_inner with a simple fetch-verify-write that never seeks.
        import conda.gateways.connection.download as _dl

        def _memfs_download_inner(
            url, target_full_path, md5, sha256, size, progress_update_callback
        ):
            import hashlib
            from pathlib import Path

            from conda.base.context import context as _ctx

            t0 = time.perf_counter()
            timeout = (
                _ctx.remote_connect_timeout_secs,
                _ctx.remote_read_timeout_secs,
            )
            session = _dl.get_session(url)
            resp = session.get(
                url,
                proxies=session.proxies,
                timeout=timeout,
            )
            if log.isEnabledFor(logging.DEBUG):
                log.debug(_dl.stringify(resp, content_max_len=256))
            resp.raise_for_status()

            data = resp.content

            if sha256 or md5:
                checksum_type = "sha256" if sha256 else "md5"
                expected = sha256 if sha256 else md5
                actual = hashlib.new(checksum_type, data).hexdigest()
                if actual != expected:
                    from conda.exceptions import ChecksumMismatchError

                    raise ChecksumMismatchError(
                        url,
                        str(target_full_path),
                        checksum_type,
                        expected,
                        actual,
                    )

            target = Path(target_full_path)
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)

            pkg = url.rsplit("/", 1)[-1]
            print(
                f"[cx-timing] download:       {time.perf_counter() - t0:.2f}s ({pkg})"
            )

        _dl.download_inner = _memfs_download_inner
        log.debug("conda-emscripten: download_inner patched (no seek)")

        # conda's built-in extractor uses conda_package_handling which calls
        # tarfile.open() — that needs seek() which MEMFS doesn't support.
        # Patch ExtractPackageAction.execute to call our WASM extractor
        # in place of context.plugin_manager.extract_package, keeping
        # all original post-extraction bookkeeping intact.
        from .extractor import extract_wasm
        from conda.core.path_actions import ExtractPackageAction

        _orig_epa_execute = ExtractPackageAction.execute

        def _wasm_epa_execute(self, progress_update_callback=None):
            import json as _json
            from os.path import basename, getsize, join, lexists

            from conda.base.context import context as _ctx
            from conda.core.package_cache_data import PackageCacheData
            from conda.gateways.disk.delete import rm_rf
            from conda.gateways.disk.read import read_index_json
            from conda.gateways.disk.create import write_as_json_to_file
            from conda.models.channel import Channel
            from conda.models.match_spec import MatchSpec
            from conda.models.records import PackageCacheRecord, PackageRecord
            from conda.common.url import has_platform
            from conda.gateways.disk.read import compute_sum

            t0 = time.perf_counter()

            log.debug(
                "conda-emscripten: extracting %s → %s (WASM)",
                self.source_full_path,
                self.target_full_path,
            )
            if lexists(self.target_full_path):
                rm_rf(self.target_full_path)

            extract_wasm(self.source_full_path, self.target_full_path)

            try:
                raw_index_json = read_index_json(self.target_full_path)
            except (OSError, _json.JSONDecodeError, FileNotFoundError):
                print(f"ERROR: corrupt package tarball at {self.source_full_path}.")
                return

            if isinstance(self.record_or_spec, MatchSpec):
                url = self.record_or_spec.get_raw_value("url")
                if not url:
                    raise ValueError("URL cannot be empty.")
                channel = (
                    Channel(url)
                    if has_platform(url, _ctx.known_subdirs)
                    else Channel(None)
                )
                fn = basename(url)
                sha256 = self.sha256 or compute_sum(self.source_full_path, "sha256")
                size = getsize(self.source_full_path)
                md5 = self.md5 or compute_sum(self.source_full_path, "md5")
                repodata_record = PackageRecord.from_objects(
                    raw_index_json,
                    url=url,
                    channel=channel,
                    fn=fn,
                    sha256=sha256,
                    size=size,
                    md5=md5,
                )
            else:
                repodata_record = PackageRecord.from_objects(
                    self.record_or_spec,
                    raw_index_json,
                )

            repodata_record_path = join(
                self.target_full_path,
                "info",
                "repodata_record.json",
            )
            write_as_json_to_file(repodata_record_path, repodata_record)

            target_package_cache = PackageCacheData(self.target_pkgs_dir)
            package_cache_record = PackageCacheRecord.from_objects(
                repodata_record,
                package_tarball_full_path=self.source_full_path,
                extracted_package_dir=self.target_full_path,
            )
            target_package_cache.insert(package_cache_record)

            pkg = basename(self.source_full_path)
            print(
                f"[cx-timing] extract:        {time.perf_counter() - t0:.2f}s ({pkg})"
            )

        ExtractPackageAction.execute = _wasm_epa_execute
        log.debug(
            "conda-emscripten: ExtractPackageAction.execute patched (WASM extractor)"
        )

        # Timing wrappers for solve and transaction phases.
        from .solver import CxWasmSolver

        _orig_solve = CxWasmSolver.solve_final_state

        def _timed_solve(self, *args, **kwargs):
            t0 = time.perf_counter()
            result = _orig_solve(self, *args, **kwargs)
            print(f"[cx-timing] solve:          {time.perf_counter() - t0:.2f}s")
            return result

        CxWasmSolver.solve_final_state = _timed_solve

        from conda.core.link import UnlinkLinkTransaction

        _orig_txn_execute = UnlinkLinkTransaction.execute

        def _timed_txn_execute(self):
            t0 = time.perf_counter()
            result = _orig_txn_execute(self)
            print(f"[cx-timing] transaction:    {time.perf_counter() - t0:.2f}s")
            return result

        UnlinkLinkTransaction.execute = _timed_txn_execute
        log.debug("conda-emscripten: timing wrappers installed")

        # The build-time subprocess stub (patch 001) raises RuntimeError,
        # which causes conda's transaction to roll back.  Replace both
        # subprocess entry points with silent no-ops that return success.
        import conda.gateways.subprocess as _sp

        def _noop_any_subprocess(args, prefix, env=None, cwd=None):
            log.debug("conda-emscripten: skipping subprocess: %s", args)
            return "", "", 0

        def _noop_subprocess_call(
            command,
            env=None,
            path=None,
            stdin=None,
            raise_on_error=True,
            capture_output=True,
        ):
            log.debug("conda-emscripten: skipping subprocess_call: %s", command)
            return _sp.Response("", "", 0)

        _sp.any_subprocess = _noop_any_subprocess
        _sp.subprocess_call = _noop_subprocess_call
        log.debug("conda-emscripten: subprocess patched (no-op)")

    except ImportError:
        pass  # conda not installed; patches not needed
