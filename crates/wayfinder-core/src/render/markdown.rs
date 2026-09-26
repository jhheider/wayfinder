//! Render `ContentBlock` to clean markdown (no HTML tags).

use super::content::{ContentBlock, InlineContent};

/// Render content blocks to clean markdown text.
pub fn render_markdown(blocks: &[ContentBlock]) -> String {
    render(blocks, true)
}

/// Like [`render_markdown`], but links are reduced to their text. Suits LLM
/// consumers: AON links every trait and term, which costs tokens, and a model
/// can look any of them up by name.
pub fn render_markdown_unlinked(blocks: &[ContentBlock]) -> String {
    render(blocks, false)
}

fn render(blocks: &[ContentBlock], links: bool) -> String {
    let mut out = String::new();
    for (i, block) in blocks.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        render_block(&mut out, block, links);
    }
    out.trim().to_string()
}

fn render_block(out: &mut String, block: &ContentBlock, links: bool) {
    match block {
        ContentBlock::Title {
            level,
            text,
            right,
            action,
        } => {
            let hashes = "#".repeat(*level as usize);
            let mut line = format!("{hashes} {text}");
            if let Some(a) = action {
                line.push_str(&format!(" ({a})"));
            }
            if let Some(r) = right {
                line.push_str(&format!(" - {r}"));
            }
            out.push_str(&line);
            out.push('\n');
        }
        ContentBlock::Paragraph { content } => {
            render_inlines(out, content, links);
            out.push('\n');
        }
        ContentBlock::KeyValue { key, value } => {
            out.push_str(&format!("**{key}** "));
            render_inlines(out, value, links);
            out.push('\n');
        }
        ContentBlock::List { items } => {
            for item in items {
                out.push_str("- ");
                render_inlines(out, item, links);
                out.push('\n');
            }
        }
        ContentBlock::Table { headers, rows } => {
            if !headers.is_empty() {
                out.push_str(&format!("| {} |\n", headers.join(" | ")));
                let sep: Vec<&str> = headers.iter().map(|_| "---").collect();
                out.push_str(&format!("| {} |\n", sep.join(" | ")));
            }
            for row in rows {
                out.push_str(&format!("| {} |\n", row.join(" | ")));
            }
        }
        ContentBlock::Aside { title, content } => {
            if let Some(t) = title {
                out.push_str(&format!("> **{t}**\n"));
            }
            for inner in content {
                out.push_str("> ");
                render_block(out, inner, links);
            }
            out.push('\n');
        }
        ContentBlock::HRule => {
            out.push_str("---\n");
        }
    }
}

fn render_inlines(out: &mut String, inlines: &[InlineContent], links: bool) {
    for ic in inlines {
        match ic {
            InlineContent::Text { text } => out.push_str(text),
            InlineContent::Bold { text } => {
                out.push_str(&format!("**{text}**"));
            }
            InlineContent::Italic { text } => {
                out.push_str(&format!("_{text}_"));
            }
            InlineContent::Link { text, url } => {
                if url.is_empty() || !links {
                    out.push_str(text);
                } else {
                    out.push_str(&format!("[{text}]({url})"));
                }
            }
            InlineContent::Trait { label } => {
                out.push_str(&format!("[{label}]"));
            }
            InlineContent::Action { text } => {
                out.push_str(&format!("({text})"));
            }
        }
    }
}
