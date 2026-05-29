from __future__ import annotations

import os
import stat
import sys
import sysconfig


class CxNotFound(FileNotFoundError): ...


def find_cx_bin() -> str:
    """Return the path to the cx binary installed by the wheel."""
    cx_exe = "cx" + (sysconfig.get_config_var("EXE") or "")
    module_bin = join_path(module_path(), "bin")

    if module_bin and os.path.isdir(module_bin):
        path = os.path.join(module_bin, cx_exe)
        if os.path.isfile(path):
            ensure_executable(path)
            return path
        raise CxNotFound(f"Package-owned cx binary is missing: {path}")

    targets = [
        sysconfig.get_path("scripts"),
        sysconfig.get_path("scripts", vars={"base": sys.base_prefix}),
        join_path(matching_parents(module_path(), "conda_express"), "bin"),
        sysconfig.get_path("scripts", scheme=user_scheme()),
    ]

    seen: list[str] = []
    for target in targets:
        if not target or target in seen:
            continue
        seen.append(target)
        path = os.path.join(target, cx_exe)
        if os.path.isfile(path):
            ensure_executable(path)
            return path

    locations = "\n".join(f"  - {target}" for target in seen)
    raise CxNotFound(
        f"Could not find the cx binary in any of the following locations:\n{locations}\n"
    )


def module_path() -> str:
    return os.path.dirname(__file__)


def matching_parents(path: str | None, match: str) -> str | None:
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


def join_path(path: str | None, *parts: str) -> str | None:
    if not path:
        return None
    return os.path.join(path, *parts)


def ensure_executable(path: str) -> None:
    if os.name == "nt":
        return
    mode = os.stat(path).st_mode
    if mode & stat.S_IXUSR:
        return
    os.chmod(path, mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)


def user_scheme() -> str:
    if sys.version_info >= (3, 10):
        return sysconfig.get_preferred_scheme("user")
    if os.name == "nt":
        return "nt_user"
    if sys.platform == "darwin" and sys._framework:
        return "osx_framework_user"
    return "posix_user"
