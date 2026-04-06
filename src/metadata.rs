//! Metadata extraction — title, author, date, description, schema.org.

use scraper::{Html, Selector};
use url::Url;

/// Extracted metadata from a page.
#[derive(Debug, Clone, Default)]
pub struct PageMetadata {
    pub title: String,
    pub author: Option<String>,
    pub published: Option<String>,
    pub site: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub language: Option<String>,
}

/// Extract metadata from HTML document.
pub fn extract(document: &Html, url: &Url) -> PageMetadata {
    let title = extract_title(document);
    let author = extract_meta(document, "author")
        .or_else(|| extract_meta_property(document, "article:author"));
    let published = extract_meta_property(document, "article:published_time")
        .or_else(|| extract_meta(document, "date"))
        .or_else(|| extract_meta(document, "pubdate"));
    let site = extract_meta_property(document, "og:site_name")
        .or_else(|| Some(url.host_str()?.to_string()));
    let description = extract_meta(document, "description")
        .or_else(|| extract_meta_property(document, "og:description"));
    let image = extract_meta_property(document, "og:image")
        .or_else(|| extract_meta(document, "twitter:image"));
    let language = extract_html_lang(document);

    PageMetadata {
        title,
        author,
        published,
        site,
        description,
        image,
        language,
    }
}

fn extract_title(document: &Html) -> String {
    // Try og:title first, then <title> tag
    if let Some(og) = extract_meta_property(document, "og:title") {
        return og;
    }
    let sel = Selector::parse("title").unwrap();
    document
        .select(&sel)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_default()
}

fn extract_meta(document: &Html, name: &str) -> Option<String> {
    let sel = Selector::parse(&format!("meta[name=\"{}\"]", name)).ok()?;
    document
        .select(&sel)
        .next()
        .and_then(|el| el.value().attr("content"))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn extract_meta_property(document: &Html, property: &str) -> Option<String> {
    let sel = Selector::parse(&format!("meta[property=\"{}\"]", property)).ok()?;
    document
        .select(&sel)
        .next()
        .and_then(|el| el.value().attr("content"))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn extract_html_lang(document: &Html) -> Option<String> {
    let sel = Selector::parse("html").ok()?;
    document
        .select(&sel)
        .next()
        .and_then(|el| el.value().attr("lang"))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Extract schema.org JSON-LD data from script tags.
pub fn extract_schema_org(html: &str) -> Option<serde_json::Value> {
    let document = Html::parse_document(html);
    let sel = Selector::parse("script[type=\"application/ld+json\"]").ok()?;
    for el in document.select(&sel) {
        let text: String = el.text().collect();
        let trimmed = text.trim();
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
            return Some(val);
        }
    }
    None
}
