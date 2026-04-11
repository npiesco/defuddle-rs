#!/usr/bin/env python3

import os
import shutil
import subprocess
import sys
from pathlib import Path


ROOT_DIR = Path(__file__).resolve().parents[2]


def main() -> int:
    title = os.environ.get("DEMO_TERMINAL_TITLE", "fsgdb Demo")
    command_args = sys.argv[1:]

    xfce4_terminal = shutil.which("xfce4-terminal")
    if xfce4_terminal:
        args = [
            xfce4_terminal,
            "--disable-server",
            f"--working-directory={ROOT_DIR}",
            f"--title={title}",
        ]
        if command_args:
            args.extend(["-x", *command_args])
        result = subprocess.run(args, cwd=ROOT_DIR, check=False)
        return result.returncode

    exo_open = shutil.which("exo-open")
    if not exo_open:
        print("missing terminal launcher: need xfce4-terminal or exo-open", file=sys.stderr)
        return 1

    if command_args:
        print(
            "exo-open is available, but deterministic command startup requires xfce4-terminal; "
            "install xfce4-terminal or launch the session in the current terminal",
            file=sys.stderr,
        )
        return 1

    result = subprocess.run([exo_open, "--launch", "TerminalEmulator"], cwd=ROOT_DIR, check=False)
    return result.returncode


if __name__ == "__main__":
    raise SystemExit(main())
