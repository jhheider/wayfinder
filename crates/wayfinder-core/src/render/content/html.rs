//! HTML tag helpers: tags, attributes, lists, tables, and horizontal rules.

use super::inline::parse_inline;
use super::{ContentBlock, InlineContent};

pub(super) fn extract_level(tag: &str) -> u8 {
    extract_attr(tag, "level")
        .and_then(|l| l.parse().ok())
        .unwrap_or(1)
}

pub(super) fn parse_list_items(content: &str, base_url: &str) -> Vec<Vec<InlineContent>> {
    let mut items = Vec::new();
    let lower = content.to_lowercase();
    let mut pos = 0;

    while pos < content.len() {
        let search = &lower[pos..];
        if let Some(start) = search.find("<li>") {
            let tag_end = pos + start + 4;
            if let Some(end) = lower[tag_end..].find("</li>") {
                let item_html = &content[tag_end..tag_end + end];
                items.push(parse_inline(item_html, base_url));
                pos = tag_end + end + 5;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    items
}

pub(super) fn parse_table(content: &str) -> Option<ContentBlock> {
    let mut headers = Vec::new();
    let mut rows = Vec::new();
    let lower = content.to_lowercase();
    let mut pos = 0;

    while pos < content.len() {
        let search = &lower[pos..];
        if let Some(start) = search.find("<tr>").or_else(|| search.find("<tr ")) {
            let tr_start = pos + start;
            // Find the > of the opening tag
            let tag_end = match content[tr_start..].find('>') {
                Some(e) => tr_start + e + 1,
                None => break,
            };
            if let Some(end) = lower[tag_end..].find("</tr>") {
                let row_html = &content[tag_end..tag_end + end];
                let is_header = lower[tr_start..tag_end + end].contains("<th>");
                let cells = parse_table_cells(row_html);
                if is_header && headers.is_empty() {
                    headers = cells;
                } else if !cells.is_empty() {
                    rows.push(cells);
                }
                pos = tag_end + end + 5;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    if headers.is_empty() && rows.is_empty() {
        None
    } else {
        Some(ContentBlock::Table { headers, rows })
    }
}

pub(super) fn parse_table_cells(content: &str) -> Vec<String> {
    let mut cells = Vec::new();
    let lower = content.to_lowercase();
    let mut pos = 0;

    while pos < content.len() {
        let search = &lower[pos..];
        if let Some(start) = search.find("<th>").or_else(|| search.find("<td>")) {
            let tag_end = start + 4;
            let close_tag = if search[start..].starts_with("<th>") {
                "</th>"
            } else {
                "</td>"
            };
            if let Some(end) = lower[pos + tag_end..].find(close_tag) {
                let cell = &content[pos + tag_end..pos + tag_end + end];
                cells.push(strip_inner_tags(cell).trim().to_string());
                pos = pos + tag_end + end + close_tag.len();
            } else {
                break;
            }
        } else {
            break;
        }
    }
    cells
}

pub(super) fn extract_action_from_html(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let start = lower.find("<actions ")?;
    let tag_end = html[start..].find('>')? + start;
    let tag = &html[start..=tag_end];
    extract_attr(tag, "string")
}

pub(super) fn matches_hrule(chars: &std::iter::Peekable<std::str::Chars<'_>>) -> bool {
    let upcoming: String = chars.clone().take(3).collect();
    upcoming == "---"
}

pub(super) fn consume_tag(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let mut tag = String::new();
    for c in chars.by_ref() {
        tag.push(c);
        if c == '>' {
            break;
        }
    }
    tag
}

pub(super) fn collect_until_close(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    tag_name: &str,
) -> String {
    let close = format!("</{tag_name}>");
    let mut content = String::new();
    let mut depth = 1u32;
    let open_prefix = format!("<{tag_name}");

    while let Some(c) = chars.next() {
        content.push(c);
        if c == '<' {
            let rest: String = chars.clone().take(close.len()).collect();
            let check = format!("<{rest}").to_lowercase();
            if check.starts_with(&close) {
                for _ in 0..close.len() - 1 {
                    chars.next();
                }
                depth -= 1;
                if depth == 0 {
                    content.pop();
                    return content;
                }
            } else if check.starts_with(&open_prefix) {
                depth += 1;
            }
        }
    }
    content
}

pub(super) fn extract_attr(tag: &str, name: &str) -> Option<String> {
    let pattern = format!("{name}=\"");
    let start = tag.find(&pattern)? + pattern.len();
    let rest = &tag[start..];
    let end = rest.find('"')?;
    Some(html_escape::decode_html_entities(&rest[..end]).to_string())
}

pub(super) fn strip_inner_tags(input: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            if c == '[' {
                let mut text = String::new();
                for lc in chars.by_ref() {
                    if lc == ']' {
                        break;
                    }
                    text.push(lc);
                }
                if chars.peek() == Some(&'(') {
                    chars.next();
                    let mut depth = 1;
                    for lc in chars.by_ref() {
                        if lc == '(' {
                            depth += 1;
                        } else if lc == ')' {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                    }
                }
                out.push_str(&text);
            } else {
                out.push(c);
            }
        }
    }
    html_escape::decode_html_entities(&out).to_string()
}
