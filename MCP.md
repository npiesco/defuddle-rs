# MCP

`defuddle-rs` now ships a local MCP server binary:

- Binary: `defuddle-mcp`
- Default transport: `stdio`
- Optional transport: streamable HTTP

## Tools

- `parse_html`
- `fetch_and_parse_url`
- `extract_metadata`
- `extract_markdown`

## Build

```bash
cargo build --release --bin defuddle-mcp
```

Or from the workspace helper script:

```bash
npm run build:mcp:release
```

## Stdio

Run the MCP server over stdio:

```bash
target/release/defuddle-mcp
```

You can also spell the mode explicitly:

```bash
target/release/defuddle-mcp stdio
```

### Example MCP config

```json
{
  "mcpServers": {
    "defuddle": {
      "command": "/absolute/path/to/defuddle-rs/target/release/defuddle-mcp",
      "cwd": "/absolute/path/to/defuddle-rs"
    }
  }
}
```

## Streamable HTTP

Run the MCP server over streamable HTTP:

```bash
target/release/defuddle-mcp http --bind 127.0.0.1:8080 --path /mcp
```

For stateless JSON responses instead of SSE framing:

```bash
target/release/defuddle-mcp http --bind 127.0.0.1:8080 --path /mcp --stateless --json-response
```

### HTTP endpoint

- Default bind: `127.0.0.1:8080`
- Default path: `/mcp`

Example URL:

```text
http://127.0.0.1:8080/mcp
```

## Notes

- `fetch_and_parse_url` is native/server-side, so it is not blocked by browser CORS.
- `extract_metadata` uses the lightweight metadata extractor directly and does not run the full content-cleaning pipeline.
- `extract_markdown` returns markdown plus lightweight metadata without returning cleaned HTML.
