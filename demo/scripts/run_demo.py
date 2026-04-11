#!/usr/bin/env python3

import argparse
import json
import os
import shlex
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path


ROOT_DIR = Path(__file__).resolve().parents[2]
DEMO_DIR = ROOT_DIR / "demo"
SCRIPTS_DIR = DEMO_DIR / "scripts"
SERVO_DIR = Path(os.environ.get("DEMO_SERVO_DIR", "/tmp/servo"))
DEMO_DB_DIR = Path(os.environ.get("DEMO_DB_DIR", "/tmp/fsgdb-servo-demo"))
DEMO_DB_PATH = Path(os.environ.get("DEMO_DB_PATH", str(DEMO_DB_DIR / "graph.db")))
SERVO_REPO_URL = os.environ.get("DEMO_SERVO_REPO_URL", "https://github.com/servo/servo")
RESET_SCRIPT = SCRIPTS_DIR / "reset_demo_state.py"
COPILOT_START = SCRIPTS_DIR / "start_copilot_session.py"
GUI_LAUNCHER = SCRIPTS_DIR / "launch_and_seed_terminal.py"
CLI_RECORDER = SCRIPTS_DIR / "record_cli_scene.py"
VSCODE_RECORDER = SCRIPTS_DIR / "record_vscode_scene.py"
WEBAPP_RECORDER = SCRIPTS_DIR / "record_webapp_scene.py"
TAURI_RECORDER = SCRIPTS_DIR / "record_tauri_scene.py"
TTS_SCRIPT = SCRIPTS_DIR / "synthesize_and_mux.py"
OPEN_TERMINAL = SCRIPTS_DIR / "open_agent_terminal.py"
MCP_CONFIG = Path("/home/npiesco/.copilot/mcp-config.json")
COPILOT_CONFIG = Path("/home/npiesco/.copilot/config.json")
FSGDB_BIN = ROOT_DIR / "target" / "release" / "fsgdb"
VSCODE_EXTENSION_DIR = ROOT_DIR / "vscode-extension"

QUERY_SCENE_CMD = (
    f"DEMO_DB_PATH={shlex.quote(str(DEMO_DB_PATH))} "
    f"DEMO_WORKSPACE_PATH={shlex.quote(str(SERVO_DIR))} "
    f"DEMO_FSGDB_BIN={shlex.quote(str(FSGDB_BIN))} "
    'DEMO_VSCODE_GREP="DEMO FLOW: opens Cypher Query Editor" '
    "timeout 180 npx @vscode/test-cli --config .vscode-test-demo.js"
)
GRAPH_SCENE_CMD = (
    f"DEMO_DB_PATH={shlex.quote(str(DEMO_DB_PATH))} "
    f"DEMO_WORKSPACE_PATH={shlex.quote(str(SERVO_DIR))} "
    f"DEMO_FSGDB_BIN={shlex.quote(str(FSGDB_BIN))} "
    'DEMO_VSCODE_GREP="DEMO FLOW: opens Graph Visualization" '
    "timeout 180 npx @vscode/test-cli --config .vscode-test-demo.js"
)
SCAN_CMD = (
    f"{shlex.quote(str(FSGDB_BIN))} --database {shlex.quote(str(DEMO_DB_PATH))} scan {shlex.quote(str(SERVO_DIR))} "
    "--enable-code-analysis --enable-git-analysis --git-max-commits 500 "
    "--exclude 'tests/**' --exclude 'third_party/**' --exclude 'resources/**'"
)
STATS_CMD = f"{shlex.quote(str(FSGDB_BIN))} --database {shlex.quote(str(DEMO_DB_PATH))} stats"
COUNT_FILES_CMD = (
    f"{shlex.quote(str(FSGDB_BIN))} --database {shlex.quote(str(DEMO_DB_PATH))} "
    "query --format json --query \"MATCH (f:File) RETURN count(f) AS files\""
)
TRUSTED_FOLDER = str(ROOT_DIR)


@dataclass
class PhaseResult:
    name: str
    passed: bool
    detail: str
    duration_seconds: float


def run(
    args: list[str],
    *,
    cwd: Path | None = None,
    timeout: int = 120,
    env: dict[str, str] | None = None,
) -> subprocess.CompletedProcess[str]:
    merged_env = os.environ.copy()
    if env:
        merged_env.update(env)
    return subprocess.run(
        args,
        cwd=str(cwd or ROOT_DIR),
        env=merged_env,
        text=True,
        capture_output=True,
        timeout=timeout,
        check=False,
    )



