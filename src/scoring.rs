//! Content scoring — Readability-style heuristics to find the main content.
//!
//! Ported from defuddle's `removals/scoring.ts` ContentScorer.
//! Uses subtraction-based approach: start with full body, remove non-content blocks.

use scraper::{ElementRef, Html, Selector};

const NAV_CLASS_INDICATORS: &[&str] = &[
    "nav",
    "navigation",
    "menu",
    "sidebar",
    "footer",
    "header",
    "banner",
    "ad",
    "ads",
    "advertisement",
    "social",
    "share",
    "comment",
    "related",
    "recommended",
    "newsletter",
    "signup",
    "cookie",
    "popup",
    "modal",
    "widget",
    "promo",
    "sponsor",
    "toolbar",
    "breadcrumb",
    "pagination",
];

const CONTENT_CLASS_INDICATORS: &[&str] = &[
    "content",
    "article",
    "post",
    "entry",
    "story",
    "main",
    "body-text",
    "page-content",
    "entry-content",
    "post-content",
    "article-content",
];

/// Find the main content of an HTML page.
///
/// Strategy: try semantic elements first, then fall back to
/// subtraction-based scoring (remove non-content blocks from body).
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
                let text: String = el.text().collect();
                if text.trim().split_whitespace().count() > 20 {
                    return el.html();
                }
            }
        }
    }

    // Subtraction-based: start with body, remove non-content blocks
    if let Ok(sel) = Selector::parse("body") {
        if let Some(body) = document.select(&sel).next() {
            let body_html = body.inner_html();
            return remove_non_content_blocks(&body_html);
        }
    }

    html.to_string()
}

/// Remove blocks that score as non-content from the HTML.
fn remove_non_content_blocks(html: &str) -> String {
    let doc = Html::parse_fragment(html);
    let mut to_remove: Vec<String> = Vec::new();

    let block_sel = Selector::parse(
        "div, section, aside, nav, header, footer, form, figure, details, fieldset",
    )
    .unwrap();

    for el in doc.root_element().select(&block_sel) {
        // Skip elements inside <pre> (code blocks)
        if is_inside_pre(&el) {
            continue;
        }

        // Skip elements that are likely content
        if is_likely_content(&el) {
            continue;
        }

        let score = score_non_content_block(&el);
        if score < 0.0 {
            to_remove.push(el.html());
        }
    }

    // Remove in reverse order of length (longest first) to avoid substring issues
    to_remove.sort_by(|a, b| b.len().cmp(&a.len()));

    let mut result = html.to_string();
    for removal in &to_remove {
        // Only remove if not a parent of already-removed content
        if result.contains(removal.as_str()) {
            result = result.replacen(removal.as_str(), "", 1);
        }
    }

    result
}

/// Score a block element — negative = non-content, positive = content.
fn score_non_content_block(el: &ElementRef) -> f64 {
    let mut score = 0.0f64;

    let text: String = el.text().collect();
    let text_len = text.len() as f64;
    let word_count = text.split_whitespace().count() as f64;

    // Very short blocks are suspicious
    if word_count < 5.0 {
        score -= 10.0;
    }

    // Class/ID indicators
    let class_name = el.value().attr("class").unwrap_or("").to_lowercase();
    let id_name = el.value().attr("id").unwrap_or("").to_lowercase();
    let class_and_id = format!("{} {}", class_name, id_name);

    for indicator in NAV_CLASS_INDICATORS {
        if class_and_id.contains(indicator) {
            score -= 25.0;
        }
    }
    for indicator in CONTENT_CLASS_INDICATORS {
        if class_and_id.contains(indicator) {
            score += 30.0;
        }
    }

    // Link density — high link density = navigation
    if let Ok(sel) = Selector::parse("a") {
        let link_text_len: f64 = el
            .select(&sel)
            .map(|a| a.text().collect::<String>().len() as f64)
            .sum();
        let link_density = if text_len > 0.0 {
            link_text_len / text_len
        } else {
            1.0
        };
        if link_density > 0.5 {
            score -= 20.0 * link_density;
        }
    }

    // Paragraph presence = content
    if let Ok(sel) = Selector::parse("p") {
        let p_count = el.select(&sel).count() as f64;
        score += p_count * 5.0;
    }

    // Comma density — prose has commas
    let commas = text.matches(',').count() as f64;
    score += commas * 0.5;

    // Tag name bonus/penalty
    let tag = el.value().name();
    match tag {
        "nav" | "footer" | "header" | "aside" => score -= 30.0,
        "article" | "main" | "section" => score += 10.0,
        "form" => score -= 15.0,
        _ => {}
    }

    score
}

/// Check if an element is likely content (should not be removed).
fn is_likely_content(el: &ElementRef) -> bool {
    let tag = el.value().name();

    // Semantic content tags
    if matches!(tag, "article" | "main") {
        return true;
    }

    // Content class indicators
    let class_name = el.value().attr("class").unwrap_or("").to_lowercase();
    let id_name = el.value().attr("id").unwrap_or("").to_lowercase();
    for indicator in CONTENT_CLASS_INDICATORS {
        if class_name.contains(indicator) || id_name.contains(indicator) {
            return true;
        }
    }

    // Has many paragraphs
    if let Ok(sel) = Selector::parse("p") {
        if el.select(&sel).count() >= 3 {
            return true;
        }
    }

    // Has headings + paragraphs (article structure)
    let has_headings = Selector::parse("h1, h2, h3, h4, h5, h6")
        .ok()
        .map(|sel| el.select(&sel).count() > 0)
        .unwrap_or(false);
    let has_paragraphs = Selector::parse("p")
        .ok()
        .map(|sel| el.select(&sel).count() > 0)
        .unwrap_or(false);
    if has_headings && has_paragraphs {
        return true;
    }

    false
}

fn is_inside_pre(el: &ElementRef) -> bool {
    let mut node = el.parent();
    while let Some(parent) = node {
        if let Some(elem) = parent.value().as_element() {
            if elem.name() == "pre" {
                return true;
            }
        }
        node = parent.parent();
    }
    false
}
