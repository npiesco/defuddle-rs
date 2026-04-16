#!/usr/bin/env python3
"""Run defuddle segment 1: visible hook scene on the live SQLite page."""
import os
import signal
import subprocess
import sys
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[2]
OUTPUT_DIR = ROOT_DIR / "demo" / "output"
DEMO_URL = os.environ.get(
    "DEFUDDLE_DEMO_URL",
    "https://sqlite.org/forum/forumpost/40a2358ad9241700?",
)
BROWSER_PROFILE_DIR = Path("/tmp/defuddle-demo-chromium-hook")
SEGMENT_PATH = OUTPUT_DIR / "seg_01_hook.mp4"


def _check_prerequisites() -> list[str]:
    import shutil
    errors = []
    if not os.environ.get("DISPLAY"):
        errors.append("DISPLAY is not set")
    if not shutil.which("chromium"):
        errors.append("chromium not installed")
    if not shutil.which("xdotool"):
        errors.append("xdotool not installed")
    return errors


def _run_capture(*args: str) -> str:
    result = subprocess.run(args, check=False, capture_output=True, text=True)
    return result.stdout.strip() if result.returncode == 0 else ""


def _wait_for_window_by_pid(pid: int, timeout: float = 15.0) -> str:
    deadline = time.time() + timeout
    while time.time() < deadline:
        output = _run_capture("xdotool", "search", "--onlyvisible", "--pid", str(pid))
        windows = [w.strip() for w in output.splitlines() if w.strip()]
        if windows:
            return windows[-1]
        time.sleep(0.1)
    return ""


def main() -> int:
    errors = _check_prerequisites()
    if errors:
        print("[hook-scene] PREREQUISITES FAILED:", file=sys.stderr)
        for e in errors:
            print(f"  - {e}", file=sys.stderr)
        return 1

    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    SEGMENT_PATH.unlink(missing_ok=True)
    BROWSER_PROFILE_DIR.mkdir(parents=True, exist_ok=True)

    process = subprocess.Popen(
        [
            "chromium",
            "--new-window",
            DEMO_URL,
            "--no-first-run",
            "--no-default-browser-check",
            f"--user-data-dir={BROWSER_PROFILE_DIR}",
        ],
        cwd=ROOT_DIR,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        start_new_session=True,
    )

    window_id = _wait_for_window_by_pid(process.pid, timeout=30.0)
    if not window_id:
        print(f"[hook-scene] browser window not found for pid={process.pid}", file=sys.stderr)
        return 1

    title = _run_capture("xdotool", "getwindowname", window_id)
    subprocess.run(
        ["xdotool", "windowactivate", "--sync", window_id],
        check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )
    hold_seconds = float(os.environ.get("DEMO_HOOK_HOLD_SECONDS", "8"))
    print(
        f"[hook-scene] visible window_id={window_id} pid={process.pid} title={title!r} url={DEMO_URL}",
        file=sys.stderr,
    )
    time.sleep(hold_seconds)
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        pass
    print(f"[hook-scene] held visible hook scene for {hold_seconds:g}s", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
