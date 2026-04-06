//! Standardize HTML elements — normalize code blocks, callouts, etc.

/// Standardize cleaned HTML content.
pub fn standardize(html: &str) -> String {
    let mut output = html.to_string();
    output = standardize_code_blocks(&output);
    output
}

/// Normalize various code block formats into consistent `<pre><code>` elements.
fn standardize_code_blocks(html: &str) -> String {
    // For now, pass through — the markdown converter handles code blocks.
    // This will be expanded to handle platform-specific code rendering
    // (e.g., highlight.js spans, Prism classes, etc.)
    html.to_string()
}
