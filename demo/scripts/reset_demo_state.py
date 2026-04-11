#!/usr/bin/env python3

import shutil
from pathlib import Path


TMP_PATHS = [
    Path("/tmp/defuddle-demo-chromium-hook"),
    Path("/tmp/defuddle-demo-close.html"),
]


def safe_remove(path: Path) -> None:
    resolved = path.resolve()
    if not str(resolved).startswith("/tmp/"):
        raise RuntimeError(f"refusing to delete non-/tmp path: {resolved}")
    if resolved.is_dir():
        shutil.rmtree(resolved, ignore_errors=True)
    else:
        resolved.unlink(missing_ok=True)


def main() -> int:
    for path in TMP_PATHS:
        safe_remove(path)
    print("Reset defuddle demo temp state")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
