#!/usr/bin/env python3
"""Record the six canonical defuddle demo segments."""

from __future__ import annotations

import os
import subprocess
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path.home() / "candycam" / "bindings" / "python"))
from capture import DemoRecorder, QualityPreset  # type: ignore  # noqa: E402


ROOT_DIR = Path(__file__).resolve().parents[2]
SCRIPTS_DIR = ROOT_DIR / "demo" / "scripts"
OUTPUT_DIR = ROOT_DIR / "demo" / "output"

SCENES: list[tuple[str, Path, str]] = [
    ("seg_01_hook.mp4", SCRIPTS_DIR / "record_hook_scene.py", "SQLite User Forum: absurder-sql"),
    ("seg_02_extension.mp4", SCRIPTS_DIR / "record_extension_scene.py", "SQLite User Forum: absurder-sql"),
    ("seg_03_rust.mp4", SCRIPTS_DIR / "record_rust_scene.py", "defuddle Rust"),
    ("seg_04_python.mp4", SCRIPTS_DIR / "record_python_scene.py", "defuddle Python"),
    ("seg_06_close.mp4", SCRIPTS_DIR / "record_close_scene.py", "defuddle demo close"),
]


def run_capture(*args: str) -> str:
    result = subprocess.run(args, check=False, capture_output=True, text=True)
    return result.stdout.strip() if result.returncode == 0 else ""


def wait_for_window(title_substring: str, timeout: float = 30.0) -> str:
    deadline = time.time() + timeout
    while time.time() < deadline:
        output = run_capture("xdotool", "search", "--onlyvisible", "--name", title_substring)
        windows = [line.strip() for line in output.splitlines() if line.strip()]
        if windows:
            return windows[-1]
        time.sleep(0.2)
    return ""


def validate_mp4(path: Path) -> bool:
    result = subprocess.run(
        ["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", str(path)],
        check=False,
        capture_output=True,
        text=True,
        timeout=10,
    )
    if result.returncode != 0:
        return False
    try:
        return float(result.stdout.strip()) > 0
    except ValueError:
        return False


def record_scene(output_name: str, script_path: Path, title_substring: str) -> None:
    output_path = OUTPUT_DIR / output_name
    output_path.unlink(missing_ok=True)

    process = subprocess.Popen(
        [sys.executable, str(script_path)],
        cwd=ROOT_DIR,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
        text=True,
        start_new_session=True,
    )

    try:
        window_title = wait_for_window(title_substring)
        if not window_title:
            raise RuntimeError(f"window not found for {title_substring!r}")

        recorder = DemoRecorder()
        recorder.start_recording_window_with_quality(
            str(output_path),
            title_substring,
            QualityPreset.SCREEN_SHARE,
        )
        recorder.wait_for_first_frame(5000)

        _, stderr = process.communicate(timeout=180)
        recorder.stop_recording()

        if process.returncode != 0:
            raise RuntimeError(stderr.strip() or f"{script_path.name} exited {process.returncode}")
        if not validate_mp4(output_path):
            raise RuntimeError(f"recording invalid: {output_path}")
        print(f"[recorded] {output_name}")
    finally:
        try:
            process.kill()
        except ProcessLookupError:
            pass


def main() -> int:
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    for command in ["xdotool", "ffprobe"]:
        if not run_capture("which", command):
            raise RuntimeError(f"missing required command: {command}")

    for output_name, script_path, title_substring in SCENES:
        record_scene(output_name, script_path, title_substring)

    mcp_output = OUTPUT_DIR / "seg_05_mcp.mp4"
    if not validate_mp4(mcp_output):
        raise RuntimeError("seg_05_mcp.mp4 missing or invalid; rerun the visible MCP scene first")
    print("[ok] canonical defuddle segments ready")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
