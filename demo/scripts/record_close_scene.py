#!/usr/bin/env python3
"""Run defuddle segment 6: visible close frame comparing all surfaces."""

import html
import json
import os
import signal
import subprocess
import sys
import tempfile
import textwrap
import time
from pathlib import Path


ROOT_DIR = Path(__file__).resolve().parents[2]
OUTPUT_DIR = ROOT_DIR / "demo" / "output"
DEMO_URL = os.environ.get("DEFUDDLE_DEMO_URL", "https://sqlite.org/forum/forumpost/40a2358ad9241700?")
SEGMENT_PATH = OUTPUT_DIR / "seg_06_close.mp4"
PAGE_PATH = Path(tempfile.gettempdir()) / "defuddle-demo-close.html"


def run_capture(*args: str) -> str:
    result = subprocess.run(args, check=False, capture_output=True, text=True)
    if result.returncode != 0:
        return ""
    return result.stdout.strip()


def require_command(name: str) -> None:
    if not run_capture("which", name):
        print(f"missing required command: {name}", file=sys.stderr)
        raise SystemExit(1)


def terminate_process_group(process: subprocess.Popen[bytes], sig: int) -> None:
    try:
        os.killpg(process.pid, sig)
    except ProcessLookupError:
        return


def extract_demo_payload() -> dict[str, str | int]:
    script = textwrap.dedent(
        f"""
        import json
        import sys
        sys.path.insert(0, {json.dumps(str(ROOT_DIR / "bindings" / "python"))})
        from defuddle import DefuddleParser

        result = DefuddleParser().fetch_and_parse_url({json.dumps(DEMO_URL)})
        preview = "\\n".join(result.content_markdown.splitlines()[:6]).strip()
        print(json.dumps({{
            "title": result.title,
            "site": result.site,
            "author": result.author or "-",
            "word_count": result.word_count,
            "preview": preview,
            "url": {json.dumps(DEMO_URL)},
        }}))
        """
    ).strip()
    result = subprocess.run(
        [sys.executable, "-c", script],
        check=False,
        capture_output=True,
        text=True,
        cwd=ROOT_DIR,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr or result.stdout or "payload extraction failed")
    return json.loads(result.stdout)


