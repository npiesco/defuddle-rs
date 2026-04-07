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
    let collapsed = collapse_whitespace(&output);
    let escaped = escape_list_markers(&collapsed);
    fix_inline_code_spacing(&escaped)
}

/// Escape numbered list markers (e.g. `1.` → `1\.`) that appear inside emphasis
/// or at the start of lines where they weren't intended as lists.
fn escape_list_markers(text: &str) -> String {
    use regex::Regex;
    let re = Regex::new(r"\*\*(\d+)\. ").unwrap();
    re.replace_all(text, r"**$1\. ").to_string()
}

/// Add space between closing inline code backtick and immediately following
/// grammatical suffix (s, es, ed, ing, 's). Matches Turndown/defuddle behavior.
/// Skips lines inside fenced code blocks.
fn fix_inline_code_spacing(text: &str) -> String {
    use regex::Regex;
    // Only match: code backtick followed by s/es/ed/ing/'s (grammatical suffixes)
    let re = Regex::new(r"([^`])`(s\b|es\b|ed\b|ing\b|'s\b)").unwrap();
    let mut in_fence = false;
    let mut result = String::with_capacity(text.len());

    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            result.push_str(line);
        } else if in_fence {
            result.push_str(line);
        } else {
            let fixed = re.replace_all(line, "$1` $2");
            result.push_str(&fixed);
        }
        result.push('\n');
    }

    if result.ends_with('\n') && !text.ends_with('\n') {
        result.pop();
    }
    result
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
                        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                            let prefix = match tag {
                                "h1" => "#",
                                "h2" => "##",
                                "h3" => "###",
                                "h4" => "####",
                                "h5" => "#####",
                                "h6" => "######",
                                _ => "#",
                            };
                            // Collect heading text, collapsing whitespace
                            let heading_text: String = child_ref.text().collect();
                            let clean = heading_text
                                .split_whitespace()
                                .collect::<Vec<_>>()
                                .join(" ");
                            if !clean.is_empty() {
                                output.push_str(&format!("\n\n{} {}\n\n", prefix, clean));
                            }
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
                            let aria_hidden = el.attr("aria-hidden").unwrap_or("") == "true";
                            // Skip anchor-only links and aria-hidden anchors
                            if href.is_empty() || href.starts_with("javascript:") || aria_hidden {
                                convert_node(&child_ref, output, depth);
                            } else {
                                // Check if this link wraps block-level content
                                let has_blocks = Selector::parse("p, div, ul, ol, h1, h2, h3, h4, h5, h6, blockquote, pre, table")
                                    .ok()
                                    .map(|s| child_ref.select(&s).next().is_some())
                                    .unwrap_or(false);
                                if has_blocks {
                                    // Block-level link — treat as transparent wrapper
                                    convert_node(&child_ref, output, depth);
                                } else {
                                    let link_text = {
                                        let mut s = String::new();
                                        convert_node(&child_ref, &mut s, depth);
                                        s
                                    };
                                    let trimmed = link_text.trim();
                                    if trimmed.is_empty() {
                                        // Empty link text — skip
                                    } else {
                                        let title = el.attr("title").unwrap_or("");
                                        if title.is_empty() {
                                            output.push_str(&format!("[{}]({})", trimmed, href));
                                        } else {
                                            output.push_str(&format!(
                                                "[{}]({} \"{}\")",
                                                trimmed, href, title
                                            ));
                                        }
                                    }
                                }
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
                        "div" | "section" | "article" | "main" | "span" | "figure" | "sup"
                        | "sub" | "small" | "mark" | "del" | "ins" | "abbr" | "cite" | "time"
                        | "details" | "summary" | "dl" | "dt" | "dd" | "address" => {
                            convert_node(&child_ref, output, depth);
                        }
                        "figcaption" => {
                            let text: String = child_ref.text().collect();
                            let clean = text.split_whitespace().collect::<Vec<_>>().join(" ");
                            if !clean.is_empty() {
                                output.push_str(&clean);
                            }
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
                // Convert cell contents to markdown (preserving links, code, etc.)
                let mut cell_md = String::new();
                convert_node(&cell, &mut cell_md, 0);
                let cleaned = cell_md.trim().replace('\n', " ").replace('|', "\\|");
                // Collapse multiple spaces
                cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
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
