from __future__ import annotations

import os
import subprocess
import sys

from conda_express import find_cx_bin


def main() -> None:
    cx = find_cx_bin()

    args = [cx, *sys.argv[1:]]
    if sys.platform == "win32":
        raise SystemExit(subprocess.call(args))
    else:
        os.execvp(cx, args)


if __name__ == "__main__":
    main()
