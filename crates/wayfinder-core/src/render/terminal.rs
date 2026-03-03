//! Render `ContentBlock` to styled terminal text spans.
//! No ANSI codes — consumers apply their own styling via the `Span` enum.

use super::content::{ContentBlock, InlineContent};

/// A styled span of text for terminal rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Span {
    Text(String),
    Bold(String),
    Italic(String),
    /// Heading — level 1..3.
    Title {
        level: u8,
        text: String,
    },
    Trait(String),
    Link {
        text: String,
        url: String,
    },
    Action(String),
    HRule,
    Newline,
    ListItem(Vec<Span>),
    TableHeader(Vec<String>),
    TableRow(Vec<String>),
    AsideTitle(String),
}

/// Render content blocks to a flat list of terminal spans.
pub fn render_spans(blocks: &[ContentBlock]) -> Vec<Span> {
    let mut spans = Vec::new();
    for block in blocks {
        render_block(&mut spans, block);
    }
    spans
}

fn render_block(spans: &mut Vec<Span>, block: &ContentBlock) {
    match block {
        ContentBlock::Title {
            level,
            text,
            right,
            action,
        } => {
            push_newline(spans);
            spans.push(Span::Title {
                level: *level,
                text: text.clone(),
            });
            if let Some(a) = action {
                spans.push(Span::Action(a.clone()));
            }
            if let Some(r) = right {
                spans.push(Span::Text(format!(" [{r}]")));
            }
            push_newline(spans);
        }
        ContentBlock::Paragraph { content } => {
            render_inlines(spans, content);
            push_newline(spans);
        }
        ContentBlock::KeyValue { key, value } => {
            spans.push(Span::Bold(key.clone()));
            push_text(spans, " ");
            render_inlines(spans, value);
            push_newline(spans);
        }
        ContentBlock::List { items } => {
            for item in items {
                let mut inner = Vec::new();
                render_inlines(&mut inner, item);
                spans.push(Span::ListItem(inner));
            }
        }
        ContentBlock::Table { headers, rows } => {
            if !headers.is_empty() {
                spans.push(Span::TableHeader(headers.clone()));
            }
            for row in rows {
                spans.push(Span::TableRow(row.clone()));
            }
        }
        ContentBlock::Aside { title, content } => {
            push_newline(spans);
            if let Some(t) = title {
                spans.push(Span::AsideTitle(t.clone()));
                push_newline(spans);
            }
            for inner in content {
                render_block(spans, inner);
            }
            push_newline(spans);
        }
        ContentBlock::HRule => {
            spans.push(Span::HRule);
        }
    }
}

fn render_inlines(spans: &mut Vec<Span>, inlines: &[InlineContent]) {
    for ic in inlines {
        match ic {
            InlineContent::Text { text } => push_text(spans, text),
            InlineContent::Bold { text } => {
                spans.push(Span::Bold(text.clone()));
            }
            InlineContent::Italic { text } => {
                spans.push(Span::Italic(text.clone()));
            }
            InlineContent::Link { text, url } => {
                spans.push(Span::Link {
                    text: text.clone(),
                    url: url.clone(),
                });
            }
            InlineContent::Trait { label } => {
                spans.push(Span::Trait(label.clone()));
            }
            InlineContent::Action { text } => {
                spans.push(Span::Action(text.clone()));
            }
        }
    }
}

/// Map an action string to a Unicode icon.
pub fn action_icon(action: &str) -> &str {
    match action.to_lowercase().as_str() {
        "single action" => "◆",
        "two actions" => "◆◆",
        "three actions" => "◆◆◆",
        "free action" => "◇",
        "reaction" => "↺",
        "variable" => "◆–◆◆◆",
        _ => action,
    }
}

fn push_text(spans: &mut Vec<Span>, text: &str) {
    if text.is_empty() {
        return;
    }
    if let Some(Span::Text(s)) = spans.last_mut() {
        s.push_str(text);
    } else {
        spans.push(Span::Text(text.to_string()));
    }
}

fn push_newline(spans: &mut Vec<Span>) {
    // Avoid triple+ newlines
    let tail: Vec<&Span> = spans.iter().rev().take(2).collect();
    if tail.len() >= 2 && matches!(tail[0], Span::Newline) && matches!(tail[1], Span::Newline) {
        return;
    }
    spans.push(Span::Newline);
}
