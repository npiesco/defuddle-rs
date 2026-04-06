//! defuddle-rs — Clean-room Rust port of defuddle.
//!
//! Extract clean markdown content from web pages, stripping ads, navigation,
//! and clutter. Produces readable markdown with metadata extraction.
//!
//! Pipeline (matches defuddle's parseInternal):
//! 1. Extract metadata (before DOM modification)
//! 2. Try site-specific extractor
//! 3. Find main content element (entry point selectors + scoring)
//! 4. Remove hidden elements
//! 5. Remove by selector (exact + partial)
//! 6. Score and remove non-content blocks
//! 7. Strip title h1
//! 8. Standardize content
//! 9. Convert to markdown

pub mod constants;
pub mod elements;
pub mod extractors;
pub mod fetch;
pub mod markdown;
pub mod metadata;
pub mod removals;
pub mod scoring;
pub mod standardize;

use scraper::Html;
use serde::{Deserialize, Serialize};
use url::Url;

/// Result of parsing a web page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefuddleResult {
    pub title: String,
    pub author: Option<String>,
    pub published: Option<String>,
    pub site: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub language: Option<String>,
    pub content_html: String,
    pub content_markdown: String,
    pub word_count: usize,
    pub schema_org: Option<serde_json::Value>,
}

/// Main entry point.
pub struct Defuddle;

impl Defuddle {
    /// Parse raw HTML from a given URL and extract clean content + markdown.
    pub fn parse(html: &str, url: &str) -> Result<DefuddleResult, DefuddleError> {
        let parsed_url = Url::parse(url).map_err(|e| DefuddleError::InvalidUrl(e.to_string()))?;
        let document = Html::parse_document(html);

        // 1. Extract metadata before modifying DOM
        let meta = metadata::extract(&document, &parsed_url);
        let schema_org = metadata::extract_schema_org(html);

        // 2. Try site-specific extractor first
        if let Some(extracted) = extractors::try_extract(&document, &parsed_url) {
            let word_count = count_words(&extracted.content_html);
            let content_markdown = markdown::html_to_markdown(&extracted.content_html);
            return Ok(DefuddleResult {
                title: extracted.title.unwrap_or(meta.title),
                author: extracted.author.or(meta.author),
                published: extracted.published.or(meta.published),
                site: extracted.site.or(meta.site),
                description: meta.description,
                image: meta.image,
                language: meta.language,
                content_html: extracted.content_html,
                content_markdown,
                word_count,
                schema_org,
            });
        }

        // 3. Find main content element (entry point selectors + scoring)
        let main_content_html = scoring::find_main_content(html);

        // Count words from full content (including title heading) — matches defuddle
        let word_count = count_words(&main_content_html);

        // 4-6. Remove clutter pipeline (hidden → exact selectors → partial → scoring)
        let cleaned = removals::remove_clutter(&main_content_html);

        // 7. Strip title h1 from content (title extracted as metadata)
        let without_title = Self::strip_title_heading(&cleaned, &meta.title);

        // 8. Standardize content
        let standardized = standardize::standardize(&without_title);

        // 9. Convert to markdown
        let content_markdown = markdown::html_to_markdown(&standardized);

        Ok(DefuddleResult {
            title: meta.title,
            author: meta.author,
            published: meta.published,
            site: meta.site,
            description: meta.description,
            image: meta.image,
            language: meta.language,
            content_html: standardized,
            content_markdown,
            word_count,
            schema_org,
        })
    }

    /// Fetch a URL and parse it.
    pub async fn fetch_and_parse(url: &str) -> Result<DefuddleResult, DefuddleError> {
        let html = fetch::get(url).await?;
        Self::parse(&html, url)
    }

    /// Remove the first <h1> whose text matches the page title.
    fn strip_title_heading(html: &str, title: &str) -> String {
        let doc = Html::parse_fragment(html);
        if let Ok(sel) = scraper::Selector::parse("h1") {
            for el in doc.select(&sel) {
                let h1_text: String = el.text().collect::<String>();
                let h1_trimmed = h1_text.trim();
                let title_trimmed = title.trim();
                if h1_trimmed == title_trimmed
                    || title_trimmed.starts_with(h1_trimmed)
                    || h1_trimmed.starts_with(title_trimmed)
                {
                    let h1_html = el.html();
                    return html.replacen(&h1_html, "", 1);
                }
            }
        }
        html.to_string()
    }
}

fn count_words(html: &str) -> usize {
    let doc = Html::parse_fragment(html);
    let text: String = doc.root_element().text().collect();
    text.split_whitespace().count()
}

#[derive(Debug, thiserror::Error)]
pub enum DefuddleError {
    #[error("invalid URL: {0}")]
    InvalidUrl(String),
    #[error("HTTP fetch failed: {0}")]
    Fetch(String),
    #[error("parse error: {0}")]
    Parse(String),
}
