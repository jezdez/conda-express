from __future__ import annotations

import os
import stat
import sys
import sysconfig


class CxNotFound(FileNotFoundError): ...


def find_cx_bin() -> str:
    """Return the path to the cx binary installed by the wheel."""
    cx_exe = "cx" + (sysconfig.get_config_var("EXE") or "")

    targets = [
        _join(_module_path(), "bin"),
        sysconfig.get_path("scripts"),
        sysconfig.get_path("scripts", vars={"base": sys.base_prefix}),
        _join(_matching_parents(_module_path(), "conda_express"), "bin"),
        sysconfig.get_path("scripts", scheme=_user_scheme()),
    ]

    seen: list[str] = []
    for target in targets:
        if not target or target in seen:
            continue
        seen.append(target)
        path = os.path.join(target, cx_exe)
        if os.path.isfile(path):
            _ensure_executable(path)
            return path

    locations = "\n".join(f"  - {target}" for target in seen)
    raise CxNotFound(
        f"Could not find the cx binary in any of the following locations:\n{locations}\n"
    )


def _module_path() -> str:
    return os.path.dirname(__file__)


def _matching_parents(path: str | None, match: str) -> str | None:
    """Walk backwards through path segments matching a glob pattern."""
    from fnmatch import fnmatch

    if not path:
        return None
    parts = path.split(os.sep)
    match_parts = match.split("/")
    if len(parts) < len(match_parts):
        return None
    if not all(fnmatch(p, m) for p, m in zip(reversed(parts), reversed(match_parts))):
        return None
    return os.sep.join(parts[: -len(match_parts)])


def _join(path: str | None, *parts: str) -> str | None:
    if not path:
        return None
    return os.path.join(path, *parts)


def _ensure_executable(path: str) -> None:
    if os.name == "nt":
        return
    mode = os.stat(path).st_mode
    if mode & stat.S_IXUSR:
        return
    os.chmod(path, mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)


def _user_scheme() -> str:
    if sys.version_info >= (3, 10):
        return sysconfig.get_preferred_scheme("user")
    if os.name == "nt":
        return "nt_user"
    if sys.platform == "darwin" and sys._framework:
        return "osx_framework_user"
    return "posix_user"
