"""Package extractor for Emscripten environments.

Extracts .conda and .tar.bz2 packages via cx-wasm's Rust WASM code,
with a Python streaming tarfile fallback for .tar.bz2 if WASM fails.
"""

import logging
import os

log = logging.getLogger(__name__)


def extract_wasm(source_path, dest_dir):
    """Extract a conda package archive on Emscripten MEMFS.

    Routes both .conda and .tar.bz2 through the cx-wasm Rust extractor.
    Falls back to Python's streaming tarfile for .tar.bz2 if WASM fails.
    """
    source_path = os.fspath(source_path)
    dest_dir = os.fspath(dest_dir)
    filename = os.path.basename(source_path)

    if not os.path.isfile(source_path):
        raise FileNotFoundError(
            f"extractor: source archive not found: {source_path}"
        )

    file_size = os.path.getsize(source_path)
    log.info(
        "extractor: extracting %s (%d bytes) -> %s",
        filename, file_size, dest_dir,
    )

    if file_size == 0:
        raise RuntimeError(f"extractor: archive is empty (0 bytes): {source_path}")

    try:
        _extract_via_wasm(source_path, dest_dir, filename)
    except Exception:
        if filename.endswith(".tar.bz2"):
            log.info("extractor: WASM failed for %s, using Python tarfile", filename)
            _extract_tar_bz2(source_path, dest_dir, filename)
        else:
            raise


def _extract_via_wasm(source_path, dest_dir, filename):
    """Extract any supported format via cx-wasm's Rust WASM extractor.

    Converts Python bytes to a JS ``Uint8Array`` explicitly — wasm-bindgen
    requires this exact type for ``&[u8]`` parameters.  A plain
    ``pyjs.to_js(bytes)`` may produce a different JS type that wasm-bindgen
    can't interpret, resulting in an empty reader on the Rust side.
    """
    import js
    import pyjs

    with open(source_path, "rb") as f:
        archive_bytes = f.read()

    file_count = 0

    def on_file(path, data):
        nonlocal file_count
        full_path = os.path.join(dest_dir, path)
        parent = os.path.dirname(full_path)
        if parent:
            os.makedirs(parent, exist_ok=True)
        with open(full_path, "wb") as out:
            out.write(bytes(pyjs.to_py(data)))
        file_count += 1

    js_bytes = js.Uint8Array.new(pyjs.to_js(archive_bytes))
    js_callable, handle = pyjs.create_callable(on_file)
    try:
        js.cx_extract_package(js_bytes, filename, js_callable)
    finally:
        handle.delete()

    log.info("extractor: extracted %d files from %s (wasm)", file_count, filename)


def _extract_tar_bz2(source_path, dest_dir, filename):
    """Fallback: extract .tar.bz2 using Python's tarfile in streaming mode.

    The ``r|bz2`` mode reads sequentially without seeking, which is
    required on Emscripten's MEMFS.
    """
    import tarfile

    os.makedirs(dest_dir, exist_ok=True)
    file_count = 0
    with open(source_path, "rb") as raw:
        with tarfile.open(fileobj=raw, mode="r|bz2") as tar:
            for member in tar:
                if member.isdir():
                    os.makedirs(os.path.join(dest_dir, member.name), exist_ok=True)
                    continue
                parent = os.path.dirname(os.path.join(dest_dir, member.name))
                if parent:
                    os.makedirs(parent, exist_ok=True)
                extracted = tar.extractfile(member)
                if extracted is not None:
                    with open(os.path.join(dest_dir, member.name), "wb") as out:
                        out.write(extracted.read())
                    file_count += 1
    log.info("extractor: extracted %d files from %s (tar|bz2)", file_count, filename)
