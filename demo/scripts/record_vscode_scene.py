#!/usr/bin/env python3
"""Run defuddle segment 4: visible Python bindings consumer in a real terminal."""
import os
import shutil
import signal
import subprocess
import sys
import tempfile
import time
from pathlib import Path


ROOT_DIR = Path(__file__).resolve().parents[2]
PY_BINDINGS_DIR = ROOT_DIR / "bindings" / "python"
DEMO_URL = os.environ.get(
    "DEFUDDLE_DEMO_URL",
    "https://sqlite.org/forum/forumpost/40a2358ad9241700?",
)


def _check_prerequisites() -> list[str]:
    errors = []
    if not os.environ.get("DISPLAY"):
        errors.append("DISPLAY is not set")
    if not shutil.which("python3"):
        errors.append("python3 not installed")
    if not shutil.which("xdotool"):
        errors.append("xdotool not installed")
    if not shutil.which("xfce4-terminal"):
        errors.append("xfce4-terminal not installed")
    if not (PY_BINDINGS_DIR / "defuddle" / "__init__.py").is_file():
        errors.append(f"Python bindings package not found: {PY_BINDINGS_DIR / 'defuddle' / '__init__.py'}")
    if not (PY_BINDINGS_DIR / "defuddle" / "libdefuddle_rs.so").is_file():
        errors.append(f"Python bindings library not found: {PY_BINDINGS_DIR / 'defuddle' / 'libdefuddle_rs.so'}")
    return errors


def _run_capture(*args: str) -> str:
    result = subprocess.run(args, check=False, capture_output=True, text=True)
    return result.stdout.strip() if result.returncode == 0 else ""


def _wait_for_window_by_title(title: str, timeout: float = 20.0) -> str:
    deadline = time.time() + timeout
    while time.time() < deadline:
        output = _run_capture("xdotool", "search", "--onlyvisible", "--name", title)
        windows = [w.strip() for w in output.splitlines() if w.strip()]
        if windows:
            return windows[-1]
        time.sleep(0.2)
    return ""


def main() -> int:
    errors = _check_prerequisites()
    if errors:
        print("[python-scene] PREREQUISITES FAILED:", file=sys.stderr)
        for error in errors:
            print(f"  - {error}", file=sys.stderr)
        return 1

    hold_seconds = float(os.environ.get("DEMO_PYTHON_HOLD_SECONDS", "12"))
    run_token = str(int(time.time()))
    terminal_title = f"defuddle Python {run_token}"
    done_file = Path(f"/tmp/defuddle-python-scene-done-{run_token}")
    log_file = Path(f"/tmp/defuddle-python-scene-{run_token}.log")
    temp_dir = Path(tempfile.mkdtemp(prefix="defuddle-python-scene-", dir="/tmp"))
    done_file.unlink(missing_ok=True)

    script_path = temp_dir / "demo.py"
    script_path.write_text(
        "from defuddle import DefuddleParser\n"
        f'url = "{DEMO_URL}"\n'
        "parser = DefuddleParser()\n"
        "result = parser.fetch_and_parse_url(url)\n"
        'print(f"Title: {result.title}")\n'
        'print(f"Site: {result.site or \'-\'}")\n'
        'print(f"Author: {result.author or \'-\'}")\n'
        'print(f"Word count: {result.word_count}")\n'
        "print()\n"
        'print("Markdown preview:")\n'
        "for line in [line for line in result.content_markdown.splitlines() if line.strip()][:8]:\n"
        "    print(line)\n",
        encoding="utf-8",
    )

    runner_path = temp_dir / "run.sh"
    runner_path.write_text(
        "#!/usr/bin/env bash\n"
        "set -euo pipefail\n"
        f"printf '\\033]0;{terminal_title}\\007'\n"
        "clear\n"
        "echo 'Python bindings consumer'\n"
        f"echo 'URL: {DEMO_URL}'\n"
        "echo\n"
        f"cd {temp_dir}\n"
        f"export PYTHONPATH={PY_BINDINGS_DIR}\n"
        "echo '$ python3 demo.py'\n"
        "echo\n"
        f"python3 demo.py 2>&1 | tee {log_file}\n"
        "echo\n"
        f"sleep {hold_seconds}\n"
        f"touch {done_file}\n",
        encoding="utf-8",
    )
    runner_path.chmod(0o755)

    process = subprocess.Popen(
        [
            "xfce4-terminal",
            "--disable-server",
            f"--working-directory={temp_dir}",
            f"--title={terminal_title}",
            "-x",
            "bash",
            "--noprofile",
            "--norc",
            "-c",
            f"exec {runner_path}",
        ],
        cwd=ROOT_DIR,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        start_new_session=True,
    )

    try:
        window_id = _wait_for_window_by_title(terminal_title, timeout=30.0)
        if not window_id:
            print(f"[python-scene] terminal window not found for title={terminal_title!r}", file=sys.stderr)
            return 1

        subprocess.run(
            ["xdotool", "windowactivate", "--sync", window_id],
            check=False,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        print(
            f"[python-scene] visible window_id={window_id} pid={process.pid} title={terminal_title!r} url={DEMO_URL}",
            file=sys.stderr,
        )

        deadline = time.time() + 180.0
        while time.time() < deadline:
            if done_file.is_file():
                break
            time.sleep(0.5)
        else:
            if log_file.is_file():
                preview = log_file.read_text(errors="ignore")[-1200:]
                if preview.strip():
                    print(f"[python-scene] terminal output:\n{preview}", file=sys.stderr)
            print("[python-scene] timed out waiting for Python scene completion", file=sys.stderr)
            return 1

        print(f"[python-scene] held visible Python scene for {hold_seconds:g}s", file=sys.stderr)
        return 0
    finally:
        try:
            os.killpg(process.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        done_file.unlink(missing_ok=True)


if __name__ == "__main__":
    raise SystemExit(main())
