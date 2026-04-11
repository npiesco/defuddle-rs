#!/usr/bin/env python3

import json
import os
import re
import shlex
import signal
import subprocess
import sys
import tempfile
import time
from pathlib import Path


ROOT_DIR = Path(__file__).resolve().parents[2]
DEMO_URL = os.environ.get("DEFUDDLE_DEMO_URL", "https://sqlite.org/forum/forumpost/40a2358ad9241700?")
TIMINGS_PATH = ROOT_DIR / "demo" / "output" / "timings.json"
PROMPT = (
    f"We are demonstrating defuddle on {DEMO_URL}. "
    "Use the defuddle MCP tools first. "
    "Show the extraction result through the MCP surface, focusing on the page title, site, word count, and a short markdown preview. "
    "Keep the interaction visible and direct. "
    "Do not edit files and do not ask questions."
)
ANSI_RE = re.compile(r"\x1b\[[0-9;?]*[ -/]*[@-~]")
COPILOT_CONFIG = Path.home() / ".copilot" / "config.json"


def strip_ansi(text: str) -> str:
    return ANSI_RE.sub("", text).replace("\r", "")


def visible_prelude() -> None:
    start_delay = float(os.environ.get("DEMO_START_DELAY_SECONDS", "4"))
    prompt_delay = float(os.environ.get("DEMO_PROMPT_DELAY_SECONDS", "3"))
    print()
    print("defuddle MCP demo session")
    print(f"Repo: {ROOT_DIR}")
    print(f"URL: {DEMO_URL}")
    print("Mode: visible")
    print(f"Starting Copilot in {start_delay:g}s...")
    time.sleep(start_delay)
    print()
    print("Task for Copilot:")
    print(PROMPT)
    print()
    print(f"Launching agent in {prompt_delay:g}s...")
    time.sleep(prompt_delay)
    print()


def run_capture(*args: str) -> str:
    result = subprocess.run(args, check=False, capture_output=True, text=True)
    if result.returncode != 0:
        return ""
    return result.stdout.strip()


def get_window_id() -> str:
    configured = os.environ.get("DEMO_WINDOW_ID", "").strip()
    if configured:
        return configured
    if not os.environ.get("DISPLAY"):
        return ""
    return run_capture("xdotool", "getactivewindow")


def send_keys(window_id: str, *keys: str) -> None:
    if not window_id:
        return
    subprocess.run(
        ["xdotool", "windowactivate", "--sync", window_id],
        check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )
    subprocess.run(
        ["xdotool", "key", "--clearmodifiers", *keys],
        check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )


def type_text(window_id: str, text: str) -> None:
    if not window_id:
        return
    subprocess.run(
        ["xdotool", "windowactivate", "--sync", window_id],
        check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )
    lines = text.split("\n")
    for index, chunk in enumerate(lines):
        subprocess.run(
            ["xdotool", "type", "--delay", "1", "--clearmodifiers", chunk],
            check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
        )
        if index < len(lines) - 1:
            subprocess.run(
                ["xdotool", "key", "--clearmodifiers", "Shift+Return"],
                check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
            )


def terminate_process_group(process: subprocess.Popen[bytes], sig: int) -> None:
    try:
        os.killpg(process.pid, sig)
    except ProcessLookupError:
        return


def ensure_folder_trust() -> None:
    folder = str(ROOT_DIR)
    try:
        data = json.loads(COPILOT_CONFIG.read_text()) if COPILOT_CONFIG.is_file() else {}
    except (json.JSONDecodeError, OSError):
        data = {}
    trusted = data.get("trusted_folders", [])
    if not isinstance(trusted, list):
        trusted = []
    if folder not in trusted:
        trusted.append(folder)
        data["trusted_folders"] = trusted
        COPILOT_CONFIG.parent.mkdir(parents=True, exist_ok=True)
        COPILOT_CONFIG.write_text(json.dumps(data, indent=2) + "\n")
        print(f"[controller] added {folder} to Copilot trusted_folders", file=sys.stderr)
    else:
        print("[controller] folder already trusted", file=sys.stderr)


