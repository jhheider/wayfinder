//! Rendering `wayfinder_core::aon::Document`s into the compact plain-text that
//! the MCP tools return to the model.

use wayfinder_core::aon::Document;
use wayfinder_core::render::{parse_content, render_markdown_unlinked};
use wayfinder_core::search::is_legacy;

/// A document field that only some categories carry, read from the flattened
/// `extra` map as a string (e.g. `actions`).
fn extra_str<'a>(doc: &'a Document, key: &str) -> Option<&'a str> {
    doc.extra
        .get(key)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

/// An `extra` field that is a JSON array of strings (e.g. `legacy_name`,
/// `source_raw`).
fn extra_vec(doc: &Document, key: &str) -> Vec<String> {
    doc.extra
        .get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// Resolve a document's (relative) URL against a site base into an absolute URL.
pub fn absolute_url(doc: &Document, base_url: &str) -> String {
    match doc.url.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        None => String::new(),
        Some(url) if url.starts_with("http://") || url.starts_with("https://") => url.to_string(),
        Some(url) => format!("{base_url}{url}"),
    }
}

fn type_label(doc: &Document) -> &str {
    let type_ = doc.doc_type.as_deref().unwrap_or("");
    if type_.is_empty() {
        doc.category.as_deref().unwrap_or("")
    } else {
        type_
    }
}

/// The rarity, unless it is "common" or already listed among the traits (AON
/// repeats non-common rarities as a trait).
fn notable_rarity(doc: &Document) -> Option<&str> {
    doc.rarity
        .as_deref()
        .filter(|r| !r.eq_ignore_ascii_case("common"))
        .filter(|r| !doc.traits.iter().any(|t| t.eq_ignore_ascii_case(r)))
}

/// `legacy` for pre-remaster entries, `formerly X` for renamed remastered ones.
fn edition_note(doc: &Document) -> Option<String> {
    let formerly = extra_vec(doc, "legacy_name");
    if is_legacy(doc) {
        Some("legacy".to_string())
    } else if !formerly.is_empty() {
        Some(format!("formerly {}", formerly.join(", ")))
    } else {
        None
    }
}

/// Render a one-entry summary block for `search` results.
pub fn format_summary(index: usize, doc: &Document, base_url: &str) -> String {
    let name = doc.name.as_deref().unwrap_or("Unknown");
    let label = type_label(doc);
    let mut header = format!("{index}. {name}");
    if !label.is_empty() {
        header.push_str(&format!(" - {label}"));
        if let Some(level) = doc.level {
            header.push_str(&format!(" {level}"));
        }
    } else if let Some(level) = doc.level {
        header.push_str(&format!(" (level {level})"));
    }
    if !doc.traits.is_empty() {
        header.push_str(&format!(" [{}]", doc.traits.join(", ")));
    }
    if let Some(rarity) = notable_rarity(doc) {
        header.push_str(&format!(" ({rarity})"));
    }
    if let Some(note) = edition_note(doc) {
        header.push_str(&format!(" ({note})"));
    }

    let mut block = header;
    if let Some(summary) = doc
        .summary
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        block.push_str(&format!("\n   {summary}"));
    }
    let url = absolute_url(doc, base_url);
    if !url.is_empty() {
        block.push_str(&format!("\n   {url}"));
    }
    block.push('\n');
    block
}

/// Render the full detail view for a `get` result: a few lines of metadata
/// the rules text does not carry, then the entry itself as markdown (AON's
/// structure kept: stat block lines, action costs, heightening, tables).
pub fn format_detail(doc: &Document, base_url: &str) -> String {
    let mut out = String::new();
    let url = absolute_url(doc, base_url);
    if !url.is_empty() {
        out.push_str(&format!("URL: {url}\n"));
    }
    let formerly = extra_vec(doc, "legacy_name");
    if is_legacy(doc) {
        out.push_str("Edition: legacy (pre-remaster); the Remaster replaced this entry\n");
    } else if !formerly.is_empty() {
        out.push_str(&format!("Formerly: {}\n", formerly.join(", ")));
    }
    if let Some(pfs) = &doc.pfs {
        out.push_str(&format!("PFS: {pfs}\n"));
    }

    match doc.markdown.as_deref().filter(|s| !s.trim().is_empty()) {
        Some(md) => {
            out.push('\n');
            out.push_str(&render_markdown_unlinked(&parse_content(md, base_url)));
            out.push('\n');
        }
        None => out.push_str(&plain_detail(doc)),
    }
    out
}

/// Fallback for entries with no `markdown`: a header built from the fields,
/// then the flat `text` (or the summary).
fn plain_detail(doc: &Document) -> String {
    let name = doc.name.as_deref().unwrap_or("Unknown");
    let mut out = format!("\n# {name}");
    let label = type_label(doc);
    if !label.is_empty() {
        out.push_str(&format!(" ({label}"));
        if let Some(level) = doc.level {
            out.push_str(&format!(" {level}"));
        }
        out.push(')');
    }
    out.push('\n');
    if let Some(actions) = extra_str(doc, "actions") {
        out.push_str(&format!("Actions: {actions}\n"));
    }
    if !doc.traits.is_empty() {
        out.push_str(&format!("Traits: {}\n", doc.traits.join(", ")));
    }
    let source_raw = extra_vec(doc, "source_raw");
    let source = if source_raw.is_empty() {
        doc.source.join("; ")
    } else {
        source_raw.join("; ")
    };
    if !source.is_empty() {
        out.push_str(&format!("Source: {source}\n"));
    }
    let body = doc
        .text
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .or(doc.summary.as_deref())
        .unwrap_or("");
    if !body.is_empty() {
        out.push('\n');
        out.push_str(body.trim());
        out.push('\n');
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
        let url = absolute_url(doc, base_url);
        if !url.is_empty() {
            out.push_str(&format!(" {url}"));
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
#[path = "format_tests.rs"]
mod tests;
