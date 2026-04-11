#!/usr/bin/env python3
"""Run defuddle segment 3: visible Rust crate consumer in a real terminal."""
import json
import os
import shutil
import signal
import subprocess
import sys
import tempfile
import time
from pathlib import Path


ROOT_DIR = Path(__file__).resolve().parents[2]
DEMO_URL = os.environ.get(
    "DEFUDDLE_DEMO_URL",
    "https://sqlite.org/forum/forumpost/40a2358ad9241700?",
)


def _check_prerequisites() -> list[str]:
    errors = []
    if not os.environ.get("DISPLAY"):
        errors.append("DISPLAY is not set")
    if not shutil.which("cargo"):
        errors.append("cargo not installed")
    if not shutil.which("xdotool"):
        errors.append("xdotool not installed")
    if not shutil.which("xfce4-terminal"):
        errors.append("xfce4-terminal not installed")
    if not (ROOT_DIR / "Cargo.toml").is_file():
        errors.append(f"Cargo.toml not found: {ROOT_DIR / 'Cargo.toml'}")
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
        print("[rust-scene] PREREQUISITES FAILED:", file=sys.stderr)
        for error in errors:
            print(f"  - {error}", file=sys.stderr)
        return 1

    hold_seconds = float(os.environ.get("DEMO_RUST_HOLD_SECONDS", "12"))
    run_token = str(int(time.time()))
    terminal_title = f"defuddle Rust {run_token}"
    done_file = Path(f"/tmp/defuddle-rust-scene-done-{run_token}")
    log_file = Path(f"/tmp/defuddle-rust-scene-{run_token}.log")
    temp_dir = Path(tempfile.mkdtemp(prefix="defuddle-rust-scene-", dir="/tmp"))
    src_dir = temp_dir / "src"
    src_dir.mkdir(parents=True, exist_ok=True)
    done_file.unlink(missing_ok=True)

    cargo_toml = f"""\
[package]
name = "defuddle-demo-rust-scene"
version = "0.1.0"
edition = "2024"

[dependencies]
defuddle-rs = {{ path = "{ROOT_DIR}" }}
tokio = {{ version = "1", features = ["rt", "macros"] }}
"""

    main_rs = f"""\
use defuddle_rs::Defuddle;

#[tokio::main(flavor = "current_thread")]
async fn main() {{
    let url = {json.dumps(DEMO_URL)};
    let result = Defuddle::fetch_and_parse(url).await.expect("fetch_and_parse failed");

    println!("Title: {{}}", result.title);
    println!("Site: {{}}", result.site.unwrap_or_else(|| "-".to_string()));
    println!("Author: {{}}", result.author.unwrap_or_else(|| "-".to_string()));
    println!("Word count: {{}}", result.word_count);
    println!();
    println!("Markdown preview:");

    for line in result.content_markdown.lines().filter(|line| !line.trim().is_empty()).take(8) {{
        println!("{{}}", line);
    }}
}}
"""

    script_path = temp_dir / "run.sh"
    script_path.write_text(
        "#!/usr/bin/env bash\n"
        "set -euo pipefail\n"
        f"printf '\\033]0;{terminal_title}\\007'\n"
        "clear\n"
        "echo 'Rust crate consumer'\n"
        f"echo 'URL: {DEMO_URL}'\n"
        "echo\n"
        f"cd {temp_dir}\n"
        "echo '$ cargo run --quiet'\n"
        "echo\n"
        f"cargo run --quiet 2>&1 | tee {log_file}\n"
        "echo\n"
        f"sleep {hold_seconds}\n"
        f"touch {done_file}\n",
        encoding="utf-8",
    )
    script_path.chmod(0o755)
    (temp_dir / "Cargo.toml").write_text(cargo_toml, encoding="utf-8")
    (src_dir / "main.rs").write_text(main_rs, encoding="utf-8")

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
            f"exec {script_path}",
        ],
        cwd=ROOT_DIR,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        start_new_session=True,
    )

    try:
        window_id = _wait_for_window_by_title(terminal_title, timeout=30.0)
        if not window_id:
            print(f"[rust-scene] terminal window not found for title={terminal_title!r}", file=sys.stderr)
            return 1

        subprocess.run(
            ["xdotool", "windowactivate", "--sync", window_id],
            check=False,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        print(
            f"[rust-scene] visible window_id={window_id} pid={process.pid} title={terminal_title!r} url={DEMO_URL}",
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
                    print(f"[rust-scene] terminal output:\n{preview}", file=sys.stderr)
            print("[rust-scene] timed out waiting for Rust scene completion", file=sys.stderr)
            return 1

        print(f"[rust-scene] held visible Rust scene for {hold_seconds:g}s", file=sys.stderr)
        return 0
    finally:
        try:
            os.killpg(process.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        done_file.unlink(missing_ok=True)


if __name__ == "__main__":
    raise SystemExit(main())
