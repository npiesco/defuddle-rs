SEGMENT_NARRATION = [
    {
        "file": "seg_01_hook.mp4",
        "label": "Hook",
        "text": (
            "Start on a noisy SQLite forum thread. The useful part is the content, "
            "not the surrounding chrome."
        ),
    },
    {
        "file": "seg_02_extension.mp4",
        "label": "Browser Extension",
        "text": (
            "The browser extension captures that same live page and opens the side panel "
            "with the cleaned extraction result."
        ),
    },
    {
        "file": "seg_03_rust.mp4",
        "label": "Rust Crate",
        "text": (
            "Now the same extraction runs directly from the Rust crate, proving the parser core "
            "is not tied to the browser."
        ),
    },
    {
        "file": "seg_04_python.mp4",
        "label": "Python Bindings",
        "text": (
            "The Python bindings expose that same engine to scripts and workflows with the same result."
        ),
    },
    {
        "file": "seg_05_mcp.mp4",
        "label": "MCP",
        "text": (
            "Through MCP, Copilot can call defuddle as a tool, fetch the page, and work from extracted content "
            "instead of raw page structure."
        ),
    },
    {
        "file": "seg_06_close.mp4",
        "label": "Close",
        "text": "One parser. Four surfaces. Same result.",
    },
]

NARRATION_TEXT = " ".join(segment["text"] for segment in SEGMENT_NARRATION)
