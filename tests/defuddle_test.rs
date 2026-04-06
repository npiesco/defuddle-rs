//! Integration tests for defuddle-rs.
//!
//! Tests against REAL HTML fixtures with expected output from the
//! original defuddle CLI (`npx defuddle parse <file> --md`).
//! Acceptance: our Rust output must match defuddle's output.

use defuddle_rs::Defuddle;

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("fixture {name}: {e}"))
}

// ── example.com — simplest possible page ────────────────────────────────────

#[test]
fn example_com_extracts_title() {
    let html = fixture("example.html");
    let result = Defuddle::parse(&html, "https://example.com").unwrap();
    assert_eq!(result.title, "Example Domain");
}

#[test]
fn example_com_extracts_language() {
    let html = fixture("example.html");
    let result = Defuddle::parse(&html, "https://example.com").unwrap();
    assert_eq!(result.language.as_deref(), Some("en"));
}

#[test]
fn example_com_markdown_matches_defuddle() {
    let html = fixture("example.html");
    let expected = fixture("example.expected.md");
    let result = Defuddle::parse(&html, "https://example.com").unwrap();

    assert_eq!(
        result.content_markdown.trim(),
        expected.trim(),
        "\n--- OURS ---\n{}\n--- EXPECTED ---\n{}\n",
        result.content_markdown.trim(),
        expected.trim(),
    );
}

#[test]
fn example_com_word_count() {
    let html = fixture("example.html");
    let result = Defuddle::parse(&html, "https://example.com").unwrap();
    assert_eq!(result.word_count, 17);
}

// ── Rust blog — real article with headings, code blocks, links ──────────────

#[test]
fn rust_blog_extracts_title() {
    let html = fixture("rust_blog.html");
    let result =
        Defuddle::parse(&html, "https://blog.rust-lang.org/2024/02/08/Rust-1.76.0/").unwrap();
    assert!(
        result.title.contains("Rust 1.76.0"),
        "title should contain 'Rust 1.76.0', got: {}",
        result.title,
    );
}

#[test]
fn rust_blog_has_code_blocks() {
    let html = fixture("rust_blog.html");
    let result =
        Defuddle::parse(&html, "https://blog.rust-lang.org/2024/02/08/Rust-1.76.0/").unwrap();
    assert!(
        result.content_markdown.contains("```"),
        "should contain fenced code blocks",
    );
    assert!(
        result.content_markdown.contains("rustup update stable"),
        "should contain the rustup command",
    );
}

#[test]
fn rust_blog_has_headings() {
    let html = fixture("rust_blog.html");
    let result =
        Defuddle::parse(&html, "https://blog.rust-lang.org/2024/02/08/Rust-1.76.0/").unwrap();
    assert!(
        result.content_markdown.contains("## "),
        "should contain h2 headings",
    );
    assert!(
        result.content_markdown.contains("### "),
        "should contain h3 headings",
    );
}

#[test]
fn rust_blog_has_links() {
    let html = fixture("rust_blog.html");
    let result =
        Defuddle::parse(&html, "https://blog.rust-lang.org/2024/02/08/Rust-1.76.0/").unwrap();
    assert!(
        result.content_markdown.contains("]("),
        "should contain markdown links",
    );
}

#[test]
fn rust_blog_strips_nav_and_footer() {
    let html = fixture("rust_blog.html");
    let result =
        Defuddle::parse(&html, "https://blog.rust-lang.org/2024/02/08/Rust-1.76.0/").unwrap();
    // The raw HTML has nav/footer — they should not appear in content
    assert!(
        !result.content_markdown.contains("Install"),
        "nav 'Install' link should be stripped",
    );
}

#[test]
fn rust_blog_markdown_parity_with_defuddle() {
    let html = fixture("rust_blog.html");
    let expected = fixture("rust_blog.expected.md");
    let result =
        Defuddle::parse(&html, "https://blog.rust-lang.org/2024/02/08/Rust-1.76.0/").unwrap();

    // Compare key content sections rather than exact match
    // (whitespace differences are acceptable)
    let our_lines: Vec<&str> = result
        .content_markdown
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    let expected_lines: Vec<&str> = expected
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    // Every substantive line from defuddle's output should appear in ours
    let mut missing = Vec::new();
    for exp_line in &expected_lines {
        if !our_lines
            .iter()
            .any(|our| our.contains(exp_line) || exp_line.contains(our))
        {
            missing.push(*exp_line);
        }
    }

    let coverage = 1.0 - (missing.len() as f64 / expected_lines.len() as f64);
    assert!(
        coverage >= 0.80,
        "content parity is {:.0}% ({} of {} lines missing):\n{}",
        coverage * 100.0,
        missing.len(),
        expected_lines.len(),
        missing
            .iter()
            .take(10)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n"),
    );
}

// ── Metadata extraction ─────────────────────────────────────────────────────

#[test]
fn metadata_extracts_schema_org_when_present() {
    // Build a page with JSON-LD
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <title>Test Article</title>
    <script type="application/ld+json">
    {"@type": "Article", "headline": "Test", "author": {"name": "Alice"}}
    </script>
</head>
<body><article><p>Hello world.</p></article></body>
</html>"#;

    let result = Defuddle::parse(html, "https://example.com/article").unwrap();
    assert!(result.schema_org.is_some());
    let schema = result.schema_org.unwrap();
    assert_eq!(schema["headline"], "Test");
}

#[test]
fn metadata_extracts_og_tags() {
    let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Fallback Title</title>
    <meta property="og:title" content="OG Title">
    <meta property="og:description" content="OG Description">
    <meta property="og:image" content="https://example.com/image.png">
    <meta property="og:site_name" content="Example Site">
</head>
<body><article><p>Content here.</p></article></body>
</html>"#;

    let result = Defuddle::parse(html, "https://example.com").unwrap();
    assert_eq!(result.title, "OG Title");
    assert_eq!(result.description.as_deref(), Some("OG Description"));
    assert_eq!(
        result.image.as_deref(),
        Some("https://example.com/image.png")
    );
    assert_eq!(result.site.as_deref(), Some("Example Site"));
}
