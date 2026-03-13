"""WASM-based package extractor for Emscripten environments.

Bridges conda's package extraction to cx-wasm's Rust WASM extraction code
via pyjs (emscripten-forge JS interop layer).
"""

import logging
import os

log = logging.getLogger(__name__)


def extract_wasm(source_path, dest_dir):
    """Extract a .conda or .tar.bz2 package using cx-wasm's WASM extractor.

    Reads the archive bytes from the Emscripten MEMFS, passes them to
    ``js.cx_extract_package`` (cx-wasm), and writes each extracted file
    into *dest_dir*.

    Parameters
    ----------
    source_path : str or PathLike
        Path to the package archive on the virtual filesystem.
    dest_dir : str or PathLike
        Directory to extract package contents into.
    """
    import js
    import pyjs

    source_path = os.fspath(source_path)
    dest_dir = os.fspath(dest_dir)
    filename = os.path.basename(source_path)

    if not os.path.isfile(source_path):
        raise FileNotFoundError(
            f"wasm-extractor: source archive not found: {source_path}"
        )

    file_size = os.path.getsize(source_path)
    log.info(
        "wasm-extractor: extracting %s (%d bytes) -> %s",
        filename,
        file_size,
        dest_dir,
    )

    if file_size == 0:
        raise RuntimeError(f"wasm-extractor: archive is empty (0 bytes): {source_path}")

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

    js_bytes = pyjs.to_js(archive_bytes)
    js_bytes = pyjs.new(js.Uint8Array, js_bytes)

    js_callable, handle = pyjs.create_callable(on_file)
    try:
        js.cx_extract_package(js_bytes, filename, js_callable)
    finally:
        handle.delete()

    log.info("wasm-extractor: extracted %d files from %s", file_count, filename)
