//! Rendering `wayfinder_core::aon::Document`s into the compact plain text the
//! MCP tools return to the model. The facts (edition, rarity, URLs, rules
//! content) come from `Document`'s methods, shared with the `wf` CLI; only
//! the layout lives here.

use wayfinder_core::aon::Document;
use wayfinder_core::render::render_markdown_unlinked;

fn type_label(doc: &Document) -> &str {
    match doc.doc_type.as_deref().filter(|t| !t.is_empty()) {
        Some(t) => t,
        None => doc.category.as_deref().unwrap_or(""),
    }
}

/// Render a one-entry summary block for `search` results.
pub fn format_summary(index: usize, doc: &Document, base_url: &str) -> String {
    let name = doc.name.as_deref().unwrap_or("Unknown");
    let label = type_label(doc);
    let mut out = format!("{index}. {name}");
    match (label.is_empty(), doc.level) {
        (false, Some(level)) => out.push_str(&format!(" - {label} {level}")),
        (false, None) => out.push_str(&format!(" - {label}")),
        (true, Some(level)) => out.push_str(&format!(" (level {level})")),
        (true, None) => {}
    }
    if !doc.traits.is_empty() {
        out.push_str(&format!(" [{}]", doc.traits.join(", ")));
    }
    if let Some(rarity) = doc.notable_rarity() {
        out.push_str(&format!(" ({rarity})"));
    }
    if let Some(note) = doc.edition_note() {
        out.push_str(&format!(" ({note})"));
    }
    if let Some(summary) = doc
        .summary
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        out.push_str(&format!("\n   {summary}"));
    }
    if let Some(url) = doc.absolute_url(base_url) {
        out.push_str(&format!("\n   {url}"));
    }
    out.push('\n');
    out
}

/// Render the full detail view for a `get` result: a few lines of metadata
/// the rules text does not carry, then the entry as markdown (AON's structure
/// kept: stat block lines, action costs, heightening, tables).
pub fn format_detail(doc: &Document, base_url: &str) -> String {
    let mut out = String::new();
    if let Some(url) = doc.absolute_url(base_url) {
        out.push_str(&format!("URL: {url}\n"));
    }
    if doc.is_legacy() {
        out.push_str("Edition: legacy (pre-remaster); the Remaster replaced this entry\n");
    } else if !doc.legacy_name.is_empty() {
        out.push_str(&format!("Formerly: {}\n", doc.legacy_name.join(", ")));
    }
    if let Some(pfs) = &doc.pfs {
        out.push_str(&format!("PFS: {pfs}\n"));
    }
    // AON's markdown opens with its own title, traits and source; the flat
    // `text` fallback does not, so rebuild those from the fields.
    if doc.markdown.as_deref().is_none_or(|m| m.trim().is_empty()) {
        out.push_str(&plain_header(doc));
    }
    let mut body = render_markdown_unlinked(&doc.content(base_url));
    if body.is_empty() {
        body = doc.summary.as_deref().unwrap_or("").trim().to_string();
    }
    if !body.is_empty() {
        out.push('\n');
        out.push_str(&body);
        out.push('\n');
    }
    out
}

/// Title, actions, traits and source lines for entries without markdown.
fn plain_header(doc: &Document) -> String {
    let name = doc.name.as_deref().unwrap_or("Unknown");
    let label = type_label(doc);
    let mut out = format!("\n# {name}");
    match (label.is_empty(), doc.level) {
        (false, Some(level)) => out.push_str(&format!(" ({label} {level})")),
        (false, None) => out.push_str(&format!(" ({label})")),
        _ => {}
    }
    out.push('\n');
    if let Some(actions) = doc.extra_str("actions") {
        out.push_str(&format!("Actions: {actions}\n"));
    }
    if !doc.traits.is_empty() {
        out.push_str(&format!("Traits: {}\n", doc.traits.join(", ")));
    }
    let source_raw = doc.extra_strings("source_raw");
    let source = if source_raw.is_empty() {
        doc.source.join("; ")
    } else {
        source_raw.join("; ")
    };
    if !source.is_empty() {
        out.push_str(&format!("Source: {source}\n"));
    }
    out
}

/// One line per entry, for listing the other candidates a `get` passed over.
pub fn format_alternatives(docs: &[Document], base_url: &str) -> String {
    let mut out = String::new();
    for doc in docs {
        let name = doc.name.as_deref().unwrap_or("Unknown");
        let cat = doc.category.as_deref().unwrap_or("?");
        out.push_str(&format!("- {name} (category: {cat})"));
        if let Some(url) = doc.absolute_url(base_url) {
            out.push_str(&format!(" {url}"));
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
#[path = "format_tests.rs"]
mod tests;
