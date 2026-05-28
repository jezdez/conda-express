#!/usr/bin/env python3
from __future__ import annotations

import argparse
import base64
import csv
import hashlib
import io
from pathlib import Path
import textwrap
import zipfile


ROOT = Path(__file__).resolve().parents[1]
PACKAGE = "conda_express"
DIST = "conda_express"


def main() -> None:
    parser = argparse.ArgumentParser(description="Build a conda-express binary wheel")
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--version", required=True)
    parser.add_argument("--platform-tag", required=True)
    parser.add_argument("--out-dir", default=Path("dist"), type=Path)
    args = parser.parse_args()

    binary = args.binary.resolve()
    if not binary.is_file():
        raise SystemExit(f"binary not found: {binary}")

    out_dir = args.out_dir.resolve()
    out_dir.mkdir(parents=True, exist_ok=True)

    wheel_name = f"{DIST}-{args.version}-py3-none-{args.platform_tag}.whl"
    wheel_path = out_dir / wheel_name
    dist_info = f"{DIST}-{args.version}.dist-info"

    records: list[tuple[str, str, str]] = []
    with zipfile.ZipFile(wheel_path, "w", compression=zipfile.ZIP_DEFLATED) as wheel:
        for path in sorted((ROOT / "python" / PACKAGE).glob("**/*")):
            if path.is_file():
                rel = Path(PACKAGE) / path.relative_to(ROOT / "python" / PACKAGE)
                write_file(wheel, records, rel.as_posix(), path.read_bytes())

        script_name = "cx.exe" if binary.name.endswith(".exe") else "cx"
        write_file(
            wheel,
            records,
            f"{PACKAGE}/bin/{script_name}",
            binary.read_bytes(),
            mode=0o755,
        )

        write_file(wheel, records, f"{dist_info}/METADATA", metadata(args.version))
        write_file(wheel, records, f"{dist_info}/WHEEL", wheel_metadata(args.platform_tag))
        write_file(wheel, records, f"{dist_info}/entry_points.txt", entry_points())
        write_file(wheel, records, f"{dist_info}/top_level.txt", f"{PACKAGE}\n")
        write_file(
            wheel,
            records,
            f"{dist_info}/licenses/LICENSE",
            (ROOT / "LICENSE").read_bytes(),
        )

        record_path = f"{dist_info}/RECORD"
        records.append((record_path, "", ""))
        record = render_record(records)
        info = zipfile.ZipInfo(record_path)
        info.external_attr = (0o644 & 0xFFFF) << 16
        wheel.writestr(info, record)

    print(wheel_path)


def metadata(version: str) -> str:
    return textwrap.dedent(
        f"""\
        Metadata-Version: 2.4
        Name: conda-express
        Version: {version}
        Summary: A lightweight, single-binary conda bootstrapper
        License-Expression: BSD-3-Clause
        Requires-Python: >=3.8
        Description-Content-Type: text/markdown
        Project-URL: Repository, https://github.com/jezdez/conda-express
        Project-URL: Documentation, https://jezdez.github.io/conda-express/
        Project-URL: Issues, https://github.com/jezdez/conda-express/issues
        Classifier: Development Status :: 3 - Alpha
        Classifier: Environment :: Console
        Classifier: Intended Audience :: Developers
        Classifier: Intended Audience :: Science/Research
        Classifier: License :: OSI Approved :: BSD License
        Classifier: Programming Language :: Python :: 3
        Classifier: Topic :: System :: Installation/Setup
        Classifier: Topic :: System :: Software Distribution

        conda-express installs the Pronto-built `cx` binary for this platform.
        """
    )


def wheel_metadata(platform_tag: str) -> str:
    tags = "\n".join(f"Tag: py3-none-{tag}" for tag in platform_tag.split("."))
    return textwrap.dedent(
        f"""\
        Wheel-Version: 1.0
        Generator: conda-express
        Root-Is-Purelib: false
        {tags}
        """
    )


def entry_points() -> str:
    return textwrap.dedent(
        """\
        [console_scripts]
        cx = conda_express.__main__:main
        """
    )


def write_file(
    wheel: zipfile.ZipFile,
    records: list[tuple[str, str, str]],
    path: str,
    content: bytes | str,
    mode: int = 0o644,
) -> None:
    if isinstance(content, str):
        content = content.encode()
    info = zipfile.ZipInfo(path)
    info.external_attr = (mode & 0xFFFF) << 16
    wheel.writestr(info, content)
    digest = base64.urlsafe_b64encode(hashlib.sha256(content).digest()).rstrip(b"=")
    records.append((path, f"sha256={digest.decode()}", str(len(content))))


def render_record(records: list[tuple[str, str, str]]) -> str:
    out = io.StringIO()
    writer = csv.writer(out, lineterminator="\n")
    writer.writerows(records)
    return out.getvalue()


if __name__ == "__main__":
    main()