def phase(name: str):
    def decorator(fn):
        fn.phase_name = name
        return fn

    return decorator


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


@phase("reset-demo-state")
def phase_reset_demo_state() -> str:
    result = run(["uv", "run", "python", str(RESET_SCRIPT)], timeout=30)
    require(result.returncode == 0, result.stderr or result.stdout or "demo reset failed")
    require(DEMO_DB_DIR.is_dir(), f"missing demo db dir after reset: {DEMO_DB_DIR}")
    require(not SERVO_DIR.exists(), f"servo dir should have been removed during reset: {SERVO_DIR}")
    return "temporary Servo clone and demo database were reset"



@phase("clone-servo")
def phase_clone_servo() -> str:
    result = run(["git", "clone", SERVO_REPO_URL, str(SERVO_DIR)], timeout=3600)
    require(result.returncode == 0, result.stderr or result.stdout or "git clone failed")
    require((SERVO_DIR / ".git").is_dir(), f"missing cloned git dir: {SERVO_DIR / '.git'}")
    return f"cloned Servo into {SERVO_DIR}"


@phase("scan-servo")
def phase_scan_servo() -> str:
    result = run(["bash", "-lc", SCAN_CMD], timeout=7200)
    require(result.returncode == 0, result.stderr or result.stdout or "fsgdb scan failed")
    require(DEMO_DB_PATH.is_file(), f"missing demo database after scan: {DEMO_DB_PATH}")
    return f"scan completed and wrote {DEMO_DB_PATH}"


@phase("release-db-lock")
def phase_release_db_lock() -> str:
    subprocess.run(["pkill", "-f", str(FSGDB_BIN)], check=False)
    time.sleep(2)
    result = run(["lsof", str(DEMO_DB_PATH)], timeout=10)
    if result.returncode == 0 and result.stdout.strip():
        raise RuntimeError(f"demo database is still locked after pkill:\n{result.stdout}")
    return "no fsgdb processes holding the demo database lock"


@phase("cli-stats")
def phase_cli_stats() -> str:
    result = run(["bash", "-lc", STATS_CMD], timeout=120)
    require(result.returncode == 0, result.stderr or result.stdout or "stats command failed")
    output = result.stdout.strip()
    require(output, "stats output was empty")
    return "stats command succeeded"


@phase("cli-count-files")
def phase_cli_count_files() -> str:
    result = run(["bash", "-lc", COUNT_FILES_CMD], timeout=120)
    require(result.returncode == 0, result.stderr or result.stdout or "count files query failed")
    start = result.stdout.find("{")
    end = result.stdout.rfind("}") + 1
    require(start != -1 and end > start, f"no JSON object in output: {result.stdout[:200]!r}")
    payload = json.loads(result.stdout[start:end])
    rows = payload.get("rows", [])
    require(rows, "count files query returned no rows")
    files_value = rows[0].get("values", {}).get("files")
    require(isinstance(files_value, int) and files_value > 0, f"unexpected file count: {files_value}")
    return f"graph contains {files_value} files"


@phase("build-vscode-extension")
def phase_build_vscode_extension() -> str:
    result = run(["npm", "run", "build"], cwd=VSCODE_EXTENSION_DIR, timeout=1800)
    require(result.returncode == 0, result.stderr or result.stdout or "VS Code extension build failed")
    bundle = VSCODE_EXTENSION_DIR / "media" / "force-graph-bundle.js"
    require(bundle.is_file(), f"missing bundle after build: {bundle}")
    return "VS Code extension built (tsc + webpack)"


@phase("vscode-query-scene")
def phase_vscode_query_scene() -> str:
    result = run(["bash", "-lc", QUERY_SCENE_CMD], cwd=VSCODE_EXTENSION_DIR, timeout=1800)
    require(result.returncode == 0, result.stderr or result.stdout or "VS Code query scene failed")
    combined = f"{result.stdout}\n{result.stderr}"
    require("DEMO FLOW: opens Cypher Query Editor" in combined, "query scene did not run the expected demo flow")
    return "VS Code query scene passed"


