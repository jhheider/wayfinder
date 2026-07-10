//! Structured content IR for AON documents.
//! Parses AON HTML/markdown into `ContentBlock` trees.

use serde::Serialize;

/// A block-level content element.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Title {
        level: u8,
        text: String,
        right: Option<String>,
        action: Option<String>,
    },
    Paragraph {
        content: Vec<InlineContent>,
    },
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    List {
        items: Vec<Vec<InlineContent>>,
    },
    KeyValue {
        key: String,
        value: Vec<InlineContent>,
    },
    Aside {
        title: Option<String>,
        content: Vec<ContentBlock>,
    },
    HRule,
}

/// Inline content within a paragraph or list item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InlineContent {
    Text { text: String },
    Bold { text: String },
    Italic { text: String },
    Link { text: String, url: String },
    Trait { label: String },
    Action { text: String },
}

/// Parse AON markdown/HTML content into structured blocks.
/// `base_url` is used to resolve relative URLs (e.g. `https://2e.aonprd.com`).
pub fn parse_content(input: &str, base_url: &str) -> Vec<ContentBlock> {
    let mut blocks = Vec::new();
    let mut chars = input.chars().peekable();
    let mut inline_buf: Vec<InlineContent> = Vec::new();

    while chars.peek().is_some() {
        let c = *chars.peek().unwrap();
        // Check for --- hrule at start of line (after flush or start of input)
        if c == '-' && inline_buf.is_empty() && matches_hrule(&chars) {
            chars.next(); // -
            chars.next(); // -
            chars.next(); // -
            // Consume rest of line
            while chars.peek().is_some_and(|&ch| ch != '\n') {
                chars.next();
            }
            blocks.push(ContentBlock::HRule);
            continue;
        }
        if c == '<' {
            let tag = consume_tag(&mut chars);
            let lower = tag.to_lowercase();

            if lower.starts_with("<title") {
                flush_paragraph(&mut inline_buf, &mut blocks);
                let content = collect_until_close(&mut chars, "title");
                let level = extract_level(&tag);
                let right = extract_attr(&tag, "right");
                let action = extract_action_from_html(&content);
                let clean = strip_inner_tags(&content);
                if !clean.trim().is_empty() {
                    blocks.push(ContentBlock::Title {
                        level,
                        text: clean.trim().to_string(),
                        right,
                        action,
                    });
                }
            } else if lower.starts_with("<trait ") {
                if let Some(label) = extract_attr(&tag, "label") {
                    inline_buf.push(InlineContent::Trait { label });
                }
            } else if lower.starts_with("<actions") {
                if let Some(s) = extract_attr(&tag, "string") {
                    inline_buf.push(InlineContent::Action { text: s });
                }
            } else if lower == "<hr>" || lower == "<hr />" {
                flush_paragraph(&mut inline_buf, &mut blocks);
                blocks.push(ContentBlock::HRule);
            } else if lower.starts_with("<br") {
                // Treat <br> as paragraph break if we have content
                if !inline_buf.is_empty() {
                    flush_paragraph(&mut inline_buf, &mut blocks);
                }
            } else if lower.starts_with("<aside") {
                flush_paragraph(&mut inline_buf, &mut blocks);
                let aside_content = collect_until_close(&mut chars, "aside");
                let title = extract_attr(&tag, "title");
                let inner = parse_content(&aside_content, base_url);
                blocks.push(ContentBlock::Aside {
                    title,
                    content: inner,
                });
            } else if lower == "<ul>" || lower == "<ol>" {
                flush_paragraph(&mut inline_buf, &mut blocks);
                let list_tag = if lower == "<ul>" { "ul" } else { "ol" };
                let list_content = collect_until_close(&mut chars, list_tag);
                let items = parse_list_items(&list_content, base_url);
                if !items.is_empty() {
                    blocks.push(ContentBlock::List { items });
                }
            } else if lower == "<li>" {
                // Bare <li> without <ul> wrapper
                let content = collect_until_close(&mut chars, "li");
                let inlines = parse_inline(&content, base_url);
                if !inlines.is_empty() {
                    flush_paragraph(&mut inline_buf, &mut blocks);
                    blocks.push(ContentBlock::List {
                        items: vec![inlines],
                    });
                }
            } else if lower == "<table>" || lower.starts_with("<table ") {
                flush_paragraph(&mut inline_buf, &mut blocks);
                let table_content = collect_until_close(&mut chars, "table");
                if let Some(table) = parse_table(&table_content) {
                    blocks.push(table);
                }
            } else if lower.starts_with("<tr>") || lower.starts_with("<tr ") {
                // Bare <tr> without <table> wrapper
                flush_paragraph(&mut inline_buf, &mut blocks);
                let row_content = collect_until_close(&mut chars, "tr");
                let cells = parse_table_cells(&row_content);
                if !cells.is_empty() {
                    blocks.push(ContentBlock::Table {
                        headers: vec![],
                        rows: vec![cells],
                    });
                }
            } else if lower.starts_with("</") {
                // closing tags -- ignored
            }
            // Other tags silently ignored
        } else if c == '[' {
            let (text, url) = consume_link(&mut chars);
            let url = resolve_url(&url, base_url);
            inline_buf.push(InlineContent::Link { text, url });
        } else if c == '*' && chars.clone().nth(1) == Some('*') {
            let text = consume_bold(&mut chars);
            inline_buf.push(InlineContent::Bold { text });
        } else if c == '_' && chars.clone().nth(1).is_some_and(|n| n.is_alphabetic()) {
            let text = consume_italic(&mut chars);
            inline_buf.push(InlineContent::Italic { text });
        } else if c == '\r' {
            chars.next();
        } else if c == '\n' {
            chars.next();
            // Consume optional \r after \n
            while chars.peek() == Some(&'\r') {
                chars.next();
            }
            // Check for --- on next line (horizontal rule)
            if matches_hrule(&chars) {
                // Consume the ---
                chars.next(); // -
                chars.next(); // -
                chars.next(); // -
                // Consume rest of line
                while chars.peek().is_some_and(|&c| c != '\n') {
                    chars.next();
                }
                flush_paragraph(&mut inline_buf, &mut blocks);
                blocks.push(ContentBlock::HRule);
            } else if chars.peek() == Some(&'\n') {
                // Double newline = paragraph break
                chars.next();
                // Consume additional blank lines
                while chars.peek() == Some(&'\r') || chars.peek() == Some(&'\n') {
                    chars.next();
                }
                flush_paragraph(&mut inline_buf, &mut blocks);
            } else if !inline_buf.is_empty() {
                push_inline_text(&mut inline_buf, " ");
            }
        } else {
            push_inline_char(&mut inline_buf, c);
            chars.next();
        }
    }

    flush_paragraph(&mut inline_buf, &mut blocks);
    blocks
}

