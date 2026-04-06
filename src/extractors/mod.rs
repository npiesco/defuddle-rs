//! Site-specific extractors — handle known sites with custom logic.

pub mod github;

use scraper::Html;
use url::Url;

/// Result from a site-specific extractor.
pub struct ExtractorResult {
    pub content_html: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub published: Option<String>,
    pub site: Option<String>,
}

/// Try all registered extractors. Returns the first match.
pub fn try_extract(document: &Html, url: &Url) -> Option<ExtractorResult> {
    let domain = url.host_str()?;

    if domain.contains("github.com") {
        return github::extract(document, url);
    }

    // More extractors will be added here as they're implemented:
    // reddit, youtube, hackernews, twitter, chatgpt, claude, etc.

    None
}
