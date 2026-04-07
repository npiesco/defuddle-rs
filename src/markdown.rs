use dom_query::Selection;
use dom_query::{Document, NodeRef};
use regex::Regex;

pub fn convert(html: &str) -> String {
    let doc = Document::fragment(html);
    let mut output = String::new();
    for child in doc.root().children().iter() {
        convert_node(child, &mut output);
    }
    let collapsed = collapse_whitespace(&output);
    let escaped = escape_list_markers(&collapsed);
    fix_inline_code_spacing(&escaped)
}

fn convert_node(node: &NodeRef, output: &mut String) {
    if node.is_text() {
        output.push_str(&node.text().to_string());
        return;
    }
    if !node.is_element() {
        return;
    }

    let tag = node
        .node_name()
        .unwrap_or_default()
        .to_string()
        .to_lowercase();
    match tag.as_str() {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            let prefix = match tag.as_str() {
                "h1" => "#",
                "h2" => "##",
                "h3" => "###",
                "h4" => "####",
                "h5" => "#####",
                "h6" => "######",
                _ => "#",
            };
            let text = node.text().to_string();
            let clean = text.split_whitespace().collect::<Vec<_>>().join(" ");
            if !clean.is_empty() {
                output.push_str(&format!("\n\n{} {}\n\n", prefix, clean));
            }
        }
        "p" => {
            output.push_str("\n\n");
            convert_children(node, output);
            output.push_str("\n\n");
        }
        "br" => output.push('\n'),
        "hr" => output.push_str("\n\n---\n\n"),
        "strong" | "b" => {
            output.push_str("**");
            convert_children(node, output);
            output.push_str("**");
        }
        "em" | "i" => {
            output.push('*');
            convert_children(node, output);
            output.push('*');
        }
        "code" => {
            output.push('`');
            convert_children(node, output);
            output.push('`');
        }
        "pre" => {
            let lang = Selection::from(node.clone())
                .select("code")
                .attr("class")
                .map(|c| {
                    let cs = c.to_string();
                    cs.split_whitespace()
                        .find(|s| s.starts_with("language-"))
                        .map(|s| s.strip_prefix("language-").unwrap_or("").to_string())
                        .unwrap_or_default()
                })
                .unwrap_or_default();
            output.push_str(&format!(
                "\n\n```{}\n{}\n```\n\n",
                lang,
                node.text().to_string().trim()
            ));
        }
        "a" => {
            let href = node.attr("href").unwrap_or_default().to_string();
            let aria = node
                .attr("aria-hidden")
                .map(|v| v.to_string() == "true")
                .unwrap_or(false);
            if href.is_empty() || href.starts_with("javascript:") || aria {
                convert_children(node, output);
            } else if Selection::from(node.clone())
                .select("p, div, ul, ol, table, blockquote, section, article")
                .exists()
            {
                convert_children(node, output);
            } else {
                let mut link_text = String::new();
                convert_children(node, &mut link_text);
                let trimmed = link_text.trim();
                if !trimmed.is_empty() {
                    let title = node.attr("title").unwrap_or_default().to_string();
                    if title.is_empty() {
                        output.push_str(&format!("[{}]({})", trimmed, href));
                    } else {
                        output.push_str(&format!("[{}]({} \"{}\")", trimmed, href, title));
                    }
                }
            }
        }
        "img" => {
            let src = node.attr("src").unwrap_or_default().to_string();
            let alt = node.attr("alt").unwrap_or_default().to_string();
            if !src.is_empty() {
                output.push_str(&format!("![{}]({})", alt, src));
            }
        }
        "ul" => {
            output.push_str("\n\n");
            convert_list(node, output, 0, false);
            output.push('\n');
        }
        "ol" => {
            output.push_str("\n\n");
            convert_list(node, output, 0, true);
            output.push('\n');
        }
        "blockquote" => {
            output.push_str("\n\n");
            let mut inner = String::new();
            convert_children(node, &mut inner);
            for line in inner.trim().lines() {
                output.push_str(&format!("> {}\n", line));
            }
            output.push('\n');
        }
        "table" => {
            output.push_str("\n\n");
            convert_table(node, output);
            output.push('\n');
        }
        "figcaption" => {
            let text = node.text().to_string();
            let clean = text.split_whitespace().collect::<Vec<_>>().join(" ");
            if !clean.is_empty() {
                output.push_str(&clean);
            }
        }
        "script" | "style" | "nav" | "noscript" | "iframe" => {}
        _ => convert_children(node, output),
    }
}

fn convert_children(node: &NodeRef, output: &mut String) {
    for child in node.children().iter() {
        convert_node(child, output);
    }
}

fn convert_list(node: &NodeRef, output: &mut String, depth: usize, ordered: bool) {
    let indent = "  ".repeat(depth);
    let mut idx = 0;
    for child in node.children().iter() {
        let tag = child
            .node_name()
            .unwrap_or_default()
            .to_string()
            .to_lowercase();
        if tag != "li" {
            continue;
        }
        idx += 1;
        let prefix = if ordered {
            format!("{}{}. ", indent, idx)
        } else {
            format!("{}- ", indent)
        };
        output.push_str(&prefix);
        let mut inner = String::new();
        convert_children(child, &mut inner);
        for (j, line) in inner.trim().lines().enumerate() {
            if j > 0 {
                output.push_str(&format!("\n{}  {}", indent, line));
            } else {
                output.push_str(line);
            }
        }
        output.push('\n');
    }
}

fn convert_table(node: &NodeRef, output: &mut String) {
    let mut row_idx = 0;
    for tr in node.descendants().iter() {
        let tag = tr
            .node_name()
            .unwrap_or_default()
            .to_string()
            .to_lowercase();
        if tag != "tr" {
            continue;
        }
        let cells: Vec<String> = tr
            .children()
            .iter()
            .filter(|c| {
                let t = c.node_name().unwrap_or_default().to_string().to_lowercase();
                t == "th" || t == "td"
            })
            .map(|cell| {
                let mut md = String::new();
                convert_children(&cell, &mut md);
                md.trim()
                    .replace('\n', " ")
                    .replace('|', "\\|")
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect();
        if cells.is_empty() {
            continue;
        }
        output.push_str(&format!("| {} |\n", cells.join(" | ")));
        if row_idx == 0 {
            output.push_str(&format!("| {} |\n", vec!["---"; cells.len()].join(" | ")));
        }
        row_idx += 1;
    }
}

fn collapse_whitespace(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut nc = 0usize;
    for ch in text.chars() {
        if ch == '\n' {
            nc += 1;
            if nc <= 2 {
                result.push(ch);
            }
        } else {
            nc = 0;
            result.push(ch);
        }
    }
    result
        .lines()
        .map(|l| l.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn escape_list_markers(text: &str) -> String {
    Regex::new(r"\*\*(\d+)\. ")
        .unwrap()
        .replace_all(text, r"**$1\. ")
        .to_string()
}

fn fix_inline_code_spacing(text: &str) -> String {
    let re = Regex::new(r"([^`])`(s\b|es\b|ed\b|ing\b|'s\b)").unwrap();
    let mut in_fence = false;
    let mut result = String::with_capacity(text.len());
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
        }
        if in_fence {
            result.push_str(line);
        } else {
            result.push_str(&re.replace_all(line, "$1` $2"));
        }
        result.push('\n');
    }
    if result.ends_with('\n') && !text.ends_with('\n') {
        result.pop();
    }
    result
}
