//! Structured content IR for AON documents.
//! Parses AON HTML/markdown into `ContentBlock` trees.

mod html;
mod inline;

use serde::Serialize;

use html::{
    collect_until_close, consume_tag, extract_action_from_html, extract_attr, extract_level,
    matches_hrule, parse_list_items, parse_table, parse_table_cells, strip_inner_tags,
};
use inline::{
    consume_bold, consume_italic, consume_link, parse_inline, push_inline_char, push_inline_text,
    resolve_url,
};

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
