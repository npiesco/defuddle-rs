"""
Candycam recording helper for the fsgdb demo.

This is intentionally kept very close to the Duckcells helper so the same
record-window-by-PID workflow can be reused for terminal, VS Code, and agent
surfaces.
"""
import ctypes
import ctypes.wintypes
import io
import os
import subprocess
import sys
import time

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding="utf-8", errors="replace")

os.environ["CANDYCAM_BACKEND"] = os.environ.get("CANDYCAM_BACKEND", "xcap")

from capture import DemoRecorder, QualityPreset  # type: ignore

_QUALITY = QualityPreset.SCREEN_SHARE


def _validate_mp4(path: str) -> bool:
    try:
        result = subprocess.run(
            ["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", path],
            capture_output=True,
            text=True,
            timeout=10,
        )
        if result.returncode != 0:
            return False
        duration = result.stdout.strip()
        return duration != "" and float(duration) > 0
    except Exception:
        return False


def _wait_for_valid_mp4(path: str, timeout: float = 10.0) -> bool:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if os.path.exists(path) and os.path.getsize(path) > 0 and _validate_mp4(path):
            return True
        time.sleep(0.25)
    return False


def _get_hwnds_for_pid(target_pid: int) -> list[tuple[int, str]]:
    results: list[tuple[int, str]] = []

    def cb(hwnd, _):
        pid = ctypes.wintypes.DWORD()
        ctypes.windll.user32.GetWindowThreadProcessId(hwnd, ctypes.byref(pid))
        if pid.value == target_pid:
            length = ctypes.windll.user32.GetWindowTextLengthW(hwnd)
            if length > 0:
                buf = ctypes.create_unicode_buffer(length + 1)
                ctypes.windll.user32.GetWindowTextW(hwnd, buf, length + 1)
                title = buf.value
                if "IME" not in title and "MSCTFIME" not in title:
                    results.append((hwnd, title))
        return True

    WNDENUMPROC = ctypes.WINFUNCTYPE(
        ctypes.wintypes.BOOL, ctypes.wintypes.HWND, ctypes.wintypes.LPARAM
    )
    ctypes.windll.user32.EnumWindows(WNDENUMPROC(cb), 0)
    return results


def _find_candycam_window_for_pid(recorder: DemoRecorder, target_pid: int):
    hwnds = _get_hwnds_for_pid(target_pid)
    if not hwnds:
        return None

    window_infos = recorder.list_window_info()
    candycam_ids = {w.id: w.title for w in window_infos}

    for hwnd, _title in hwnds:
        if hwnd in candycam_ids:
            return (hwnd, candycam_ids[hwnd])
    return None


def main() -> None:
    recorder = DemoRecorder()
    current_path: str | None = None
    recording = False

    print("READY", flush=True)

    for line in sys.stdin:
        cmd = line.strip()
        if not cmd:
            continue

        if cmd == "LIST_WINDOWS":
            titles = recorder.list_windows()
            print(f"WINDOWS: {' | '.join(titles)}", flush=True)

        elif cmd.startswith("RECORD_WINDOW "):
            if recording:
                recorder.stop_recording()
                recording = False

            parts = cmd.split(" ", 2)
            if len(parts) < 3:
                print("ERROR: RECORD_WINDOW <path> <title>", flush=True)
                continue

            path, title = parts[1], parts[2]
            current_path = path
            all_titles = recorder.list_windows()
            matched = [t for t in all_titles if title.lower() in t.lower()]
            if not matched:
                print(f"ERROR: No window matching '{title}'. Visible windows: {' | '.join(all_titles)}", flush=True)
                current_path = None
                continue

            try:
                recorder.start_recording_window_with_quality(path, title, _QUALITY)
                recording = True
                print(f"RECORDING {path} (matched: {matched[0]})", flush=True)
            except Exception as exc:
                print(f"ERROR: {exc}", flush=True)

        elif cmd.startswith("RECORD_WINDOW_PID "):
            if recording:
                recorder.stop_recording()
                recording = False

            parts = cmd.split(" ", 2)
            if len(parts) < 3:
                print("ERROR: RECORD_WINDOW_PID <path> <pid>", flush=True)
                continue

            path = parts[1]
            try:
                target_pid = int(parts[2])
            except ValueError:
                print(f"ERROR: Invalid PID: {parts[2]}", flush=True)
                continue

            current_path = path
            match = _find_candycam_window_for_pid(recorder, target_pid)
            if match is None:
                hwnds = _get_hwnds_for_pid(target_pid)
                print(f"ERROR: No candycam window for PID {target_pid}. OS HWNDs: {hwnds}", flush=True)
                current_path = None
                continue

            window_id, window_title = match
            try:
                recorder.start_recording_window_by_id_with_quality(path, window_id, _QUALITY)
                recording = True
                print(f"RECORDING {path} (pid={target_pid}, hwnd={window_id}, title={window_title})", flush=True)
            except Exception as exc:
                print(f"ERROR: {exc}", flush=True)
                current_path = None

        elif cmd == "STOP":
            if recording:
                recorder.stop_recording()
                recording = False
                if current_path and _wait_for_valid_mp4(current_path):
                    size = os.path.getsize(current_path)
                    print(f"STOPPED {current_path} ({size} bytes, valid)", flush=True)
                elif current_path:
                    size = os.path.getsize(current_path) if os.path.exists(current_path) else 0
                    print(f"ERROR STOP segment corrupt: {current_path} ({size} bytes, NO moov atom)", flush=True)
                else:
                    print("STOPPED (no path)", flush=True)
                current_path = None
            else:
                print("STOPPED (not recording)", flush=True)

        elif cmd == "QUIT":
            if recording:
                recorder.stop_recording()
                recording = False
            print("QUIT", flush=True)
            return

        else:
            print(f"ERROR: Unknown command: {cmd}", flush=True)


if __name__ == "__main__":
    main()