fn flush_paragraph(buf: &mut Vec<InlineContent>, blocks: &mut Vec<ContentBlock>) {
    if buf.is_empty() {
        return;
    }
    let mut content = std::mem::take(buf);
    // Only trim leading/trailing whitespace-only text nodes
    while matches!(content.first(), Some(InlineContent::Text { text }) if text.trim().is_empty()) {
        content.remove(0);
    }
    while matches!(content.last(), Some(InlineContent::Text { text }) if text.trim().is_empty()) {
        content.pop();
    }
    if !content.is_empty() {
        if let Some(InlineContent::Bold { text }) = content.first() {
            let key = text.clone();
            let mut value: Vec<InlineContent> = content[1..].to_vec();
            // Trim leading whitespace from first value inline
            if let Some(InlineContent::Text { text }) = value.first_mut() {
                *text = text.trim_start().to_string();
                if text.is_empty() {
                    value.remove(0);
                }
            }
            blocks.push(ContentBlock::KeyValue { key, value });
        } else {
            blocks.push(ContentBlock::Paragraph { content });
        }
    }
}

fn push_inline_char(buf: &mut Vec<InlineContent>, c: char) {
    if let Some(InlineContent::Text { text }) = buf.last_mut() {
        text.push(c);
    } else {
        buf.push(InlineContent::Text {
            text: c.to_string(),
        });
    }
}

fn push_inline_text(buf: &mut Vec<InlineContent>, s: &str) {
    if s.is_empty() {
        return;
    }
    if let Some(InlineContent::Text { text }) = buf.last_mut() {
        text.push_str(s);
    } else {
        buf.push(InlineContent::Text {
            text: s.to_string(),
        });
    }
}

fn extract_level(tag: &str) -> u8 {
    extract_attr(tag, "level")
        .and_then(|l| l.parse().ok())
        .unwrap_or(1)
}

