//! Content scoring — find main content using entry point selectors + scoring.
//!
//! Ported from defuddle's findMainContent + ContentScorer.scoreElement.

use scraper::{ElementRef, Html, Selector};

use crate::constants::*;

/// Find the main content element's HTML using defuddle's entry point strategy.
///
/// 1. Try each ENTRY_POINT_SELECTOR in priority order
/// 2. Score each matched element with ContentScorer logic
/// 3. Pick the best candidate, preferring specific children over parents
pub fn find_main_content(html: &str) -> String {
    let document = Html::parse_document(html);

    let mut candidates: Vec<(String, f64, usize)> = Vec::new(); // (html, score, priority_index)

    for (index, sel_str) in ENTRY_POINT_SELECTORS.iter().enumerate() {
        if let Ok(sel) = Selector::parse(sel_str) {
            for el in document.select(&sel) {
                let priority_score = (ENTRY_POINT_SELECTORS.len() - index) as f64 * 40.0;
                let content_score = score_element(&el);
                let total = priority_score + content_score;
                candidates.push((el.html(), total, index));
            }
        }
    }

    if candidates.is_empty() {
        return html.to_string();
    }

    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    candidates[0].0.clone()
}

/// Score an element as a content candidate.
/// Ported from defuddle ContentScorer.scoreElement.
pub fn score_element(el: &ElementRef) -> f64 {
    let mut score = 0.0f64;

    let text: String = el.text().collect();
    let words = text.split_whitespace().count() as f64;
    score += words;

    // Paragraph count bonus
    if let Ok(sel) = Selector::parse("p") {
        score += el.select(&sel).count() as f64 * 10.0;
    }

    // Comma counting — prose has commas
    score += text.matches(',').count() as f64;

    // Image density penalty
    if let Ok(sel) = Selector::parse("img") {
        let img_count = el.select(&sel).count() as f64;
        let img_density = img_count / words.max(1.0);
        score -= img_density * 3.0;
    }

    // Content class indicators
    let class = el.value().attr("class").unwrap_or("").to_lowercase();
    if class.contains("content") || class.contains("article") || class.contains("post") {
        score += 15.0;
    }

    // Footnote presence bonus
    if let Ok(sel) = Selector::parse(FOOTNOTE_INLINE_REFS) {
        if el.select(&sel).next().is_some() {
            score += 10.0;
        }
    }
    if let Ok(sel) = Selector::parse(FOOTNOTE_LIST_SELECTORS) {
        if el.select(&sel).next().is_some() {
            score += 10.0;
        }
    }

    // Nested table penalty
    if let Ok(sel) = Selector::parse("table") {
        score -= el.select(&sel).count() as f64 * 5.0;
    }

    // Link density multiplier (capped at 0.5 reduction)
    if let Ok(sel) = Selector::parse("a") {
        let link_text_len: f64 = el
            .select(&sel)
            .map(|a| a.text().collect::<String>().len() as f64)
            .sum();
        let text_len = text.len().max(1) as f64;
        let link_density = (link_text_len / text_len).min(0.5);
        score *= 1.0 - link_density;
    }

    score
}
