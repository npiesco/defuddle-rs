#!/usr/bin/env python3
"""Run defuddle segment 2: real extension side panel on the live SQLite page via visible Playwright."""
import os
import shutil
import signal
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT_DIR = Path(__file__).resolve().parents[2]
EXTENSION_DIR = ROOT_DIR / "extension"
PLAYWRIGHT_DIR = ROOT_DIR / "tests" / "wasm-smoke" / "node_modules" / "playwright"
DEMO_URL = os.environ.get(
    "DEFUDDLE_DEMO_URL",
    "https://sqlite.org/forum/forumpost/40a2358ad9241700?",
)


def _check_prerequisites() -> list[str]:
    errors = []
    if not os.environ.get("DISPLAY"):
        errors.append("DISPLAY is not set")
    if not shutil.which("node"):
        errors.append("node not installed")
    if not EXTENSION_DIR.is_dir():
        errors.append(f"extension dir not found: {EXTENSION_DIR}")
    if not (EXTENSION_DIR / "manifest.json").is_file():
        errors.append(f"extension manifest not found: {EXTENSION_DIR / 'manifest.json'}")
    if not PLAYWRIGHT_DIR.is_dir():
        errors.append(f"playwright package not found: {PLAYWRIGHT_DIR}")
    return errors


def main() -> int:
    errors = _check_prerequisites()
    if errors:
        print("[extension-scene] PREREQUISITES FAILED:", file=sys.stderr)
        for error in errors:
            print(f"  - {error}", file=sys.stderr)
        return 1

    hold_seconds = float(os.environ.get("DEMO_EXTENSION_HOLD_SECONDS", "12"))
    user_data_dir = Path(tempfile.mkdtemp(prefix="defuddle-demo-ext-", dir="/tmp"))
    script_path = Path(tempfile.mkstemp(prefix="defuddle-playwright-ext-", suffix=".cjs", dir="/tmp")[1])

    script_path.write_text(
        f"""\
const path = require('path');
const {{ chromium }} = require({PLAYWRIGHT_DIR.as_posix()!r});

const extensionDir = {EXTENSION_DIR.as_posix()!r};
const demoUrl = {DEMO_URL!r};
const holdSeconds = {hold_seconds!r};
const userDataDir = {str(user_data_dir)!r};

(async () => {{
  const context = await chromium.launchPersistentContext(userDataDir, {{
    headless: false,
    channel: 'chromium',
    args: [
      `--disable-extensions-except=${{extensionDir}}`,
      `--load-extension=${{extensionDir}}`,
    ],
    viewport: null,
  }});

  try {{
    let page = context.pages()[0];
    if (!page) {{
      page = await context.newPage();
    }}

    await page.goto(demoUrl, {{ waitUntil: 'domcontentloaded' }});
    await page.bringToFront();

    let serviceWorker = context.serviceWorkers()[0];
    if (!serviceWorker) {{
      serviceWorker = await context.waitForEvent('serviceworker');
    }}
    const extensionId = serviceWorker.url().split('/')[2];

    const helper = await context.newPage();
    await helper.goto(`chrome-extension://${{extensionId}}/panel.html`, {{ waitUntil: 'domcontentloaded' }});
    await helper.evaluate((targetUrl) => {{
      const button = document.querySelector('#capture-active-tab');
      if (!button) {{
        throw new Error('capture button not found');
      }}
      const openForTarget = async () => {{
        const tabs = await chrome.tabs.query({{ lastFocusedWindow: true }});
        const target = tabs.find((tab) => typeof tab.url === 'string' && tab.url.startsWith(targetUrl));
        if (!target || target.id == null || target.windowId == null) {{
          throw new Error('sqlite target tab not found');
        }}
        const response = await chrome.tabs.sendMessage(target.id, {{ type: 'DEFUDDLE_CAPTURE_PAGE' }});
        if (!response || !response.ok) {{
          throw new Error('capture failed');
        }}
        await chrome.storage.session.set({{ defuddle_latest_capture: response.payload }});
        await chrome.sidePanel.open({{ windowId: target.windowId }});
        return response.payload;
      }};
      button.addEventListener('click', () => {{
        window.__defuddlePromise = openForTarget();
      }}, {{ once: true }});
    }}, demoUrl.replace(/\\?$/, ''));

    await helper.click('#capture-active-tab');
    const payload = await helper.evaluate(() => window.__defuddlePromise);
    await page.bringToFront();

    console.error(`[extension-scene] visible url=${{payload.url || ''}} title=${{payload.title || ''}} extension_id=${{extensionId}}`);
    await page.waitForTimeout(holdSeconds * 1000);
  }} finally {{
    await context.close();
  }}
}})().catch((error) => {{
  console.error(error && error.stack ? error.stack : String(error));
  process.exit(1);
}});
""",
        encoding="utf-8",
    )

    process = subprocess.Popen(
        ["node", str(script_path)],
        cwd=str(ROOT_DIR),
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
        text=True,
        start_new_session=True,
    )

    try:
        _, stderr = process.communicate(timeout=120)
    except subprocess.TimeoutExpired:
        try:
            os.killpg(process.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        print("[extension-scene] timed out waiting for visible Playwright flow", file=sys.stderr)
        return 1
    finally:
        script_path.unlink(missing_ok=True)

    if process.returncode != 0:
        if stderr.strip():
            print(stderr.rstrip(), file=sys.stderr)
        return process.returncode

    if stderr.strip():
        print(stderr.rstrip(), file=sys.stderr)
    print(f"[extension-scene] held visible extension scene for {hold_seconds:g}s", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