@phase("vscode-graph-scene")
def phase_vscode_graph_scene() -> str:
    result = run(["bash", "-lc", GRAPH_SCENE_CMD], cwd=VSCODE_EXTENSION_DIR, timeout=1800)
    require(result.returncode == 0, result.stderr or result.stdout or "VS Code graph scene failed")
    combined = f"{result.stdout}\n{result.stderr}"
    require("DEMO FLOW: opens Graph Visualization" in combined, "graph scene did not run the expected demo flow")
    return "VS Code graph scene passed"


@phase("copilot-mcp-config")
def phase_copilot_mcp_config() -> str:
    require(MCP_CONFIG.is_file(), f"missing Copilot MCP config: {MCP_CONFIG}")
    data = json.loads(MCP_CONFIG.read_text())
    servers = data.get("mcpServers", {})
    fsgdb_server = servers.get("fsgdb")
    require(isinstance(fsgdb_server, dict), "fsgdb MCP server missing from Copilot config")
    require(
        fsgdb_server.get("command") == str(FSGDB_BIN),
        f"fsgdb MCP command is {fsgdb_server.get('command')!r}, expected {FSGDB_BIN!s}",
    )
    args = fsgdb_server.get("args")
    require(isinstance(args, list), "fsgdb MCP args missing from Copilot config")
    require("mcp" in args, "fsgdb MCP config does not run in mcp mode")
    require(str(DEMO_DB_PATH) in args, f"fsgdb MCP config does not point at demo database {DEMO_DB_PATH}")
    return "Copilot config points fsgdb MCP at the release binary and demo database"


@phase("copilot-folder-trust")
def phase_copilot_folder_trust() -> str:
    require(COPILOT_CONFIG.is_file(), f"missing Copilot config: {COPILOT_CONFIG}")
    data = json.loads(COPILOT_CONFIG.read_text())
    trusted_folders = data.get("trusted_folders")
    if trusted_folders is None:
        data["trusted_folders"] = [TRUSTED_FOLDER]
        status = "added"
    else:
        require(isinstance(trusted_folders, list), "trusted_folders is not a list in Copilot config")
        if TRUSTED_FOLDER not in trusted_folders:
            trusted_folders.append(TRUSTED_FOLDER)
            status = "added"
        else:
            status = "already-present"

    COPILOT_CONFIG.write_text(json.dumps(data, indent=2) + "\n")
    verified = json.loads(COPILOT_CONFIG.read_text())
    require(TRUSTED_FOLDER in verified.get("trusted_folders", []), f"{TRUSTED_FOLDER} was not persisted")
    return f"repo trust {status} in Copilot config"


@phase("copilot-start-prompt")
def phase_copilot_start_prompt() -> str:
    content = COPILOT_START.read_text()
    require("Use the fsgdb MCP tools first" in content, "Copilot prompt does not prioritize fsgdb MCP")
    require('"copilot"' in content, "Copilot start script is not launching Copilot")
    require(
        '"-p"' not in content and "'-p'" not in content,
        "Copilot start script regressed to non-interactive -p mode",
    )
    require(
        "type_text(window_id, PROMPT)" in content,
        "Copilot start script does not inject the prompt via xdotool",
    )
    require("--allow-all-tools" in content, "Copilot start script is not suppressing tool approval prompts")
    require("--no-ask-user" in content, "Copilot start script is not suppressing ask-user prompts")
    require("Confirm folder trust" in content, "Copilot start script does not handle folder trust prompts")
    require(
        "demo_done_seen_at" in content,
        "Copilot start script does not define completion detection",
    )
    return "Copilot startup script uses interactive mode with xdotool injection and completion detection"



@phase("gui-launch-prereqs")
def phase_gui_launch_prereqs() -> str:
    require(os.environ.get("DISPLAY"), "DISPLAY is not set")
    for command in ["uv", "xdotool", "import", "script"]:
        result = run(["which", command], timeout=10)
        require(result.returncode == 0, f"missing required GUI/demo command: {command}")
    terminal_ok = any(run(["which", cmd], timeout=10).returncode == 0 for cmd in ["xfce4-terminal", "exo-open"])
    require(terminal_ok, "missing required terminal launcher: xfce4-terminal or exo-open")
    require(GUI_LAUNCHER.is_file(), f"missing launcher: {GUI_LAUNCHER}")
    return f"DISPLAY={os.environ['DISPLAY']} and GUI launcher prerequisites are present"


