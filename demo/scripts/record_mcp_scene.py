#!/usr/bin/env python3

import os
import subprocess
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path.home() / "candycam" / "bindings" / "python"))
from capture import DemoRecorder, QualityPreset  # noqa: E402

ROOT_DIR = Path(__file__).resolve().parents[2]
START_SCRIPT = ROOT_DIR / "demo" / "scripts" / "start_copilot_session.py"
OPEN_TERMINAL = ROOT_DIR / "demo" / "scripts" / "open_agent_terminal.py"
OUTPUT_DIR = ROOT_DIR / "demo" / "output"
RECORDING_PATH = OUTPUT_DIR / "seg_05_mcp.mp4"
RECORDING_STARTED_AT_PATH = OUTPUT_DIR / "recording_started_at"


def run_capture(*args: str) -> str:
    result = subprocess.run(args, check=False, capture_output=True, text=True)
    if result.returncode != 0:
        return ""
    return result.stdout.strip()


def require_command(name: str) -> None:
    if not run_capture("which", name):
        print(f"missing required command: {name}", file=sys.stderr)
        raise SystemExit(1)


def get_window_pid(window_id: str) -> str:
    return run_capture("xdotool", "getwindowpid", window_id)


def get_window_name(window_id: str) -> str:
    return run_capture("xdotool", "getwindowname", window_id)


def wait_for_target_window(pid: int, timeout_seconds: float) -> str:
    deadline = time.time() + timeout_seconds
    while time.time() < deadline:
        output = run_capture("xdotool", "search", "--onlyvisible", "--pid", str(pid))
        windows = [line.strip() for line in output.splitlines() if line.strip()]
        if windows:
            return windows[-1]
        time.sleep(0.1)
    return ""


def activate_window(window_id: str) -> None:
    subprocess.run(
        ["xdotool", "windowactivate", "--sync", window_id],
        check=False,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )


def capture_window_screenshot(window_id: str, screenshot_path: Path) -> None:
    screenshot_path.parent.mkdir(parents=True, exist_ok=True)
    result = subprocess.run(
        ["import", "-window", window_id, str(screenshot_path)],
        check=False,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
        text=True,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr.strip() or f"failed to capture screenshot for window {window_id}")
    if not screenshot_path.is_file() or screenshot_path.stat().st_size == 0:
        raise RuntimeError(f"screenshot was not written: {screenshot_path}")


def _run_capture(*args: str) -> str:
    result = subprocess.run(args, check=False, capture_output=True, text=True)
    return result.stdout.strip() if result.returncode == 0 else ""


def _wait_for_window_by_title(title: str, timeout: float = 15.0) -> str:
    deadline = time.time() + timeout
    while time.time() < deadline:
        output = _run_capture("xdotool", "search", "--onlyvisible", "--name", title)
        windows = [w.strip() for w in output.splitlines() if w.strip()]
        if windows:
            return windows[-1]
        time.sleep(0.1)
    return ""


def main() -> int:
    require_command("xdotool")
    require_command("uv")

    if "DISPLAY" not in os.environ or not os.environ["DISPLAY"]:
        print("DISPLAY is not set; cannot launch a GUI terminal from this environment.", file=sys.stderr)
        return 1

    if not START_SCRIPT.is_file():
        print(f"start script not found: {START_SCRIPT}", file=sys.stderr)
        return 1

    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    RECORDING_PATH.unlink(missing_ok=True)
    RECORDING_STARTED_AT_PATH.unlink(missing_ok=True)

    run_token = str(int(time.time()))
    terminal_title = f"defuddle Demo {run_token}"
    window_id_file = Path(f"/tmp/defuddle-demo-window-id-{run_token}")
    exit_code_file = Path(f"/tmp/defuddle-demo-exit-code-{run_token}")
    screenshot_path = Path(os.environ.get("DEMO_LAUNCH_SCREENSHOT", "/tmp/defuddle-demo-launch.png"))
    window_id_file.unlink(missing_ok=True)
    exit_code_file.unlink(missing_ok=True)

    subprocess.Popen(
        [
            sys.executable,
            str(OPEN_TERMINAL),
            "bash",
            "-lc",
            (
                f"while [ ! -f {window_id_file} ]; do sleep 0.1; done && "
                f"export DEMO_WINDOW_ID=$(cat {window_id_file}) && "
                "uv run python ./demo/scripts/start_copilot_session.py; "
                f"echo $? > {exit_code_file}; "
                "sleep 2"
            ),
        ],
        cwd=ROOT_DIR,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        start_new_session=True,
        env={**os.environ, "DEMO_VISIBLE_MODE": "1", "DEMO_TERMINAL_TITLE": terminal_title},
    )

    target_window = _wait_for_window_by_title(terminal_title, timeout=15.0)
    if not target_window:
        print(f"unable to detect terminal window '{terminal_title}'", file=sys.stderr)
        return 1

    activate_window(target_window)
    time.sleep(0.5)
    try:
        capture_window_screenshot(target_window, screenshot_path)
    except RuntimeError as exc:
        print(f"[launcher] screenshot warning: {exc}", file=sys.stderr)

    window_id_file.write_text(target_window)
    print(
        f"Launched terminal window {target_window} "
        f"name={get_window_name(target_window) or 'unknown'} "
        f"screenshot={screenshot_path} running {START_SCRIPT}"
    )

    recorder = DemoRecorder()
    recorder.start_recording_window_with_quality(
        str(RECORDING_PATH), terminal_title, QualityPreset.SCREEN_SHARE,
    )
    recorder.wait_for_first_frame(5000)
    RECORDING_STARTED_AT_PATH.write_text(str(time.time()))
    print(f"[launcher] recording started → {RECORDING_PATH}", file=sys.stderr)

    max_wait = float(os.environ.get("DEMO_MAX_RUNTIME_SECONDS", "1800"))
    deadline = time.time() + max_wait
    while time.time() < deadline:
        if exit_code_file.is_file():
            break
        time.sleep(1.0)

    recorder.stop_recording()
    print(f"[launcher] recording stopped", file=sys.stderr)

    if not RECORDING_PATH.is_file() or RECORDING_PATH.stat().st_size == 0:
        print(f"[launcher] recording not produced: {RECORDING_PATH}", file=sys.stderr)
        return 1
    print(f"[launcher] recording saved ({RECORDING_PATH.stat().st_size // 1024} KB): {RECORDING_PATH}")

    if exit_code_file.is_file():
        try:
            return int(exit_code_file.read_text().strip())
        except ValueError:
            return 1
    else:
        print("WARNING: session did not write exit code (timeout?)", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
