use dom_query::Document;
use url::Url;

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

pub fn extract(doc: &Document, url: &Url) -> PageMetadata {
    let title = meta_property(doc, "og:title").unwrap_or_else(|| {
        let sel = doc.select("title");
        if sel.exists() {
            sel.text().to_string().trim().to_string()
        } else {
            String::new()
        }
    });
    PageMetadata {
        title,
        author: meta_name(doc, "author").or_else(|| meta_property(doc, "article:author")),
        published: meta_property(doc, "article:published_time").or_else(|| meta_name(doc, "date")),
        site: meta_property(doc, "og:site_name").or_else(|| url.host_str().map(|s| s.to_string())),
        description: meta_name(doc, "description").or_else(|| meta_property(doc, "og:description")),
        image: meta_property(doc, "og:image"),
        language: {
            let sel = doc.select("html");
            sel.attr("lang")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        },
    }
}

fn meta_name(doc: &Document, name: &str) -> Option<String> {
    let sel = doc.select(&format!("meta[name=\"{}\"]", name));
    sel.attr("content")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn meta_property(doc: &Document, prop: &str) -> Option<String> {
    let sel = doc.select(&format!("meta[property=\"{}\"]", prop));
    sel.attr("content")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn extract_schema_org(doc: &Document) -> Option<serde_json::Value> {
    let sel = doc.select("script[type=\"application/ld+json\"]");
    if !sel.exists() {
        return None;
    }
    let text = sel.text().to_string();
    serde_json::from_str(text.trim()).ok()
}
