from __future__ import annotations

import os
import sys
import sysconfig


class CxNotFound(FileNotFoundError): ...


def find_cx_bin():
    """Return the path to the cx binary."""
    cx_exe = "cx" + sysconfig.get_config_var("EXE")

    targets = [
        sysconfig.get_path("scripts"),
        sysconfig.get_path("scripts", vars={"base": sys.base_prefix}),
        _join(
            _matching_parents(_module_path(), "lib/python*/site-packages/conda_express"),
            "bin",
        )
        if sys.platform != "win32"
        else _join(
            _matching_parents(
                _module_path(), "Lib/site-packages/conda_express"
            ),
            "Scripts",
        ),
        _join(_matching_parents(_module_path(), "conda_express"), "bin"),
        sysconfig.get_path("scripts", scheme=_user_scheme()),
    ]

    seen = []
    for t in targets:
        if not t or t in seen:
            continue
        seen.append(t)
        path = os.path.join(t, cx_exe)
        if os.path.isfile(path):
            return path

    locations = "\n".join(f"  - {t}" for t in seen)
    raise CxNotFound(
        f"Could not find the cx binary in any of the following locations:\n{locations}\n"
    )


def _module_path():
    return os.path.dirname(__file__)


def _matching_parents(path, match):
    """Walk backwards through path segments matching glob pattern."""
    from fnmatch import fnmatch

    if not path:
        return None
    parts = path.split(os.sep)
    match_parts = match.split("/")
    if len(parts) < len(match_parts):
        return None
    if not all(
        fnmatch(p, m)
        for p, m in zip(reversed(parts), reversed(match_parts))
    ):
        return None
    return os.sep.join(parts[: -len(match_parts)])


def _join(path, *parts):
    if not path:
        return None
    return os.path.join(path, *parts)


def _user_scheme():
    if sys.version_info >= (3, 10):
        return sysconfig.get_preferred_scheme("user")
    elif os.name == "nt":
        return "nt_user"
    elif sys.platform == "darwin" and sys._framework:
        return "osx_framework_user"
    else:
        return "posix_user"