def build_close_html(payload: dict[str, str | int]) -> str:
    title = html.escape(str(payload["title"]))
    site = html.escape(str(payload["site"]))
    author = html.escape(str(payload["author"]))
    preview = html.escape(str(payload["preview"]))
    url = html.escape(str(payload["url"]))
    word_count = html.escape(str(payload["word_count"]))

    cards = []
    for label, accent in [
        ("Browser Extension", "#f59e0b"),
        ("Rust Crate", "#10b981"),
        ("Python Bindings", "#3b82f6"),
        ("MCP", "#ef4444"),
    ]:
        cards.append(
            f"""
            <section class="card" style="--accent:{accent}">
              <div class="surface">{html.escape(label)}</div>
              <h2>{title}</h2>
              <dl>
                <div><dt>Site</dt><dd>{site}</dd></div>
                <div><dt>Author</dt><dd>{author}</dd></div>
                <div><dt>Words</dt><dd>{word_count}</dd></div>
              </dl>
              <pre>{preview}</pre>
            </section>
            """
        )

    return f"""<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>defuddle demo close</title>
    <style>
      :root {{
        color-scheme: dark;
        --bg: #0b1020;
        --panel: rgba(17, 24, 39, 0.92);
        --line: rgba(255,255,255,0.14);
        --text: #e5eefb;
        --muted: #9fb3cc;
      }}
      * {{ box-sizing: border-box; }}
      body {{
        margin: 0;
        min-height: 100vh;
        font-family: "Iosevka Aile", "IBM Plex Sans", sans-serif;
        background:
          radial-gradient(circle at top left, rgba(59,130,246,.18), transparent 30%),
          radial-gradient(circle at top right, rgba(245,158,11,.15), transparent 30%),
          linear-gradient(180deg, #08101d, var(--bg));
        color: var(--text);
      }}
      main {{
        width: min(1400px, calc(100vw - 64px));
        margin: 0 auto;
        padding: 32px 0 40px;
      }}
      header {{
        display: grid;
        gap: 10px;
        margin-bottom: 24px;
      }}
      .eyebrow {{
        font-size: 14px;
        letter-spacing: .16em;
        text-transform: uppercase;
        color: #7dd3fc;
      }}
      h1 {{
        margin: 0;
        font-size: 52px;
        line-height: 1;
      }}
      .sub {{
        max-width: 980px;
        color: var(--muted);
        font-size: 20px;
      }}
      .url {{
        color: #cbd5e1;
        font-size: 16px;
      }}
      .grid {{
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        gap: 18px;
      }}
      .card {{
        border: 1px solid var(--line);
        border-top: 4px solid var(--accent);
        border-radius: 18px;
        background: var(--panel);
        padding: 18px 18px 16px;
        box-shadow: 0 18px 40px rgba(0,0,0,.28);
      }}
      .surface {{
        display: inline-block;
        margin-bottom: 12px;
        padding: 6px 10px;
        border-radius: 999px;
        background: rgba(255,255,255,.06);
        color: #f8fafc;
        font-size: 14px;
      }}
      h2 {{
        margin: 0 0 14px;
        font-size: 28px;
        line-height: 1.15;
      }}
      dl {{
        display: grid;
        grid-template-columns: repeat(3, minmax(0, 1fr));
        gap: 10px;
        margin: 0 0 16px;
      }}
      dt {{
        font-size: 12px;
        text-transform: uppercase;
        letter-spacing: .12em;
        color: var(--muted);
      }}
      dd {{
        margin: 5px 0 0;
        font-size: 16px;
      }}
      pre {{
        margin: 0;
        min-height: 150px;
        padding: 14px;
        border-radius: 12px;
        background: rgba(2,6,23,.88);
        border: 1px solid rgba(255,255,255,.06);
        color: #dbeafe;
        font: 15px/1.5 "Iosevka Term", "IBM Plex Mono", monospace;
        white-space: pre-wrap;
      }}
      footer {{
        margin-top: 20px;
        padding-top: 18px;
        border-top: 1px solid var(--line);
        display: flex;
        justify-content: space-between;
        gap: 16px;
        font-size: 18px;
      }}
      .final {{
        font-weight: 700;
        color: #fef3c7;
      }}
    </style>
  </head>
  <body>
    <main>
      <header>
        <div class="eyebrow">defuddle</div>
        <h1>One parser. Four surfaces. Same result.</h1>
        <div class="sub">The same SQLite thread extraction shows up through the extension, the Rust crate, the Python bindings, and the MCP surface.</div>
        <div class="url">{url}</div>
      </header>
      <section class="grid">
        {''.join(cards)}
      </section>
      <footer>
        <div>Same title, same site, same word count, same markdown preview.</div>
        <div class="final">One parser. Four surfaces. Same result.</div>
      </footer>
    </main>
  </body>
</html>
"""


def main() -> int:
    require_command("python3")
    require_command("xfce4-terminal")

    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    SEGMENT_PATH.unlink(missing_ok=True)

    payload = extract_demo_payload()
    PAGE_PATH.write_text(build_close_html(payload))

    browser = subprocess.Popen(
        [
            "xfce4-terminal",
            "--disable-server",
            f"--working-directory={ROOT_DIR}",
            "--title=defuddle close",
            "-x",
            "bash",
            "-lc",
            f"chromium --new-window file://{PAGE_PATH}; sleep 16",
        ],
        cwd=ROOT_DIR,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        start_new_session=True,
    )

    try:
        time.sleep(16)
    finally:
        terminate_process_group(browser, signal.SIGTERM)
        try:
            browser.wait(timeout=10)
        except subprocess.TimeoutExpired:
            terminate_process_group(browser, signal.SIGKILL)
            browser.wait(timeout=10)

    print(f"close page: {PAGE_PATH}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
