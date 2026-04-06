//! GitHub extractor — extract README, issue, PR, or file content from GitHub pages.

use scraper::{Html, Selector};
use url::Url;

use super::ExtractorResult;

pub fn extract(document: &Html, url: &Url) -> Option<ExtractorResult> {
    let path = url.path();

    // README / repo root
    if let Some(content) = extract_readme(document) {
        return Some(ExtractorResult {
            content_html: content,
            title: extract_repo_title(document),
            author: None,
            published: None,
            site: Some("GitHub".into()),
        });
    }

    // Issue or PR
    if path.contains("/issues/") || path.contains("/pull/") {
        if let Some(content) = extract_issue_or_pr(document) {
            return Some(ExtractorResult {
                content_html: content,
                title: extract_page_title(document),
                author: extract_author(document),
                published: None,
                site: Some("GitHub".into()),
            });
        }
    }

    None
}

fn extract_readme(document: &Html) -> Option<String> {
    let sel = Selector::parse("article.markdown-body").ok()?;
    document.select(&sel).next().map(|el| el.html())
}

fn extract_issue_or_pr(document: &Html) -> Option<String> {
    // Issue/PR body
    let sel = Selector::parse(".comment-body").ok()?;
    let comments: Vec<String> = document.select(&sel).map(|el| el.html()).collect();
    if comments.is_empty() {
        return None;
    }
    Some(comments.join("\n<hr>\n"))
}

fn extract_repo_title(document: &Html) -> Option<String> {
    let sel = Selector::parse("[itemprop=\"name\"] a, .AppHeader-context-item-label").ok()?;
    document
        .select(&sel)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
}

fn extract_page_title(document: &Html) -> Option<String> {
    let sel = Selector::parse(".js-issue-title, .gh-header-title").ok()?;
    document
        .select(&sel)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
}

fn extract_author(document: &Html) -> Option<String> {
    let sel = Selector::parse(".author").ok()?;
    document
        .select(&sel)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
}