PHASES = [
    phase_reset_demo_state,
    phase_release_db_lock,
    phase_clone_servo,
    phase_scan_servo,
    phase_cli_stats,
    phase_cli_count_files,
    phase_build_vscode_extension,
    phase_vscode_query_scene,
    phase_vscode_graph_scene,
    phase_copilot_mcp_config,
    phase_copilot_folder_trust,
    phase_copilot_start_prompt,
    phase_gui_launch_prereqs,
]


def run_gates(phases: list = PHASES) -> tuple[bool, list[PhaseResult]]:
    results: list[PhaseResult] = []

    for fn in phases:
        name = fn.phase_name
        print(f"[RUN ] {name}")
        started_at = time.time()
        try:
            detail = fn()
        except Exception as exc:  # noqa: BLE001
            duration_seconds = time.time() - started_at
            results.append(PhaseResult(name=name, passed=False, detail=str(exc), duration_seconds=duration_seconds))
            print(f"[FAIL] {name} ({duration_seconds:.2f}s): {exc}")
            break
        else:
            duration_seconds = time.time() - started_at
            results.append(PhaseResult(name=name, passed=True, detail=detail, duration_seconds=duration_seconds))
            print(f"[PASS] {name} ({duration_seconds:.2f}s): {detail}")

    print("\nGate summary:")
    for result in results:
        status = "PASS" if result.passed else "FAIL"
        print(f"- {status} {result.name} ({result.duration_seconds:.2f}s): {result.detail}")

    if len(results) != len(phases):
        failed = next((result for result in results if not result.passed), None)
        if failed:
            print(f"\nHard stop at phase: {failed.name}", file=sys.stderr)
        return False, results

    print("\nAll gated phases passed.")
    return True, results


def _run_recording_step(name: str, script: Path, extra_env: dict | None = None) -> int:
    print(f"\n[RUN ] {name}")
    started_at = time.time()
    env = {**os.environ, **(extra_env or {})}
    result = subprocess.run(
        ["uv", "run", "python", str(script)],
        cwd=str(ROOT_DIR), text=True, env=env, check=False,
    )
    duration_seconds = time.time() - started_at
    if result.returncode != 0:
        print(f"[FAIL] {name} ({duration_seconds:.2f}s): exited {result.returncode}")
        return result.returncode
    print(f"[PASS] {name} ({duration_seconds:.2f}s)")
    return 0


def launch_demo_session() -> int:
    rc = _run_recording_step("record-cli-scene", CLI_RECORDER)
    if rc != 0:
        return rc

    rc = _run_recording_step("record-vscode-scene", VSCODE_RECORDER)
    if rc != 0:
        return rc

    rc = _run_recording_step("record-webapp-scene", WEBAPP_RECORDER)
    if rc != 0:
        return rc

    rc = _run_recording_step("record-tauri-scene", TAURI_RECORDER)
    if rc != 0:
        return rc

    rc = _run_recording_step(
        "record-copilot-scene",
        GUI_LAUNCHER,
        extra_env={
            "DEMO_VISIBLE_MODE": "1",
            "DEMO_START_DELAY_SECONDS": os.environ.get("DEMO_START_DELAY_SECONDS", "4"),
            "DEMO_PROMPT_DELAY_SECONDS": os.environ.get("DEMO_PROMPT_DELAY_SECONDS", "3"),
        },
    )
    if rc != 0:
        return rc

    print("\n[RUN ] synthesize-and-mux")
    started_at = time.time()
    tts_result = subprocess.run(
        ["uv", "run", "--with", "azure-cognitiveservices-speech", "python", str(TTS_SCRIPT)],
        cwd=str(ROOT_DIR), text=True, check=False,
    )
    duration_seconds = time.time() - started_at
    if tts_result.returncode != 0:
        print(f"[FAIL] synthesize-and-mux ({duration_seconds:.2f}s): exited {tts_result.returncode}")
        return tts_result.returncode
    print(f"[PASS] synthesize-and-mux ({duration_seconds:.2f}s)")
    return 0