/// Parse inline content from an HTML string fragment.
fn parse_inline(input: &str, base_url: &str) -> Vec<InlineContent> {
    let mut result = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c == '<' {
            let tag = consume_tag(&mut chars);
            let lower = tag.to_lowercase();
            if lower.starts_with("<trait ") {
                if let Some(label) = extract_attr(&tag, "label") {
                    result.push(InlineContent::Trait { label });
                }
            } else if lower.starts_with("<actions") {
                if let Some(s) = extract_attr(&tag, "string") {
                    result.push(InlineContent::Action { text: s });
                }
            } else if lower == "<b>" || lower == "<strong>" {
                let close = if lower == "<b>" { "b" } else { "strong" };
                let content = collect_until_close(&mut chars, close);
                let clean = strip_inner_tags(&content);
                if !clean.is_empty() {
                    result.push(InlineContent::Bold { text: clean });
                }
            } else if lower == "<i>" || lower == "<em>" {
                let close = if lower == "<i>" { "i" } else { "em" };
                let content = collect_until_close(&mut chars, close);
                let clean = strip_inner_tags(&content);
                if !clean.is_empty() {
                    result.push(InlineContent::Italic { text: clean });
                }
            }
            // ignore other tags
        } else if c == '[' {
            let (text, url) = consume_link(&mut chars);
            let url = resolve_url(&url, base_url);
            result.push(InlineContent::Link { text, url });
        } else if c == '*' && chars.clone().nth(1) == Some('*') {
            let text = consume_bold(&mut chars);
            result.push(InlineContent::Bold { text });
        } else if c == '_' && chars.clone().nth(1).is_some_and(|n| n.is_alphabetic()) {
            let text = consume_italic(&mut chars);
            result.push(InlineContent::Italic { text });
        } else {
            push_inline_char(&mut result, c);
            chars.next();
        }
    }
    result
}

fn parse_list_items(content: &str, base_url: &str) -> Vec<Vec<InlineContent>> {
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

fn parse_table(content: &str) -> Option<ContentBlock> {
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

fn parse_table_cells(content: &str) -> Vec<String> {
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

fn extract_action_from_html(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let start = lower.find("<actions ")?;
    let tag_end = html[start..].find('>')? + start;
    let tag = &html[start..=tag_end];
    extract_attr(tag, "string")
}

fn matches_hrule(chars: &std::iter::Peekable<std::str::Chars<'_>>) -> bool {
    let upcoming: String = chars.clone().take(3).collect();
    upcoming == "---"
}

// --- Shared parsing helpers ---

fn consume_tag(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let mut tag = String::new();
    for c in chars.by_ref() {
        tag.push(c);
        if c == '>' {
            break;
        }
    }
    tag
}

fn collect_until_close(
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

fn extract_attr(tag: &str, name: &str) -> Option<String> {
    let pattern = format!("{name}=\"");
    let start = tag.find(&pattern)? + pattern.len();
    let rest = &tag[start..];
    let end = rest.find('"')?;
    Some(html_escape::decode_html_entities(&rest[..end]).to_string())
}

fn strip_inner_tags(input: &str) -> String {
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

fn consume_link(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> (String, String) {
    chars.next(); // consume '['
    let mut text = String::new();
    for c in chars.by_ref() {
        if c == ']' {
            break;
        }
        text.push(c);
    }
    let mut url = String::new();
    if chars.peek() == Some(&'(') {
        chars.next();
        let mut depth = 1;
        for c in chars.by_ref() {
            if c == '(' {
                depth += 1;
            } else if c == ')' {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            url.push(c);
        }
    }
    (text, url)
}

fn consume_bold(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    chars.next();
    chars.next();
    let mut text = String::new();
    while let Some(&c) = chars.peek() {
        if c == '*' {
            chars.next();
            if chars.peek() == Some(&'*') {
                chars.next();
                break;
            }
            text.push('*');
        } else {
            text.push(c);
            chars.next();
        }
    }
    text
}

fn resolve_url(url: &str, base_url: &str) -> String {
    if url.starts_with('/') {
        format!("{base_url}{url}")
    } else {
        url.to_string()
    }
}

fn consume_italic(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    chars.next();
    let mut text = String::new();
    for c in chars.by_ref() {
        if c == '_' {
            break;
        }
        text.push(c);
    }
    text
}
