use crate::constants::*;
use dom_query::{Document, NodeRef, Selection};
use regex::Regex;

/// Run the full clutter removal pipeline.
/// `main_content_id` is the NodeId of the main content element — ancestors are protected.
pub fn remove_clutter(doc: &Document, main_content: &Selection) {
    let main_id = main_content.nodes().first().map(|n| n.id);
    remove_hidden(doc, main_id);
    remove_by_exact_selector(doc, main_id);
    remove_by_partial_selector(doc, main_id);
    score_and_remove(doc, main_id);
}

/// Check if `node` is an ancestor of (contains) the main content element.
fn contains_main(node: &NodeRef, main_id: Option<dom_query::NodeId>) -> bool {
    let Some(mid) = main_id else {
        return false;
    };
    // Check if main_id is a descendant of this node
    for desc in node.descendants().iter() {
        if desc.id == mid {
            return true;
        }
    }
    false
}

fn remove_hidden(doc: &Document, main_id: Option<dom_query::NodeId>) {
    for node in doc.select("[style]").nodes().iter() {
        if contains_main(node, main_id) {
            continue;
        }
        let style = node
            .attr("style")
            .unwrap_or_default()
            .to_string()
            .to_lowercase();
        if style.contains("display:none")
            || style.contains("display: none")
            || style.contains("visibility:hidden")
            || style.contains("visibility: hidden")
            || style.contains("opacity:0")
            || style.contains("opacity: 0")
        {
            if !Selection::from(node.clone())
                .select("math, [data-mathml], .katex-mathml")
                .exists()
            {
                node.remove_from_parent();
            }
        }
    }
    for node in doc.select(".hidden, .invisible").nodes().iter() {
        if contains_main(node, main_id) {
            continue;
        }
        // Don't remove hidden math elements
        let class = node.attr("class").unwrap_or_default().to_string();
        if class.contains("math") {
            continue;
        }
        node.remove_from_parent();
    }
}

fn remove_by_exact_selector(doc: &Document, main_id: Option<dom_query::NodeId>) {
    for sel_str in EXACT_SELECTORS {
        if let Some(sel) = doc.try_select(sel_str) {
            for node in sel.nodes().iter() {
                if contains_main(node, main_id) {
                    continue;
                }
                if is_inside_pre(node) {
                    continue;
                }
                if is_footnote_related(node) {
                    continue;
                }
                node.remove_from_parent();
            }
        }
    }
}

