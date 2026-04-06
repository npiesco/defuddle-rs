//! Clutter removal pipeline — ported from defuddle's removals/.
//!
//! Steps (applied in order to the main content element):
//! 1. remove_hidden — display:none, visibility:hidden, opacity:0, CSS framework classes
//! 2. remove_by_selector — exact + partial class/id matching
//! 3. remove_by_content_pattern — text-based navigation/boilerplate detection
//! 4. score_and_remove — heuristic scoring of remaining blocks

use regex::Regex;
use scraper::{Html, Selector};

use crate::constants::*;

/// Run the full clutter removal pipeline on HTML.
pub fn remove_clutter(html: &str) -> String {
    let mut output = html.to_string();
    output = remove_hidden(&output);
    output = remove_by_exact_selector(&output);
    output = remove_by_partial_selector(&output);
    output = score_and_remove(&output);
    output
}

// ── Step 1: Remove hidden elements ──────────────────────────────────────────

fn remove_hidden(html: &str) -> String {
    let doc = Html::parse_fragment(html);
    let mut output = html.to_string();

    // Inline style patterns
    let hidden_re = Regex::new(
        r"(?:^|;\s*)(?:display\s*:\s*none|visibility\s*:\s*hidden|opacity\s*:\s*0)(?:\s*;|\s*$)",
    )
    .unwrap();

    if let Ok(sel) = Selector::parse("*") {
        for el in doc.root_element().select(&sel) {
            // Skip math elements
            if el.value().name() == "math" {
                continue;
            }
            if Selector::parse("math, [data-mathml], .katex-mathml")
                .ok()
                .map(|s| el.select(&s).next().is_some())
                .unwrap_or(false)
            {
                continue;
            }

            let style = el.value().attr("style").unwrap_or("");
            if hidden_re.is_match(style) {
                output = output.replacen(&el.html(), "", 1);
                continue;
            }

            // CSS framework hidden utilities
            let class = el.value().attr("class").unwrap_or("");
            let tokens: Vec<&str> = class.split_whitespace().collect();
            let is_hidden = tokens.iter().any(|t| {
                *t == "hidden"
                    || t.ends_with(":hidden")
                    || *t == "invisible"
                    || t.ends_with(":invisible")
            });
            if is_hidden {
                output = output.replacen(&el.html(), "", 1);
            }
        }
    }

    output
}

// ── Step 2: Remove by exact selector ────────────────────────────────────────

fn remove_by_exact_selector(html: &str) -> String {
    let doc = Html::parse_fragment(html);
    let mut output = html.to_string();

    for sel_str in EXACT_SELECTORS {
        if let Ok(sel) = Selector::parse(sel_str) {
            for el in doc.select(&sel) {
                // Skip elements inside code blocks
                if is_inside_code(&el) {
                    continue;
                }
                let el_html = el.html();
                if !el_html.is_empty() {
                    output = output.replacen(&el_html, "", 1);
                }
            }
        }
    }

    output
}

// ── Step 3: Remove by partial selector (class/id matching) ──────────────────

fn remove_by_partial_selector(html: &str) -> String {
    let doc = Html::parse_fragment(html);
    let mut output = html.to_string();

    let partial_re = build_partial_regex();

    if let Ok(sel) = Selector::parse("[class], [id], [data-component], [data-testid]") {
        for el in doc.root_element().select(&sel) {
            // Skip code elements
            if is_inside_code(&el) {
                continue;
            }

            let tag = el.value().name();
            let is_heading = matches!(tag, "h1" | "h2" | "h3" | "h4" | "h5" | "h6");

            let class = el.value().attr("class").unwrap_or("").to_lowercase();
            let id = if is_heading {
                String::new()
            } else {
                el.value().attr("id").unwrap_or("").to_lowercase()
            };
            let data_component = el
                .value()
                .attr("data-component")
                .unwrap_or("")
                .to_lowercase();
            let data_testid = el.value().attr("data-testid").unwrap_or("").to_lowercase();

            let attrs = format!("{} {} {} {}", class, id, data_component, data_testid);
            if attrs.trim().is_empty() {
                continue;
            }

            if partial_re.is_match(&attrs) {
                let el_html = el.html();
                if !el_html.is_empty() {
                    output = output.replacen(&el_html, "", 1);
                }
            }
        }
    }

    output
}

fn build_partial_regex() -> Regex {
    let escaped: Vec<String> = PARTIAL_SELECTORS.iter().map(|p| regex::escape(p)).collect();
    Regex::new(&format!("(?i){}", escaped.join("|"))).unwrap()
}

// ── Step 4: Score and remove non-content blocks ─────────────────────────────

