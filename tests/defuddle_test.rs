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

/// Compare our markdown output to defuddle CLI's expected output.
/// Returns (coverage_pct, missing_lines).
fn parity_check(ours: &str, expected: &str) -> (f64, Vec<String>) {
    let our_lines: Vec<&str> = ours
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    let expected_lines: Vec<&str> = expected
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    if expected_lines.is_empty() {
        return (1.0, vec![]);
    }

    let mut missing = Vec::new();
    for exp_line in &expected_lines {
        let found = our_lines
            .iter()
            .any(|our| our.contains(exp_line) || exp_line.contains(our));
        if !found {
            missing.push(exp_line.to_string());
        }
    }

    let coverage = 1.0 - (missing.len() as f64 / expected_lines.len() as f64);
    (coverage, missing)
}

fn assert_parity(name: &str, ours: &str, expected: &str, min_coverage: f64) {
    let (coverage, missing) = parity_check(ours, expected);
    assert!(
        coverage >= min_coverage,
        "{name}: content parity is {:.0}% (need {:.0}%), {}/{} lines missing:\n{}",
        coverage * 100.0,
        min_coverage * 100.0,
        missing.len(),
        expected.lines().filter(|l| !l.trim().is_empty()).count(),
        missing
            .iter()
            .take(15)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n"),
    );
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
    assert_parity("rust_blog", &result.content_markdown, &expected, 0.95);
}

// ── MDN (tables, structured docs) ───────────────────────────────────────────

#[test]
fn mdn_table_extracts_content() {
    let html = fixture("mdn_table.html");
    let result = Defuddle::parse(
        &html,
        "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/table",
    )
    .unwrap();
    assert!(
        !result.content_markdown.is_empty(),
        "should extract content"
    );
    assert!(
        result.content_markdown.to_lowercase().contains("table"),
        "should mention 'table'"
    );
}

#[test]
fn mdn_table_parity() {
    let html = fixture("mdn_table.html");
    let expected = fixture("mdn_table.expected.md");
    let result = Defuddle::parse(
        &html,
        "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/table",
    )
    .unwrap();
    assert_parity("mdn_table", &result.content_markdown, &expected, 0.95);
}

// ── HackerNews (thread, comments) ───────────────────────────────────────────

#[test]
fn hackernews_extracts_content() {
    let html = fixture("hackernews.html");
    let result = Defuddle::parse(&html, "https://news.ycombinator.com/item?id=39232976").unwrap();
    assert!(
        !result.content_markdown.is_empty(),
        "should extract content"
    );
}

#[test]
fn hackernews_parity() {
    let html = fixture("hackernews.html");
    let expected = fixture("hackernews.expected.md");
    let result = Defuddle::parse(&html, "https://news.ycombinator.com/item?id=39232976").unwrap();
    assert_parity("hackernews", &result.content_markdown, &expected, 0.95);
}

// ── fasterthanlime (long article, code blocks, nested lists) ────────────────

#[test]
fn fasterthanlime_extracts_content() {
    let html = fixture("fasterthanlime.html");
    let result = Defuddle::parse(
        &html,
        "https://fasterthanli.me/articles/a-half-hour-to-learn-rust",
    )
    .unwrap();
    assert!(!result.content_markdown.is_empty());
    assert!(
        result.content_markdown.contains("let") || result.content_markdown.contains("fn "),
        "should contain Rust code snippets"
    );
}

#[test]
fn fasterthanlime_parity() {
    let html = fixture("fasterthanlime.html");
    let expected = fixture("fasterthanlime.expected.md");
    let result = Defuddle::parse(
        &html,
        "https://fasterthanli.me/articles/a-half-hour-to-learn-rust",
    )
    .unwrap();
    assert_parity("fasterthanlime", &result.content_markdown, &expected, 0.95);
}

// ── Wikipedia (heavy structure, footnotes, tables, images, infobox) ─────────

#[test]
fn wikipedia_extracts_content() {
    let html = fixture("wikipedia_rust.html");
    let result = Defuddle::parse(
        &html,
        "https://en.wikipedia.org/wiki/Rust_(programming_language)",
    )
    .unwrap();
    assert!(!result.content_markdown.is_empty());
    assert!(
        result.content_markdown.contains("Rust"),
        "should mention Rust"
    );
}

#[test]
fn wikipedia_parity() {
    let html = fixture("wikipedia_rust.html");
    let expected = fixture("wikipedia_rust.expected.md");
    let result = Defuddle::parse(
        &html,
        "https://en.wikipedia.org/wiki/Rust_(programming_language)",
    )
    .unwrap();
    assert_parity("wikipedia", &result.content_markdown, &expected, 0.95);
}

// ── GitHub (README, repo page) ──────────────────────────────────────────────

#[test]
fn github_tokio_extracts_content() {
    let html = fixture("github_tokio.html");
    let result = Defuddle::parse(&html, "https://github.com/tokio-rs/tokio").unwrap();
    assert!(!result.content_markdown.is_empty());
}

#[test]
fn github_tokio_parity() {
    let html = fixture("github_tokio.html");
    let expected = fixture("github_tokio.expected.md");
    let result = Defuddle::parse(&html, "https://github.com/tokio-rs/tokio").unwrap();
    assert_parity("github_tokio", &result.content_markdown, &expected, 0.95);
}

// ── Joel on Software (classic blog, ads, sidebar) ───────────────────────────

#[test]
fn joel_test_extracts_content() {
    let html = fixture("joel_test.html");
    let result = Defuddle::parse(
        &html,
        "https://www.joelonsoftware.com/2000/08/09/the-joel-test-12-steps-to-better-code/",
    )
    .unwrap();
    assert!(!result.content_markdown.is_empty());
    assert!(
        result.content_markdown.contains("Joel Test")
            || result.content_markdown.contains("software team"),
        "should contain article content"
    );
}

#[test]
fn joel_test_parity() {
    let html = fixture("joel_test.html");
    let expected = fixture("joel_test.expected.md");
    let result = Defuddle::parse(
        &html,
        "https://www.joelonsoftware.com/2000/08/09/the-joel-test-12-steps-to-better-code/",
    )
    .unwrap();
    assert_parity("joel_test", &result.content_markdown, &expected, 0.80);
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
