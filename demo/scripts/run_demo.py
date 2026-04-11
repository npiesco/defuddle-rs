#!/usr/bin/env python3
"""Gate runner for the defuddle demo."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path


ROOT_DIR = Path(__file__).resolve().parents[2]
DEMO_DIR = ROOT_DIR / "demo"
SCRIPTS_DIR = DEMO_DIR / "scripts"
OUTPUT_DIR = DEMO_DIR / "output"
TIMELINE_PATH = OUTPUT_DIR / "timeline.json"
MCP_CONFIG = Path.home() / ".copilot" / "mcp-config.json"
COPILOT_CONFIG = Path.home() / ".copilot" / "config.json"

DEMO_URL = os.environ.get(
    "DEFUDDLE_DEMO_URL",
    "https://sqlite.org/forum/forumpost/40a2358ad9241700?",
)

SCENES: list[tuple[str, str, Path, str]] = [
    ("seg_01", "Hook", SCRIPTS_DIR / "record_cli_scene.py", "Live SQLite page hook"),
    ("seg_02", "Browser Extension", SCRIPTS_DIR / "record_webapp_scene.py", "Real installed extension side panel"),
    ("seg_03", "Rust Crate", SCRIPTS_DIR / "record_tauri_scene.py", "Visible Rust crate consumer"),
    ("seg_04", "Python Bindings", SCRIPTS_DIR / "record_vscode_scene.py", "Visible Python bindings consumer"),
    ("seg_05", "MCP", SCRIPTS_DIR / "launch_and_seed_terminal.py", "Visible Copilot + MCP scene"),
    ("seg_06", "Close", SCRIPTS_DIR / "record_close_scene.py", "Final comparison frame"),
]


@dataclass
class PhaseResult:
    name: str
    passed: bool
    detail: str
    duration_seconds: float


def run(args: list[str], timeout: int = 120) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        args,
        cwd=str(ROOT_DIR),
        text=True,
        capture_output=True,
        timeout=timeout,
        check=False,
    )


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def phase_python_and_tools() -> str:
    require(os.environ.get("DISPLAY"), "DISPLAY is not set")
    for command in ["python3", "ffprobe", "ffmpeg", "xdotool"]:
        require(shutil.which(command) is not None, f"missing command: {command}")
    return "python, display, and media tools are available"


def phase_copilot_config() -> str:
    require(COPILOT_CONFIG.is_file(), f"missing Copilot config: {COPILOT_CONFIG}")
    require(MCP_CONFIG.is_file(), f"missing Copilot MCP config: {MCP_CONFIG}")

    config = json.loads(COPILOT_CONFIG.read_text())
    trusted = config.get("trusted_folders", [])
    require(str(ROOT_DIR) in trusted, f"{ROOT_DIR} is not trusted in Copilot config")

    mcp = json.loads(MCP_CONFIG.read_text())
    servers = mcp.get("mcpServers", {})
    require("defuddle" in servers, "defuddle MCP server missing from Copilot config")
    return "Copilot trust and defuddle MCP config are in place"


def phase_scene_scripts() -> str:
    missing = [str(path) for _, _, path, _ in SCENES if not path.is_file()]
    require(not missing, f"missing scene scripts: {', '.join(missing)}")
    return "all six defuddle scene scripts are present"


def phase_output_contract() -> str:
    expected = [
        "seg_01_hook.mp4",
        "seg_02_extension.mp4",
        "seg_03_rust.mp4",
        "seg_04_python.mp4",
        "seg_05_mcp.mp4",
        "seg_06_close.mp4",
        "defuddle_demo_silent.mp4",
        "defuddle_demo_final.mp4",
    ]
    return "expected outputs: " + ", ".join(expected)


PHASES = [
    ("python-tools", phase_python_and_tools),
    ("copilot-config", phase_copilot_config),
    ("scene-scripts", phase_scene_scripts),
    ("output-contract", phase_output_contract),
]


def run_gates() -> list[PhaseResult]:
    results: list[PhaseResult] = []
    for name, fn in PHASES:
        started = time.time()
        try:
            detail = fn()
            passed = True
        except Exception as exc:
            detail = str(exc)
            passed = False
        results.append(PhaseResult(name, passed, detail, time.time() - started))
    return results


def write_timeline(results: list[PhaseResult]) -> None:
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    payload = {
        "generatedAt": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "demoUrl": DEMO_URL,
        "scenes": [
            {"id": sid, "label": label, "script": str(path), "notes": notes}
            for sid, label, path, notes in SCENES
        ],
        "gates": [
            {
                "name": result.name,
                "passed": result.passed,
                "detail": result.detail,
                "durationSeconds": round(result.duration_seconds, 3),
            }
            for result in results
        ],
    }
    TIMELINE_PATH.write_text(json.dumps(payload, indent=2) + "\n")


def launch_visible_dry_run_terminal() -> int:
    command = (
        "printf '\\nVisible defuddle dry run complete. Press Enter to close...\\n'; "
        "read -r _"
    )
    result = subprocess.run(
        [
            sys.executable,
            str(SCRIPTS_DIR / "open_agent_terminal.py"),
            "bash",
            "-lc",
            command,
        ],
        cwd=str(ROOT_DIR),
        check=False,
    )
    return result.returncode


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run defuddle demo gates and emit the timeline scaffold.")
    parser.add_argument("--dry-run", action="store_true", help="Validate the defuddle demo scaffold only.")
    parser.add_argument(
        "--visible-terminal",
        action="store_true",
        help="Open a visible terminal after the dry run so the operator can verify the launch path.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    print("=== defuddle demo gate runner ===")
    results = run_gates()
    write_timeline(results)

    failed = [result for result in results if not result.passed]
    for result in results:
        status = "PASS" if result.passed else "FAIL"
        print(f"[{status}] {result.name}: {result.detail} ({result.duration_seconds:.2f}s)")

    print(f"\nTimeline: {TIMELINE_PATH}")
    if failed:
        return 1

    if args.visible_terminal:
        return launch_visible_dry_run_terminal()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
