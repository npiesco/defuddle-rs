<div align="center">
  <h1>defuddle-rs</h1>
  <p><strong>Clean-room Rust port of <a href="https://github.com/kepano/defuddle">defuddle</a>. Extract clean markdown from any web page.</strong></p>

  [![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org/)
  [![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
  [![Tests](https://img.shields.io/badge/tests-30-success)]()
  [![Parity](https://img.shields.io/badge/defuddle%20parity-100%25-brightgreen)]()
</div>

---

Drop in a URL or raw HTML. Get back the article content as clean markdown — no ads, no navigation, no sidebars, no clutter. Metadata extracted. Links preserved. Code blocks intact.

Built as a native Rust library. No Node.js. No headless browser. No external dependencies at runtime.

The primary end-user surface in this repo is now a browser extension. Click the extension on any page to capture the current tab, then parse and inspect the extracted article locally with the bundled WASM UI.

The repo also now ships a local MCP server binary for assistants and editor integrations over stdio or streamable HTTP.

The same Rust crate now also exposes a Python package via UniFFI under `bindings/python/`, with parity for the parser-facing operations already exposed through MCP: full parse, fetch-and-parse, metadata-only extraction, and markdown-only extraction.

```rust
use defuddle_rs::Defuddle;

let html = reqwest::get("https://blog.rust-lang.org/2024/02/08/Rust-1.76.0/")
    .await?.text().await?;

let result = Defuddle::parse(&html, "https://blog.rust-lang.org/2024/02/08/Rust-1.76.0/")?;

println!("{}", result.title);             // "Announcing Rust 1.76.0"
println!("{}", result.content_markdown);  // Clean markdown article
println!("{}", result.word_count);        // 435
```

---

## Pipeline

Matches [defuddle](https://github.com/kepano/defuddle)'s `parseInternal` architecture:

```
HTML in
  │
  ├─ 1. Parse into mutable DOM (dom_query)
  ├─ 2. Extract metadata (title, author, date, schema.org, OG tags)
  ├─ 3. Try site-specific extractor (GitHub, etc.)
  ├─ 4. Find main content element (entry point selectors + scoring)
  ├─ 5. Remove hidden elements (display:none, CSS framework classes)
  ├─ 6. Remove by exact selector (70+ patterns: nav, footer, ads, etc.)
  ├─ 7. Remove by partial selector (200+ class/id patterns)
  ├─ 8. Score and remove non-content blocks (text density, link ratio)
  ├─ 9. Strip title h1 (extracted as metadata, not duplicated)
  └─ 10. Convert to markdown (headings, code, links, tables, lists)
  │
  ▼
DefuddleResult { title, author, content_markdown, word_count, schema_org, ... }
```

Every removal step receives the `mainContent` reference and **skips any element that is an ancestor of the main content** — preventing the pipeline from disconnecting the article. This matches defuddle's `element.contains(mainContent)` guard.

---

## Parity

100% line-for-line parity with [defuddle](https://github.com/kepano/defuddle) across 8 real-world fixtures — from simple pages to 1,134-line HackerNews threads.

| Fixture | Lines | Parity |
|---------|------:|--------|
| example.com | 2 | 100% |
| Rust Blog | 43 | 100% |
| MDN Table Docs | 559 | 100% |
| HackerNews | 1134 | 100% |
| fasterthanlime | 357 | 100% |
| Wikipedia (Rust) | 671 | 100% |
| GitHub (tokio) | 114 | 100% |
| Joel on Software | 89 | 100% |

Full methodology, fixture details, and side-by-side samples in **[PARITY.md](PARITY.md)**.

---

## API

### `Defuddle::parse(html, url) → Result<DefuddleResult>`

Parse raw HTML. The URL is used for metadata extraction (site name, relative URL resolution).

### `Defuddle::fetch_and_parse(url) → Result<DefuddleResult>`

Fetch a URL with reqwest and parse the response.

### `DefuddleResult`

| Field | Type | Description |
|-------|------|-------------|
| `title` | `String` | Page title (from OG tags or `<title>`) |
| `author` | `Option<String>` | Author name |
| `published` | `Option<String>` | Publication date |
| `site` | `Option<String>` | Site name / domain |
| `description` | `Option<String>` | Meta description |
| `image` | `Option<String>` | Primary image URL |
| `language` | `Option<String>` | Detected language |
| `content_html` | `String` | Cleaned HTML |
| `content_markdown` | `String` | Cleaned markdown |
| `word_count` | `usize` | Word count |
| `schema_org` | `Option<Value>` | Schema.org JSON-LD data |

---

## Python Bindings

The Python package is named `defuddle-py` and imports as `defuddle`.

The current Python surface mirrors the parser-facing MCP tools:

- `DefuddleParser.parse_html(html, url)`
- `DefuddleParser.fetch_and_parse_url(url)`
- `DefuddleParser.extract_metadata(html, url)`
- `DefuddleParser.extract_markdown(html, url)`

Build and install from source:

```bash
cargo build --release
cargo run --bin uniffi-bindgen -- generate \
    --library target/release/libdefuddle_rs.so \
    --language python \
    --out-dir bindings/python/defuddle
cp target/release/libdefuddle_rs.so bindings/python/defuddle/

uv venv /tmp/defuddle-py-uv
UV_CACHE_DIR=/tmp/uv-cache uv pip install --python /tmp/defuddle-py-uv/bin/python -e bindings/python
```

Smoke test:

```bash
/tmp/defuddle-py-uv/bin/python -c "from pathlib import Path; from defuddle import DefuddleParser; html = Path('tests/fixtures/example.html').read_text(); parser = DefuddleParser(); result = parser.extract_markdown(html, 'https://example.com'); print(result.title); print(result.word_count)"
```

Expected output:

```text
Example Domain
17
```

Notes:

- Linux library name: `libdefuddle_rs.so`
- macOS library name: `libdefuddle_rs.dylib`
- Windows library name: `defuddle_rs.dll`
- The generated UniFFI wrapper is written to `bindings/python/defuddle/defuddle_rs.py`
- The native library must sit beside that generated file before installing the package

---

## Dependencies

| Crate | Purpose |
|-------|---------|
| `dom_query` | Mutable DOM — parse, select, mutate, serialize |
| `regex` | Partial selector matching, markdown post-processing |
| `url` | URL parsing for metadata extraction |
| `reqwest` | HTTP fetch (native only) |
| `wasm-bindgen` | WASM exports (wasm32 only) |
| `serde` / `serde_json` | Result serialization, schema.org parsing |
| `thiserror` | Error types |

No `scraper`. No `html5ever` directly. `dom_query` handles the DOM with full mutation support — elements are actually removed from the tree, not string-replaced.

---

## Building

```bash
cargo build --release
cargo test
```

## MCP Server

Build the release MCP binary:

```bash
cargo build --release --bin defuddle-mcp
```

Or:

```bash
npm run build:mcp:release
```

Run over stdio:

```bash
target/release/defuddle-mcp
```

Run over streamable HTTP:

```bash
target/release/defuddle-mcp http --bind 127.0.0.1:8080 --path /mcp
```

The current MCP tools are:

- `parse_html`
- `fetch_and_parse_url`
- `extract_metadata`
- `extract_markdown`

See [MCP.md](MCP.md) for config examples and transport details.

## WebAssembly

The crate compiles to WASM for browser usage via `wasm-pack`. The `fetch` module (reqwest/tokio) is excluded on `wasm32` — only `Defuddle::parse` is available.

```bash
npm run build:wasm        # wasm-pack build → packages/defuddle-wasm/pkg/
npm run test:wasm-browser  # Playwright smoke test in headless Chromium
```

### JavaScript usage

```js
import { initDefuddleWasm, parse } from "@defuddle/wasm";

await initDefuddleWasm();

const result = parse(htmlString, "https://example.com/article");
console.log(result.title);
console.log(result.content_markdown);
```

## Browser Extension

This repo includes an extension-first workflow under `extension/`.

- The side panel is the primary UI.
- Open the Defuddle RS side panel from the extension action.
- Click `Capture Active Tab` inside the panel to pull in the current page.
- Captured pages are stored in extension session storage and parsed automatically by the panel UI.

Load the extension unpacked from:

```bash
extension/
```

Typical dev flow:

```bash
npm run build:wasm
npm run build:extension
```

To refresh the extension's bundled WASM assets after rebuilding the Rust package:

```bash
npm run build:extension
```

## Browser App

This repo still includes a static browser app build, but it is now secondary to the extension workflow.

- `/` → in-browser workspace for pasted HTML, file import, and demo usage

Build the static Pages bundle with:

```bash
npm run build
```

The build emits `dist/`, with the app mounted at the site root.

---

## Clean-Room Port

This is a clean-room Rust implementation. The original [defuddle](https://github.com/kepano/defuddle) TypeScript source is vendored in `ported_code/defuddle/` as a reference. The Rust code was written by studying the architecture, algorithms, and constants — not by translating line-by-line.

Key architectural decisions ported from defuddle:
- **Entry point selectors** with priority ordering for finding main content
- **mainContent ancestor protection** in all removal steps
- **4-step removal pipeline**: hidden → exact selectors → partial selectors → scoring
- **Content scoring heuristics**: text density, paragraph count, comma counting, link density multiplier, class/id indicators
- **Markdown conversion**: heading text collection, block-level link passthrough, figcaption plaintext, inline code suffix spacing, list marker escaping
