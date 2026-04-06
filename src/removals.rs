//! Clutter removal — strip ads, nav, hidden elements, and noise.

use scraper::{Html, Selector};

/// Selectors for elements that are almost always non-content clutter.
const REMOVE_SELECTORS: &[&str] = &[
    "script",
    "style",
    "noscript",
    "iframe",
    "nav",
    "header:not(article header)",
    "footer:not(article footer)",
    "aside",
    ".sidebar",
    ".ad",
    ".ads",
    ".advertisement",
    ".social-share",
    ".share-buttons",
    ".comments",
    ".comment-section",
    "#comments",
    ".related-posts",
    ".recommended",
    ".newsletter-signup",
    ".cookie-banner",
    ".popup",
    ".modal",
    "[aria-hidden=\"true\"]",
    "[hidden]",
    "[style*=\"display:none\"]",
    "[style*=\"display: none\"]",
    "[style*=\"visibility:hidden\"]",
    "[style*=\"visibility: hidden\"]",
];

/// Remove clutter elements from HTML. Returns cleaned HTML string.
pub fn remove_clutter(html: &str) -> String {
    let document = Html::parse_document(html);
    let mut output = html.to_string();

    // Remove elements matching clutter selectors
    for sel_str in REMOVE_SELECTORS {
        if let Ok(sel) = Selector::parse(sel_str) {
            for el in document.select(&sel) {
                let el_html = el.html();
                output = output.replace(&el_html, "");
            }
        }
    }

    output
}
