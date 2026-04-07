use super::ExtractorResult;
use dom_query::Document;
use url::Url;

pub fn extract(doc: &Document, _url: &Url) -> Option<ExtractorResult> {
    let readme = doc.select("article.markdown-body");
    if readme.exists() {
        return Some(ExtractorResult {
            content_html: readme.html().to_string(),
            title: {
                let s = doc.select("[itemprop=\"name\"] a");
                if s.exists() {
                    Some(s.text().to_string().trim().to_string())
                } else {
                    None
                }
            },
            author: None,
            published: None,
            site: Some("GitHub".into()),
        });
    }
    let comments = doc.select(".comment-body");
    if comments.exists() {
        let html: Vec<String> = comments
            .nodes()
            .iter()
            .map(|n| n.html().to_string())
            .collect();
        return Some(ExtractorResult {
            content_html: html.join("\n<hr>\n"),
            title: {
                let s = doc.select(".js-issue-title, .gh-header-title");
                if s.exists() {
                    Some(s.text().to_string().trim().to_string())
                } else {
                    None
                }
            },
            author: {
                let s = doc.select(".author");
                if s.exists() {
                    Some(s.text().to_string().trim().to_string())
                } else {
                    None
                }
            },
            published: None,
            site: Some("GitHub".into()),
        });
    }
    None
}
