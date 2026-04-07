# Parity Report — defuddle-rs vs defuddle

defuddle-rs is validated against the original [defuddle](https://github.com/kepano/defuddle) TypeScript implementation. Expected output for each fixture was generated with `npx defuddle parse <file> --md` and stored in `tests/fixtures/*.expected.md`. The Rust output is compared line-by-line after normalizing whitespace and escape characters.

All 8 fixtures pass at **100% line parity**.

## Summary

| Fixture | Source | Expected Lines | Rust Lines | Parity |
|---------|--------|---------------:|-----------:|--------|
| example.com | Simple page | 2 | 2 | 100% |
| Rust Blog | Article + code blocks | 43 | 43 | 100% |
| MDN Table Docs | Structured docs + tables | 559 | 559 | 100% |
| HackerNews | Thread + comments | 1134 | 1134 | 100% |
| fasterthanlime | Long article + code | 357 | 357 | 100% |
| Wikipedia (Rust) | Heavy structure + footnotes | 671 | 671 | 100% |
| GitHub (tokio) | README extraction | 114 | 114 | 100% |
| Joel on Software | Classic blog + sidebar | 89 | 89 | 100% |

---

## Methodology

Each test fixture consists of:
- **`<name>.html`** — raw HTML saved from the live page
- **`<name>.expected.md`** — markdown output from `npx defuddle parse <file> --md`

The Rust test harness (`tests/defuddle_test.rs`) runs `Defuddle::parse` on each HTML fixture and compares the output against the expected markdown. Comparison uses exact line matching with normalization for escape characters (`\\`, `` \` ``, `\.`) and line endings.

```bash
cargo test    # runs all 24 parity tests
```

---

## Fixture Details

### example.com

The simplest possible page — a single paragraph and a link.

**defuddle (JS) output:**
```markdown
This domain is for use in documentation examples without needing permission. Avoid use in operations.

[Learn more](https://iana.org/domains/example)
```

**defuddle-rs output:** identical.

---

### Rust Blog — Announcing Rust 1.76.0

Real blog post with headings, fenced code blocks, inline code, and markdown links.

**defuddle (JS) output** (first 15 lines):
```markdown
The Rust team is happy to announce a new version of Rust, 1.76.0. Rust is a programming language empowering everyone to build reliable and efficient software.

If you have a previous version of Rust installed via rustup, you can get 1.76.0 with:

$ rustup update stable

If you don't have it already, you can get rustup from the appropriate page on our website, and check out the detailed release notes for 1.76.0.

If you'd like to help us out by testing future releases, you might consider updating locally to use the beta channel (rustup default beta) or the nightly channel (rustup default nightly). Please report any bugs you might come across!

## What's in 1.76.0 stable

This release is relatively minor, but as always, even incremental improvements lead to a greater whole.
```

**defuddle-rs output:** identical across all 43 non-empty lines.

---

### MDN — `<table>` Element Documentation

Structured technical documentation with nested tables, definition lists, code examples, and deeply nested headings. 559 non-empty lines.

**defuddle-rs output:** 100% parity.

---

### HackerNews Thread

Comment thread with deeply nested replies, user links, timestamps, and inline formatting. The largest fixture at 1,134 non-empty lines.

**defuddle-rs output:** 100% parity.

---

### fasterthanlime — A Half Hour to Learn Rust

Long-form tutorial article with extensive Rust code blocks, nested bullet lists, inline code, and emphasis. 357 non-empty lines.

**defuddle-rs output:** 100% parity.

---

### Wikipedia — Rust (programming language)

Heavy encyclopedic content with infoboxes, footnotes, reference links, tables, and complex nested structure. 671 non-empty lines.

**defuddle-rs output:** 100% parity.

---

### GitHub — tokio Repository

Repository README page extracted through the GitHub-specific extractor. Contains badges, code blocks, and structured sections. 114 non-empty lines.

**defuddle-rs output:** 100% parity.

---

### Joel on Software — The Joel Test

Classic blog post with sidebar content, ads, and navigation that must be stripped. 89 non-empty lines of article content.

**defuddle-rs output:** 100% parity.

---

## Running

```bash
# run all parity tests
cargo test

# run a specific fixture
cargo test rust_blog_markdown_parity
cargo test hackernews_parity
```

Fixtures live in `tests/fixtures/`. To add a new fixture:

1. Save the HTML: `curl -o tests/fixtures/mysite.html "https://example.com/page"`
2. Generate expected output: `npx defuddle parse tests/fixtures/mysite.html --md > tests/fixtures/mysite.expected.md`
3. Add a test in `tests/defuddle_test.rs`
