"""Package extractor for Emscripten environments.

Extracts .conda and .tar.bz2 packages via cx-wasm's Rust WASM code,
with a Python streaming tarfile fallback for .tar.bz2 if WASM fails.
"""

from __future__ import annotations

import logging
import os
from pathlib import PurePosixPath
from posixpath import normpath

log = logging.getLogger(__name__)


def is_within(path, directory):
    """Check that *path* stays inside *directory* after resolving ``..``."""
    return PurePosixPath(normpath(path)).is_relative_to(normpath(directory))


def extract_wasm(source_path, dest_dir):
    """Extract a conda package archive on Emscripten MEMFS.

    Routes both .conda and .tar.bz2 through the cx-wasm Rust extractor.
    Falls back to Python's streaming tarfile for .tar.bz2 if WASM fails.
    """
    source_path = os.fspath(source_path)
    dest_dir = os.fspath(dest_dir)
    filename = os.path.basename(source_path)

    if not os.path.isfile(source_path):
        raise FileNotFoundError(f"extractor: source archive not found: {source_path}")

    file_size = os.path.getsize(source_path)
    log.info(
        "extractor: extracting %s (%d bytes) -> %s",
        filename,
        file_size,
        dest_dir,
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
        if not is_within(full_path, dest_dir):
            raise RuntimeError(f"extractor: path escapes destination: {path}")
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
    from posixpath import dirname, join

    os.makedirs(dest_dir, exist_ok=True)
    file_count = 0
    file_contents: dict[str, bytes] = {}
    deferred_links: list[tuple[str, str]] = []

    with open(source_path, "rb") as raw:
        with tarfile.open(fileobj=raw, mode="r|bz2") as tar:
            for member in tar:
                if not (member.isfile() or member.isdir() or member.issym() or member.islnk()):
                    continue

                if not is_within(join(dest_dir, member.name), dest_dir):
                    raise RuntimeError(
                        f"extractor: tar path escapes destination: {member.name}"
                    )

                if member.isdir():
                    os.makedirs(join(dest_dir, member.name), exist_ok=True)
                    continue

                if member.issym():
                    parent = dirname(member.name)
                    resolved = normpath(join(parent, member.linkname))
                    if not is_within(join(dest_dir, resolved), dest_dir):
                        raise RuntimeError(
                            f"extractor: symlink escapes destination: "
                            f"{member.name} -> {member.linkname}"
                        )
                    deferred_links.append((member.name, resolved))
                    continue

                if member.islnk():
                    if not is_within(join(dest_dir, member.linkname), dest_dir):
                        raise RuntimeError(
                            f"extractor: hardlink escapes destination: "
                            f"{member.name} -> {member.linkname}"
                        )
                    deferred_links.append((member.name, member.linkname))
                    continue

                parent = dirname(join(dest_dir, member.name))
                if parent:
                    os.makedirs(parent, exist_ok=True)
                extracted = tar.extractfile(member)
                if extracted is not None:
                    data = extracted.read()
                    with open(join(dest_dir, member.name), "wb") as out:
                        out.write(data)
                    file_contents[member.name] = data
                    file_count += 1

    for path, target in deferred_links:
        dest_path = join(dest_dir, path)
        parent = dirname(dest_path)
        if parent:
            os.makedirs(parent, exist_ok=True)
        if target in file_contents:
            with open(dest_path, "wb") as out:
                out.write(file_contents[target])
            file_count += 1
        else:
            src_path = join(dest_dir, target)
            if os.path.isfile(src_path):
                import shutil
                shutil.copy2(src_path, dest_path)
                file_count += 1

    log.info("extractor: extracted %d files from %s (tar|bz2)", file_count, filename)