fn score_and_remove(html: &str) -> String {
    let doc = Html::parse_fragment(html);
    let mut output = html.to_string();

    let block_sel_str = BLOCK_ELEMENTS.join(", ");
    if let Ok(sel) = Selector::parse(&block_sel_str) {
        let mut removals: Vec<String> = Vec::new();

        for el in doc.root_element().select(&sel) {
            if is_inside_code(&el) {
                continue;
            }

            if is_likely_content(&el) {
                continue;
            }

            let score = score_non_content_block(&el);
            if score < 0.0 {
                removals.push(el.html());
            }
        }

        // Remove longest first to avoid substring overlap
        removals.sort_by(|a, b| b.len().cmp(&a.len()));
        for removal in &removals {
            output = output.replacen(removal.as_str(), "", 1);
        }
    }

    output
}

/// Score a block — negative = non-content.
fn score_non_content_block(el: &scraper::ElementRef) -> f64 {
    let mut score = 0.0f64;

    let text: String = el.text().collect();
    let words = text.split_whitespace().count();

    if words < 3 {
        return 0.0;
    }

    // Commas = prose signal
    let commas = text.matches(',').count() as f64;
    score += commas;

    // Navigation indicator text matches
    let text_lower = text.to_lowercase();
    for indicator in NAVIGATION_INDICATORS {
        if text_lower.contains(indicator) {
            score -= 10.0;
        }
    }

    // Link density
    if let Ok(sel) = Selector::parse("a") {
        let links = el.select(&sel).count();
        let link_density = links as f64 / words.max(1) as f64;
        if link_density > 0.5 {
            score -= 15.0;
        }

        // High link text ratio for small blocks
        if links > 1 && words < 80 {
            let link_text_len: usize = el
                .select(&sel)
                .map(|a| a.text().collect::<String>().len())
                .sum();
            let text_len = text.len().max(1);
            if link_text_len as f64 / text_len as f64 > 0.8 {
                score -= 15.0;
            }
        }
    }

    // List + link combo (navigation)
    if let (Ok(ul_sel), Ok(a_sel)) = (Selector::parse("ul, ol"), Selector::parse("a")) {
        let lists = el.select(&ul_sel).count();
        let links = el.select(&a_sel).count();
        if lists > 0 && links > lists * 3 {
            score -= 10.0;
        }
    }

    // Non-content class patterns
    let class = el.value().attr("class").unwrap_or("").to_lowercase();
    let id = el.value().attr("id").unwrap_or("").to_lowercase();
    let non_content = [
        "advert",
        "ad-",
        "ads",
        "banner",
        "cookie",
        "copyright",
        "footer",
        "header",
        "menu",
        "nav",
        "newsletter",
        "popular",
        "privacy",
        "recommended",
        "related",
        "share",
        "sidebar",
        "social",
        "sponsored",
        "subscribe",
        "terms",
        "trending",
        "widget",
    ];
    for pattern in non_content {
        if class.contains(pattern) || id.contains(pattern) {
            score -= 8.0;
        }
    }

    score
}

/// Check if an element is likely content (should NOT be removed).
fn is_likely_content(el: &scraper::ElementRef) -> bool {
    let role = el.value().attr("role").unwrap_or("");
    if matches!(role, "article" | "main" | "contentinfo") {
        return true;
    }

    let class = el.value().attr("class").unwrap_or("").to_lowercase();
    let id = el.value().attr("id").unwrap_or("").to_lowercase();

    for indicator in CONTENT_INDICATORS {
        if class.contains(indicator) || id.contains(indicator) {
            return true;
        }
    }

    // Elements containing code, tables, figures, images = content
    if let Ok(sel) = Selector::parse(CONTENT_ELEMENT_SELECTOR) {
        if el.select(&sel).next().is_some() {
            return true;
        }
    }

    let text: String = el.text().collect();
    let words = text.split_whitespace().count();

    // Paragraphs + list items
    let p_count = Selector::parse("p")
        .ok()
        .map(|s| el.select(&s).count())
        .unwrap_or(0);
    let li_count = Selector::parse("li")
        .ok()
        .map(|s| el.select(&s).count())
        .unwrap_or(0);
    let content_blocks = p_count + li_count;

    if words > 50 && content_blocks > 1 {
        return true;
    }
    if words > 100 {
        return true;
    }
    if words > 30 && content_blocks > 0 {
        return true;
    }

    // Prose text with sentence-ending punctuation and low link density
    if words >= 10 && text.contains('.') || text.contains('?') || text.contains('!') {
        let link_count = Selector::parse("a")
            .ok()
            .map(|s| el.select(&s).count())
            .unwrap_or(0);
        let link_density = link_count as f64 / words.max(1) as f64;
        if link_density < 0.1 {
            return true;
        }
    }

    false
}

fn is_inside_code(el: &scraper::ElementRef) -> bool {
    let mut node = el.parent();
    while let Some(parent) = node {
        if let Some(elem) = parent.value().as_element() {
            if matches!(elem.name(), "pre" | "code") {
                return true;
            }
        }
        node = parent.parent();
    }
    false
}
