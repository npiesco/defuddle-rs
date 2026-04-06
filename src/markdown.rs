//! HTML → Markdown conversion.
//!
//! Clean-room implementation inspired by Turndown.js patterns.
//! Converts cleaned HTML to readable markdown.

use scraper::{ElementRef, Html, Node, Selector};

/// Convert HTML to clean Markdown.
pub fn html_to_markdown(html: &str) -> String {
    let fragment = Html::parse_fragment(html);
    let mut output = String::new();
    convert_node(&fragment.root_element(), &mut output, 0);
    collapse_whitespace(&output)
}

fn convert_node(element: &ElementRef, output: &mut String, depth: usize) {
    for child in element.children() {
        match child.value() {
            Node::Text(text) => {
                output.push_str(&text.text);
            }
            Node::Element(el) => {
                if let Some(child_ref) = ElementRef::wrap(child) {
                    let tag = el.name.local.as_ref();
                    match tag {
                        "h1" => {
                            output.push_str("\n\n# ");
                            convert_node(&child_ref, output, depth);
                            output.push_str("\n\n");
                        }
                        "h2" => {
                            output.push_str("\n\n## ");
                            convert_node(&child_ref, output, depth);
                            output.push_str("\n\n");
                        }
                        "h3" => {
                            output.push_str("\n\n### ");
                            convert_node(&child_ref, output, depth);
                            output.push_str("\n\n");
                        }
                        "h4" => {
                            output.push_str("\n\n#### ");
                            convert_node(&child_ref, output, depth);
                            output.push_str("\n\n");
                        }
                        "h5" => {
                            output.push_str("\n\n##### ");
                            convert_node(&child_ref, output, depth);
                            output.push_str("\n\n");
                        }
                        "h6" => {
                            output.push_str("\n\n###### ");
                            convert_node(&child_ref, output, depth);
                            output.push_str("\n\n");
                        }
                        "p" => {
                            output.push_str("\n\n");
                            convert_node(&child_ref, output, depth);
                            output.push_str("\n\n");
                        }
                        "br" => {
                            output.push('\n');
                        }
                        "hr" => {
                            output.push_str("\n\n---\n\n");
                        }
                        "strong" | "b" => {
                            output.push_str("**");
                            convert_node(&child_ref, output, depth);
                            output.push_str("**");
                        }
                        "em" | "i" => {
                            output.push('*');
                            convert_node(&child_ref, output, depth);
                            output.push('*');
                        }
                        "code" => {
                            // Check if inside a <pre> — if so, handled by pre
                            if depth > 0 {
                                output.push('`');
                                convert_node(&child_ref, output, depth);
                                output.push('`');
                            } else {
                                output.push('`');
                                convert_node(&child_ref, output, depth);
                                output.push('`');
                            }
                        }
                        "pre" => {
                            let lang = child_ref
                                .select(&Selector::parse("code").unwrap())
                                .next()
                                .and_then(|code| {
                                    code.value().attr("class").and_then(|c| {
                                        c.split_whitespace()
                                            .find(|cls| cls.starts_with("language-"))
                                            .map(|cls| cls.strip_prefix("language-").unwrap_or(""))
                                    })
                                })
                                .unwrap_or("");
                            output.push_str(&format!("\n\n```{}\n", lang));
                            let text: String = child_ref.text().collect();
                            output.push_str(text.trim());
                            output.push_str("\n```\n\n");
                        }
                        "a" => {
                            let href = el.attr("href").unwrap_or("");
                            if href.is_empty() || href.starts_with("javascript:") {
                                convert_node(&child_ref, output, depth);
                            } else {
                                output.push('[');
                                convert_node(&child_ref, output, depth);
                                output.push_str(&format!("]({})", href));
                            }
                        }
                        "img" => {
                            let src = el.attr("src").unwrap_or("");
                            let alt = el.attr("alt").unwrap_or("");
                            if !src.is_empty() {
                                output.push_str(&format!("![{}]({})", alt, src));
                            }
                        }
                        "ul" => {
                            output.push_str("\n\n");
                            convert_list(&child_ref, output, depth, false);
                            output.push('\n');
                        }
                        "ol" => {
                            output.push_str("\n\n");
                            convert_list(&child_ref, output, depth, true);
                            output.push('\n');
                        }
                        "blockquote" => {
                            output.push_str("\n\n");
                            let inner = {
                                let mut s = String::new();
                                convert_node(&child_ref, &mut s, depth + 1);
                                s
                            };
                            for line in inner.trim().lines() {
                                output.push_str(&format!("> {}\n", line));
                            }
                            output.push('\n');
                        }
                        "table" => {
                            output.push_str("\n\n");
                            convert_table(&child_ref, output);
                            output.push('\n');
                        }
                        "div" | "section" | "article" | "main" | "span" | "figure"
                        | "figcaption" => {
                            convert_node(&child_ref, output, depth);
                        }
                        "script" | "style" | "nav" | "footer" | "header" => {
                            // Skip
                        }
                        _ => {
                            convert_node(&child_ref, output, depth);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn convert_list(element: &ElementRef, output: &mut String, depth: usize, ordered: bool) {
    let li_sel = Selector::parse("li").unwrap();
    let indent = "  ".repeat(depth);
    for (i, li) in element.select(&li_sel).enumerate() {
        let prefix = if ordered {
            format!("{}{}. ", indent, i + 1)
        } else {
            format!("{}- ", indent)
        };
        output.push_str(&prefix);
        let mut inner = String::new();
        convert_node(&li, &mut inner, depth + 1);
        let trimmed = inner.trim();
        // Indent continuation lines
        for (j, line) in trimmed.lines().enumerate() {
            if j > 0 {
                output.push_str(&format!("\n{}  {}", indent, line));
            } else {
                output.push_str(line);
            }
        }
        output.push('\n');
    }
}

fn convert_table(element: &ElementRef, output: &mut String) {
    let tr_sel = Selector::parse("tr").unwrap();
    let th_sel = Selector::parse("th").unwrap();
    let td_sel = Selector::parse("td").unwrap();

    let rows: Vec<ElementRef> = element.select(&tr_sel).collect();
    if rows.is_empty() {
        return;
    }

    for (i, row) in rows.iter().enumerate() {
        let cells: Vec<String> = row
            .select(&th_sel)
            .chain(row.select(&td_sel))
            .map(|cell| {
                let text: String = cell.text().collect();
                text.trim().replace('|', "\\|").to_string()
            })
            .collect();

        if cells.is_empty() {
            continue;
        }

        output.push_str(&format!("| {} |\n", cells.join(" | ")));

        // Add separator after header row
        if i == 0 {
            let sep: Vec<&str> = vec!["---"; cells.len()];
            output.push_str(&format!("| {} |\n", sep.join(" | ")));
        }
    }
}

fn collapse_whitespace(text: &str) -> String {
    // Replace any run of 3+ newlines with exactly 2
    let mut result = String::with_capacity(text.len());
    let mut newline_count = 0usize;

    for ch in text.chars() {
        if ch == '\n' {
            newline_count += 1;
            if newline_count <= 2 {
                result.push(ch);
            }
        } else {
            newline_count = 0;
            result.push(ch);
        }
    }

    // Also trim trailing whitespace per line and overall
    result
        .lines()
        .map(|l| l.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}