def main() -> int:
    os.chdir(ROOT_DIR)
    ensure_folder_trust()

    if os.environ.get("DEMO_VISIBLE_MODE") == "1":
        visible_prelude()

    max_runtime_seconds = float(os.environ.get("DEMO_MAX_RUNTIME_SECONDS", "1800"))
    idle_nudge_seconds = float(os.environ.get("DEMO_IDLE_NUDGE_SECONDS", "60"))
    window_id = get_window_id()

    copilot_command = [
        "copilot",
        "--screen-reader",
        "--no-mouse",
        "--allow-all-tools",
        "--allow-all-paths",
        "--allow-all-urls",
        "--no-ask-user",
        "--add-dir", str(ROOT_DIR),
    ]
    with tempfile.NamedTemporaryFile(prefix="defuddle-copilot-", suffix=".log", delete=False) as handle:
        log_path = Path(handle.name)
    keep_log = os.environ.get("DEMO_KEEP_LOG") == "1"
    if keep_log:
        print(f"[controller] keeping transcript at {log_path}", file=sys.stderr)

    script_command = [
        "script",
        "-qefc",
        shlex.join(copilot_command),
        str(log_path),
    ]
    process = subprocess.Popen(
        script_command,
        cwd=str(ROOT_DIR),
        env={**os.environ},
        start_new_session=True,
    )

    start_time = time.time()
    timings: dict[str, float] = {}

    def record_timing(event: str) -> None:
        if event not in timings:
            timings[event] = time.time() - start_time
            TIMINGS_PATH.parent.mkdir(parents=True, exist_ok=True)
            TIMINGS_PATH.write_text(json.dumps(timings, indent=2) + "\n")
            print(f"\n[controller] timing:{event}={timings[event]:.2f}s", file=sys.stderr)

    last_seen_len = 0
    last_output_at = start_time
    trusted_sent = False
    trust_answer_pos = 0
    generic_confirm_sent = 0
    prompt_attempts = 0
    prompt_last_sent_at = 0.0
    prompt_search_start_pos = 0
    prompt_sent = False
    prompt_echo_end_pos = 0
    copilot_responded = False
    response_start_pos = 0
    finish_nudge_sent = False
    nudge_sent = 0
    nudge_echo_pos = 0
    mcp_ever_seen = False
    demo_done_seen_at: float | None = None
    exit_sent = False
    recent = ""

    try:
        while True:
            if process.poll() is not None:
                break

            now = time.time()
            if now - start_time > max_runtime_seconds:
                terminate_process_group(process, signal.SIGINT)
                print("\n[controller] timeout; sent SIGINT", file=sys.stderr)
                break

            if log_path.exists():
                content = log_path.read_text(errors="ignore")
                if len(content) > last_seen_len:
                    appended = content[last_seen_len:]
                    last_seen_len = len(content)
                    last_output_at = now
                    recent += strip_ansi(appended)
                    if len(recent) > 20000:
                        trim = len(recent) - 20000
                        recent = recent[trim:]
                        trust_answer_pos = max(0, trust_answer_pos - trim)
                        response_start_pos = max(0, response_start_pos - trim)
                        prompt_echo_end_pos = max(0, prompt_echo_end_pos - trim)
                        nudge_echo_pos = max(0, nudge_echo_pos - trim)

            recent_for_ready = recent[trust_answer_pos:]
            recent_tail = recent[-600:]
            ready_for_prompt = (
                "Environment loaded:" in recent_for_ready
                and "Type @ to mention files" in recent_for_ready
                and "Esc to cancel" not in recent_tail
            )
            prompt_echo_visible = (
                prompt_search_start_pos > 0
                and "We are demonstrating defuddle on"
                in recent[prompt_search_start_pos:]
            )
            can_retry_prompt = (
                not prompt_sent
                and prompt_attempts < 3
                and (prompt_attempts == 0 or now - prompt_last_sent_at >= 5.0)
            )
            if ready_for_prompt and can_retry_prompt:
                prompt_search_start_pos = len(recent)
                type_text(window_id, PROMPT)
                send_keys(window_id, "Return")
                prompt_attempts += 1
                prompt_last_sent_at = time.time()
                time.sleep(1.0)
                if log_path.exists():
                    flushed = log_path.read_text(errors="ignore")
                    recent = strip_ansi(flushed.replace("\r", ""))
                    last_seen_len = len(flushed)
                if "We are demonstrating defuddle on" in recent[prompt_search_start_pos:]:
                    prompt_sent = True
                    prompt_echo_end_pos = len(recent) + 2000
                    last_output_at = time.time()
                    record_timing("prompt_sent")
                    print(
                        f"\n[controller] sent initial task prompt "
                        f"(attempt={prompt_attempts}, echo_end_pos={prompt_echo_end_pos})",
                        file=sys.stderr,
                    )
                else:
                    print(
                        f"\n[controller] prompt attempt {prompt_attempts} did not echo; waiting to retry",
                        file=sys.stderr,
                    )
                time.sleep(0.8)
                continue

            if (
                "Confirm folder trust" in recent
                or "Do you trust the files in this folder?" in recent
            ) and not trusted_sent:
                send_keys(window_id, "2", "Return")
                trusted_sent = True
                trust_answer_pos = len(recent)
                print("\n[controller] answered folder trust with option 2", file=sys.stderr)
                time.sleep(0.5)
                continue

            generic_prompt = (
                "Do you want to proceed?" in recent
                or "Quick safety check:" in recent
                or ("Allow" in recent and "1." in recent and "2." in recent)
                or "Continue?" in recent
            )
            if generic_prompt and generic_confirm_sent < 2:
                send_keys(window_id, "1", "Return")
                generic_confirm_sent += 1
                print("\n[controller] answered generic prompt with option 1", file=sys.stderr)
                time.sleep(0.5)
                continue

            if prompt_sent and not copilot_responded:
                response_markers = (
                    "● Thinking", "● Search", "● Read", "● Finding",
                    "Worked for", "─ Worked for", "get_stats", "search_symbols",
                    "explain_module", "get_call_graph", "semantic_search",
                )
                if any(m in recent for m in response_markers):
                    copilot_responded = True
                    response_start_pos = max(len(recent), prompt_echo_end_pos)
                    record_timing("copilot_responding")
                    print("\n[controller] Copilot started responding", file=sys.stderr)

            if not copilot_responded:
                time.sleep(0.2)
                continue

            safe_pos = max(response_start_pos, nudge_echo_pos)
            response_text = recent[safe_pos:]

            if "(MCP: defuddle)" in response_text:
                if not mcp_ever_seen:
                    record_timing("mcp_tool_seen")
                mcp_ever_seen = True

            # Completion: agent invoked MCP tools and finished a full response turn.
            # "─ Worked for" appears in some Copilot versions; fall back to detecting the
            # idle input prompt (visible only when not generating) after substantial output.
            recent_tail = recent[-600:]
            copilot_idle = (
                "Type @ to mention files" in recent_tail
                and "Esc to cancel" not in recent_tail
            )
            analysis_complete = mcp_ever_seen and (
                "─ Worked for" in response_text
                or (len(response_text) > 3000 and copilot_idle)
            )

            if analysis_complete and demo_done_seen_at is None:
                demo_done_seen_at = now
                record_timing("demo_done")
                print("\n[controller] detected completion (summary produced)", file=sys.stderr)

            if demo_done_seen_at is not None and not finish_nudge_sent:
                type_text(
                    window_id,
                    "Analysis complete. Print a concise final summary and exit.",
                )
                send_keys(window_id, "Return")
                finish_nudge_sent = True
                last_output_at = time.time()
                print("\n[controller] sent finish nudge", file=sys.stderr)
                time.sleep(0.8)
                continue

            if demo_done_seen_at is not None and not exit_sent and now - demo_done_seen_at >= 60.0:
                send_keys(window_id, "ctrl+c")
                time.sleep(0.5)
                send_keys(window_id, "ctrl+d")
                time.sleep(0.5)
                terminate_process_group(process, signal.SIGINT)
                exit_sent = True
                print("\n[controller] terminated Copilot session after completion", file=sys.stderr)
                time.sleep(1.0)
                continue

            if demo_done_seen_at is None and now - last_output_at >= idle_nudge_seconds and nudge_sent < 1:
                type_text(
                    window_id,
                    "Continue the analysis. Use the defuddle MCP tools, then summarize and exit.",
                )
                send_keys(window_id, "Return")
                nudge_sent += 1
                last_output_at = time.time()
                nudge_echo_pos = len(recent) + 2000
                print("\n[controller] sent idle nudge", file=sys.stderr)

            time.sleep(0.2)

        try:
            return_code = process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            terminate_process_group(process, signal.SIGTERM)
            return_code = process.wait(timeout=10)
    finally:
        if not keep_log:
            try:
                log_path.unlink(missing_ok=True)
            except OSError:
                pass

    if demo_done_seen_at is None:
        print("\n[controller] completion was not observed", file=sys.stderr)
        return 1

    if return_code not in (0, 130, -2, -15):
        print(f"\n[controller] copilot exited with {return_code}", file=sys.stderr)
        return return_code

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
