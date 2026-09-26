//! Inline markdown/HTML: links, bold, italic, and the text buffer helpers.

use super::InlineContent;
use super::html::{collect_until_close, consume_tag, extract_attr, strip_inner_tags};

pub(super) fn push_inline_char(buf: &mut Vec<InlineContent>, c: char) {
    if let Some(InlineContent::Text { text }) = buf.last_mut() {
        text.push(c);
    } else {
        buf.push(InlineContent::Text {
            text: c.to_string(),
        });
    }
}

pub(super) fn push_inline_text(buf: &mut Vec<InlineContent>, s: &str) {
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

/// Parse inline content from an HTML string fragment.
pub(super) fn parse_inline(input: &str, base_url: &str) -> Vec<InlineContent> {
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

pub(super) fn consume_link(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
) -> (String, String) {
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

/// Consume `**bold**`, returning its text with any inline link markup reduced
/// to the link text (AON bolds whole links, e.g. `**[Recall Knowledge](/Rules...)**`).
pub(super) fn consume_bold(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
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
    strip_inner_tags(&text)
}

pub(super) fn resolve_url(url: &str, base_url: &str) -> String {
    if url.starts_with('/') {
        format!("{base_url}{url}")
    } else {
        url.to_string()
    }
}

pub(super) fn consume_italic(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
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
