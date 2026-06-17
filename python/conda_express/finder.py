from __future__ import annotations

import os
from pathlib import Path
import stat
import sys
import sysconfig


class CxNotFound(FileNotFoundError): ...


def find_cx_bin() -> str:
    """Return the path to the cx binary installed by the wheel."""
    cx_exe = "cx" + (sysconfig.get_config_var("EXE") or "")
    module_dir = Path(__file__).parent
    module_bin = module_dir / "bin"

    if module_bin.is_dir():
        path = module_bin / cx_exe
        if path.is_file():
            ensure_executable(str(path))
            return str(path)
        raise CxNotFound(f"Package-owned cx binary is missing: {path}")

    targets = [
        sysconfig.get_path("scripts"),
        sysconfig.get_path("scripts", vars={"base": sys.base_prefix}),
        str(module_dir.parent / "bin"),
        sysconfig.get_path("scripts", scheme=user_scheme()),
    ]

    seen: list[str] = []
    for target in targets:
        if not target or target in seen:
            continue
        seen.append(target)
        path = Path(target) / cx_exe
        if path.is_file():
            require_executable(str(path))
            return str(path)

    locations = "\n".join(f"  - {target}" for target in seen)
    raise CxNotFound(
        f"Could not find the cx binary in any of the following locations:\n{locations}\n"
    )


def ensure_executable(path: str) -> None:
    if os.name == "nt":
        return
    mode = os.stat(path).st_mode
    if mode & stat.S_IXUSR:
        return
    os.chmod(path, mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)


def require_executable(path: str) -> None:
    if os.name == "nt":
        return
    if os.access(path, os.X_OK):
        return
    raise CxNotFound(f"Found cx binary is not executable: {path}")


def user_scheme() -> str:
    if sys.version_info >= (3, 10):
        return sysconfig.get_preferred_scheme("user")
    if os.name == "nt":
        return "nt_user"
    if sys.platform == "darwin" and sys._framework:
        return "osx_framework_user"
    return "posix_user"