fn remove_by_partial_selector(doc: &Document, main_id: Option<dom_query::NodeId>) {
    let partial_re = build_partial_regex();
    for node in doc
        .select("[class], [id], [data-component], [data-testid]")
        .nodes()
        .iter()
    {
        if contains_main(node, main_id) {
            continue;
        }
        if is_inside_pre(node) {
            continue;
        }

        let name = node
            .node_name()
            .unwrap_or_default()
            .to_string()
            .to_lowercase();
        let is_heading = matches!(name.as_str(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6");

        let class = node
            .attr("class")
            .unwrap_or_default()
            .to_string()
            .to_lowercase();
        let id = if is_heading {
            String::new()
        } else {
            node.attr("id")
                .unwrap_or_default()
                .to_string()
                .to_lowercase()
        };
        let dc = node
            .attr("data-component")
            .unwrap_or_default()
            .to_string()
            .to_lowercase();
        let dt = node
            .attr("data-testid")
            .unwrap_or_default()
            .to_string()
            .to_lowercase();
        let attrs = format!("{} {} {} {}", class, id, dc, dt);
        if attrs.trim().is_empty() {
            continue;
        }

        if partial_re.is_match(&attrs) {
            if is_footnote_related(node) {
                continue;
            }
            node.remove_from_parent();
        }
    }
}

fn build_partial_regex() -> Regex {
    let escaped: Vec<String> = PARTIAL_SELECTORS.iter().map(|p| regex::escape(p)).collect();
    Regex::new(&format!("(?i){}", escaped.join("|"))).unwrap()
}

fn score_and_remove(doc: &Document, main_id: Option<dom_query::NodeId>) {
    let block_sel = BLOCK_ELEMENTS.join(", ");
    for node in doc.select(&block_sel).nodes().iter() {
        if contains_main(node, main_id) {
            continue;
        }
        if is_inside_pre(node) {
            continue;
        }
        if is_likely_content(node) {
            continue;
        }
        if is_footnote_related(node) {
            continue;
        }

        let score = score_non_content(node);
        if score < 0.0 {
            node.remove_from_parent();
        }
    }
}

fn score_non_content(node: &NodeRef) -> f64 {
    let text = node.text().to_string();
    let words = text.split_whitespace().count();
    if words < 3 {
        return 0.0;
    }
    let mut score = text.matches(',').count() as f64;

    let text_lower = text.to_lowercase();
    for ind in NAVIGATION_INDICATORS {
        if text_lower.contains(ind) {
            score -= 10.0;
        }
    }

    let link_count = Selection::from(node.clone()).select("a").length();
    if link_count as f64 / words.max(1) as f64 > 0.5 {
        score -= 15.0;
    }

    let class = node
        .attr("class")
        .unwrap_or_default()
        .to_string()
        .to_lowercase();
    let id = node
        .attr("id")
        .unwrap_or_default()
        .to_string()
        .to_lowercase();
    for p in &[
        "advert",
        "ad-",
        "ads",
        "banner",
        "cookie",
        "copyright",
        "footer",
        "header",
        "menu",
        "nav",
        "newsletter",
        "popular",
        "privacy",
        "recommended",
        "related",
        "share",
        "sidebar",
        "social",
        "sponsored",
        "subscribe",
        "terms",
        "trending",
        "widget",
    ] {
        if class.contains(p) || id.contains(p) {
            score -= 8.0;
        }
    }

    let tag = node
        .node_name()
        .unwrap_or_default()
        .to_string()
        .to_lowercase();
    match tag.as_str() {
        "nav" | "footer" | "header" | "aside" => score -= 30.0,
        "form" => score -= 15.0,
        _ => {}
    }
    score
}

fn is_likely_content(node: &NodeRef) -> bool {
    let class = node
        .attr("class")
        .unwrap_or_default()
        .to_string()
        .to_lowercase();
    let id = node
        .attr("id")
        .unwrap_or_default()
        .to_string()
        .to_lowercase();
    let role = node
        .attr("role")
        .unwrap_or_default()
        .to_string()
        .to_lowercase();

    if matches!(role.as_str(), "article" | "main" | "contentinfo") {
        return true;
    }
    for ind in CONTENT_INDICATORS {
        if class.contains(ind) || id.contains(ind) {
            return true;
        }
    }
    if Selection::from(node.clone())
        .select(CONTENT_ELEMENT_SELECTOR)
        .exists()
    {
        return true;
    }

    let words = node.text().split_whitespace().count();
    let p_count = Selection::from(node.clone()).select("p").length();
    let li_count = Selection::from(node.clone()).select("li").length();
    if words > 50 && (p_count + li_count) > 1 {
        return true;
    }
    if words > 100 {
        return true;
    }
    if words > 30 && (p_count + li_count) > 0 {
        return true;
    }
    false
}

fn is_inside_pre(node: &NodeRef) -> bool {
    for a in node.ancestors(Some(10)).iter() {
        let name = a.node_name().unwrap_or_default().to_string().to_lowercase();
        if name == "pre" || name == "code" {
            return true;
        }
    }
    false
}

fn is_footnote_related(node: &NodeRef) -> bool {
    let class = node
        .attr("class")
        .unwrap_or_default()
        .to_string()
        .to_lowercase();
    let id = node
        .attr("id")
        .unwrap_or_default()
        .to_string()
        .to_lowercase();
    if class.contains("footnote")
        || class.contains("reference")
        || class.contains("reflist")
        || id.contains("footnote")
        || id.contains("reference")
        || id.contains("reflist")
    {
        return true;
    }
    for a in node.ancestors(Some(3)).iter() {
        let c = a
            .attr("class")
            .unwrap_or_default()
            .to_string()
            .to_lowercase();
        let i = a.attr("id").unwrap_or_default().to_string().to_lowercase();
        if c.contains("footnote")
            || c.contains("reference")
            || c.contains("reflist")
            || i.contains("footnote")
            || i.contains("reference")
            || i.contains("reflist")
        {
            return true;
        }
    }
    false
}
