#!/usr/bin/env python3

import shutil
from pathlib import Path


ROOT_DIR = Path(__file__).resolve().parents[2]
SERVO_DIR = Path("/tmp/servo")
DEMO_DB_DIR = Path("/tmp/fsgdb-servo-demo")


def safe_remove(path: Path) -> None:
    resolved = path.resolve()
    if not str(resolved).startswith("/tmp/"):
        raise RuntimeError(f"refusing to delete non-/tmp path: {resolved}")
    if resolved.exists():
        shutil.rmtree(resolved)


def main() -> int:
    safe_remove(SERVO_DIR)
    safe_remove(DEMO_DB_DIR)
    DEMO_DB_DIR.mkdir(parents=True, exist_ok=True)
    print(f"Reset demo state: removed {SERVO_DIR} and recreated {DEMO_DB_DIR}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