def launch_demo_session_in_current_terminal() -> int:
    print("\n[RUN ] launch-demo-session-in-current-terminal")
    started_at = time.time()
    result = subprocess.run(
        ["uv", "run", "python", str(COPILOT_START)],
        cwd=str(ROOT_DIR),
        text=True,
        env={
            **os.environ,
            "DEMO_VISIBLE_MODE": "1",
            "DEMO_START_DELAY_SECONDS": os.environ.get("DEMO_START_DELAY_SECONDS", "4"),
            "DEMO_PROMPT_DELAY_SECONDS": os.environ.get("DEMO_PROMPT_DELAY_SECONDS", "3"),
        },
        check=False,
    )
    duration_seconds = time.time() - started_at
    if result.returncode != 0:
        print(f"[FAIL] launch-demo-session-in-current-terminal ({duration_seconds:.2f}s): exited {result.returncode}")
        return result.returncode
    print(f"[PASS] launch-demo-session-in-current-terminal ({duration_seconds:.2f}s)")
    return 0


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


def launch_visible_dry_run_terminal() -> int:
    if not os.environ.get("DISPLAY"):
        print("DISPLAY is not set; cannot hand off dry run to a visible terminal.", file=sys.stderr)
        return 1

    run_token = str(int(time.time()))
    terminal_title = f"fsgdb Demo {run_token}"
    window_id_file = Path(f"/tmp/fsgdb-demo-window-id-{run_token}")
    exit_code_file = Path(f"/tmp/fsgdb-drydemo-exit-code-{run_token}")
    window_id_file.unlink(missing_ok=True)
    exit_code_file.unlink(missing_ok=True)

    # Terminal waits for the window ID file to be written by us before starting run_demo.
    process = subprocess.Popen(
        [
            "uv", "run", "python",
            str(OPEN_TERMINAL),
            "bash",
            "-lc",
            (
                f"cd {shlex.quote(str(ROOT_DIR))} && "
                f"while [ ! -f {window_id_file} ]; do sleep 0.1; done && "
                f"export DEMO_WINDOW_ID=$(cat {window_id_file}) && "
                "DEMO_VISIBLE_DRY_RUN=1 "
                "uv run python ./demo/scripts/run_demo.py --dry-run; "
                f"echo $? > {exit_code_file}; "
                "printf '\\nDry run terminal finished. Press Enter to close...\\n'; "
                "read -r _; "
                "exec bash"
            ),
        ],
        cwd=str(ROOT_DIR),
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        start_new_session=True,
        env={**os.environ, "DEMO_TERMINAL_TITLE": terminal_title},
    )
    time.sleep(0.5)

    # Find this specific terminal window by its unique title
    window_id = _wait_for_window_by_title(terminal_title)
    if not window_id:
        print(f"[FAIL] could not find terminal window with title {terminal_title!r}", file=sys.stderr)
        process.terminate()
        return 1

    # Activate and write the confirmed window ID so the terminal can proceed
    subprocess.run(
        ["xdotool", "windowactivate", "--sync", window_id],
        check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )
    window_id_file.write_text(window_id)
    print(f"[launcher] terminal window_id={window_id}")

    # Wait for session to complete
    max_wait = float(os.environ.get("DEMO_MAX_RUNTIME_SECONDS", "1800"))
    deadline = time.time() + max_wait
    while time.time() < deadline:
        if exit_code_file.is_file():
            break
        time.sleep(1.0)

    if exit_code_file.is_file():
        try:
            return int(exit_code_file.read_text().strip())
        except ValueError:
            return 1
    else:
        print("[FAIL] launch-visible-dry-run-terminal: session timed out", file=sys.stderr)
        return 1


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run the fsgdb demo flow.")
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Run the full gated flow, then launch the visible Copilot session (no recording).",
    )
    parser.add_argument(
        "--current-terminal",
        action="store_true",
        help="Run the visible Copilot session in the current terminal instead of launching a new window.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()

    if args.dry_run and os.environ.get("DEMO_VISIBLE_DRY_RUN") != "1":
        return launch_visible_dry_run_terminal()

    passed, _results = run_gates()
    if not passed:
        return 1

    if args.dry_run:
        print("\nDry run mode: executing full flow without recording.")
        return launch_demo_session_in_current_terminal()

    if args.current_terminal:
        return launch_demo_session_in_current_terminal()

    return launch_demo_session()


if __name__ == "__main__":
    raise SystemExit(main())
