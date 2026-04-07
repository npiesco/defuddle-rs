pub mod constants;
pub mod elements;
pub mod extractors;
pub mod fetch;
pub mod markdown;
pub mod metadata;
pub mod removals;
pub mod scoring;
pub mod standardize;

use dom_query::Document;
use serde::{Deserialize, Serialize};
use url::Url;

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

pub struct Defuddle;

impl Defuddle {
    pub fn parse(html: &str, url: &str) -> Result<DefuddleResult, DefuddleError> {
        let parsed_url = Url::parse(url).map_err(|e| DefuddleError::InvalidUrl(e.to_string()))?;
        let doc = Document::from(html);

        let meta = metadata::extract(&doc, &parsed_url);
        let schema_org = metadata::extract_schema_org(&doc);

        if let Some(extracted) = extractors::try_extract(&doc, &parsed_url) {
            let content_markdown = markdown::convert(&extracted.content_html);
            let word_count = Document::from(extracted.content_html.as_str())
                .text()
                .split_whitespace()
                .count();
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

        // Find main content
        let main_selector = scoring::find_main_content_selector(&doc);
        let main_sel = doc.select(&main_selector);
        let word_count = main_sel.text().split_whitespace().count();

        // Remove clutter via DOM mutation
        let main_sel = doc.select(&main_selector);
        removals::remove_clutter(&doc, &main_sel);

        // Strip title h1
        let title_text = meta.title.clone();
        for node in doc.select("h1").nodes().iter() {
            let h1_text = node.text().to_string();
            let h1_trimmed = h1_text.trim();
            let title_trimmed = title_text.trim();
            if h1_trimmed == title_trimmed
                || title_trimmed.starts_with(h1_trimmed)
                || h1_trimmed.starts_with(title_trimmed)
            {
                node.remove_from_parent();
                break;
            }
        }

        // Re-select after mutations and serialize
        let main_after = doc.select(&main_selector);
        let content_html = main_after.inner_html().to_string();
        let content_markdown = markdown::convert(&content_html);

        Ok(DefuddleResult {
            title: meta.title,
            author: meta.author,
            published: meta.published,
            site: meta.site,
            description: meta.description,
            image: meta.image,
            language: meta.language,
            content_html,
            content_markdown,
            word_count,
            schema_org,
        })
    }

    pub async fn fetch_and_parse(url: &str) -> Result<DefuddleResult, DefuddleError> {
        let html = fetch::get(url).await?;
        Self::parse(&html, url)
    }
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
