//! Content scoring — Readability-style heuristics to find the main content.

use scraper::{Html, Selector};

/// Find the main content of an HTML page using content scoring heuristics.
///
/// Scores elements by text density, paragraph count, link density, and
/// structural cues (article, main, role="main", etc.).
pub fn find_main_content(html: &str) -> String {
    let document = Html::parse_document(html);

    // Try semantic elements first
    for selector_str in &[
        "article",
        "main",
        "[role=\"main\"]",
        ".post-content",
        ".article-content",
        ".entry-content",
    ] {
        if let Ok(sel) = Selector::parse(selector_str) {
            if let Some(el) = document.select(&sel).next() {
                return el.html();
            }
        }
    }

    // Fall back to body-level scoring
    if let Ok(sel) = Selector::parse("body") {
        if let Some(body) = document.select(&sel).next() {
            // Score direct children of body by text density
            let mut best_html = String::new();
            let mut best_score = 0.0f64;

            for child in body.children() {
                if let Some(el) = child.value().as_element() {
                    let tag = el.name();
                    // Skip obvious non-content elements
                    if matches!(
                        tag,
                        "script" | "style" | "nav" | "header" | "footer" | "aside"
                    ) {
                        continue;
                    }

                    if let Some(child_ref) = scraper::ElementRef::wrap(child) {
                        let text: String = child_ref.text().collect();
                        let text_len = text.trim().len() as f64;
                        let link_sel = Selector::parse("a").unwrap();
                        let link_text_len: f64 = child_ref
                            .select(&link_sel)
                            .map(|a| a.text().collect::<String>().len() as f64)
                            .sum();
                        let link_density = if text_len > 0.0 {
                            link_text_len / text_len
                        } else {
                            1.0
                        };

                        // Score: more text + less link density = better
                        let score = text_len * (1.0 - link_density);

                        if score > best_score {
                            best_score = score;
                            best_html = child_ref.html();
                        }
                    }
                }
            }

            if !best_html.is_empty() {
                return best_html;
            }

            return body.html();
        }
    }

    html.to_string()
}
