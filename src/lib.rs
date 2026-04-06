//! defuddle-rs — Clean-room Rust port of defuddle.
//!
//! Extract clean markdown content from web pages, stripping ads, navigation,
//! and clutter. Produces readable markdown with metadata extraction.
//!
//! Architecture (clean-room from TypeScript reference):
//! - `Defuddle::parse(html, url)` → `DefuddleResult`
//! - `scoring` — content scoring / readability heuristics
//! - `removals` — clutter removal (selectors, hidden elements, patterns)
//! - `extractors` — site-specific handlers (GitHub, Reddit, YouTube, etc.)
//! - `markdown` — HTML → Markdown conversion
//! - `metadata` — title, author, date, description, schema.org
//! - `elements` — standardize code blocks, footnotes, callouts, math
//! - `fetch` — HTTP GET with reqwest

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
    /// Page title.
    pub title: String,
    /// Author name, if detected.
    pub author: Option<String>,
    /// Publication date, if detected.
    pub published: Option<String>,
    /// Site name / domain.
    pub site: Option<String>,
    /// Meta description.
    pub description: Option<String>,
    /// Primary image URL.
    pub image: Option<String>,
    /// Detected language (e.g. "en").
    pub language: Option<String>,
    /// Cleaned HTML content.
    pub content_html: String,
    /// Cleaned markdown content.
    pub content_markdown: String,
    /// Word count of the cleaned content.
    pub word_count: usize,
    /// Extracted schema.org JSON-LD data, if present.
    pub schema_org: Option<serde_json::Value>,
}

/// Main entry point — parse HTML and extract clean content.
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

        // 3. Generic extraction: score, remove clutter, extract main content
        let (cleaned_html, word_count) = Self::generic_extract(html, &parsed_url, &meta.title)?;
        let content_markdown = markdown::html_to_markdown(&cleaned_html);

        Ok(DefuddleResult {
            title: meta.title,
            author: meta.author,
            published: meta.published,
            site: meta.site,
            description: meta.description,
            image: meta.image,
            language: meta.language,
            content_html: cleaned_html,
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

    /// Generic content extraction for pages without a site-specific extractor.
    /// Returns (cleaned_html_without_title, word_count_including_title).
    fn generic_extract(
        html: &str,
        _url: &Url,
        title: &str,
    ) -> Result<(String, usize), DefuddleError> {
        let _document = Html::parse_document(html);

        // Remove hidden elements, nav, ads, etc.
        let cleaned = removals::remove_clutter(html);

        // Score remaining elements and find the main content
        let main_content = scoring::find_main_content(&cleaned);

        // Count words from full content (including title heading)
        let word_count = count_words(&main_content);

        // Remove the first <h1> that matches the page title (defuddle behavior:
        // title is extracted as metadata, not duplicated in content)
        let without_title_h1 = Self::strip_title_heading(&main_content, title);

        // Standardize elements (code blocks, etc.)
        let standardized = standardize::standardize(&without_title_h1);

        Ok((standardized, word_count))
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

/// Errors that can occur during defuddling.
#[derive(Debug, thiserror::Error)]
pub enum DefuddleError {
    #[error("invalid URL: {0}")]
    InvalidUrl(String),
    #[error("HTTP fetch failed: {0}")]
    Fetch(String),
    #[error("parse error: {0}")]
    Parse(String),
}
