from __future__ import annotations

import os
# Windows cannot replace the current process, so it delegates to the resolved cx path.
import subprocess  # nosec B404
import sys

from conda_express import find_cx_bin


def main() -> None:
    cx = find_cx_bin()
    args = [cx, *sys.argv[1:]]
    if sys.platform == "win32":
        raise SystemExit(subprocess.call(args, shell=False))  # nosec
    # POSIX replaces the shim process with the resolved cx path.
    os.execv(cx, args)  # nosec B606


if __name__ == "__main__":
    main()
