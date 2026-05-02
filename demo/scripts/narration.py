SEGMENT_NARRATION = [
    {
        "file": "seg_01_hook.mp4",
        "label": "Hook",
        "text": (
            "We start on noisy SQLite forum thread. Page is full of navigation, "
            "headers, sidebars, footer chrome. Actual content — post itself — "
            "is buried in markup. This is problem defuddle solves: pull article "
            "out, drop everything else."
        ),
    },
    {
        "file": "seg_02_extension.mp4",
        "label": "Browser Extension",
        "text": (
            "First surface: browser extension. Captures live page directly "
            "from tab, opens side panel with cleaned result. You see "
            "title, metadata, readable markdown — extracted in place, no "
            "round trip to server."
        ),
    },
    {
        "file": "seg_03_rust.mp4",
        "label": "Rust Crate",
        "text": (
            "Same page, second surface: Rust crate. Extension and CLI both "
            "call into same parser core. From terminal, crate produces same "
            "title, same metadata, same markdown — proving engine is not tied "
            "to browser."
        ),
    },
    {
        "file": "seg_04_python.mp4",
        "label": "Python Bindings",
        "text": (
            "Third surface: Python bindings. Same engine, exposed to scripts "
            "and data workflows. One import, one call, identical output."
        ),
    },
    {
        "file": "seg_05_mcp.mp4",
        "label": "MCP",
        "text": (
            "Fourth surface, most interesting: MCP. Through Model Context "
            "Protocol, agents like Copilot call defuddle as tool. Agent invokes "
            "fetch and parse on live URL, gets back structured metadata, works from "
            "cleaned content rather than raw chrome. "
            "Watch Copilot pull title, site, word count, markdown "
            "preview — all from single tool call. Can also use extract metadata for "
            "metadata-only pass, or extract markdown to convert HTML it already has. "
            "Agent never deals with navigation, scripts, layout. Works on "
            "same clean structure extension and crate produced. Same parser, "
            "different consumer. Agent gets high-signal context instead of five "
            "thousand line HTML document. This is what makes defuddle useful for AI: "
            "deterministic extraction, identical across surfaces, exposed through "
            "standard protocol modern agents already speak."
        ),
    },
    {
        "file": "seg_06_close.mp4",
        "label": "Close",
        "text": (
            "One parser, written in Rust. Four surfaces: browser extension, Rust crate, "
            "Python bindings, MCP server. Same input produces same output "
            "everywhere. That is the whole point. Defuddle dot R S — try it."
        ),
    },
]

NARRATION_TEXT = " ".join(segment["text"] for segment in SEGMENT_NARRATION)
