pub mod github;
use dom_query::Document;
use url::Url;

pub struct ExtractorResult {
    pub content_html: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub published: Option<String>,
    pub site: Option<String>,
}

pub fn try_extract(doc: &Document, url: &Url) -> Option<ExtractorResult> {
    let domain = url.host_str()?;
    if domain.contains("github.com") {
        return github::extract(doc, url);
    }
    None
}
