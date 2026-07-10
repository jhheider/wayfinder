//! Rendering `wayfinder_core::aon::Document`s into the compact plain-text that
//! the MCP tools return to the model.

use wayfinder_core::aon::Document;

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
fn absolute_url(doc: &Document, base_url: &str) -> String {
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
    if let Some(rarity) = doc.rarity.as_deref().filter(|r| *r != "common") {
        header.push_str(&format!(" ({rarity})"));
    }

    let mut block = header;
    if let Some(summary) = doc.summary.as_deref().filter(|s| !s.is_empty()) {
        block.push_str(&format!("\n   {summary}"));
    }
    let url = absolute_url(doc, base_url);
    if !url.is_empty() {
        block.push_str(&format!("\n   {url}"));
    }
    block.push('\n');
    block
}

/// Render the full detail view for a `get` result.
pub fn format_detail(doc: &Document, base_url: &str) -> String {
    let name = doc.name.as_deref().unwrap_or("Unknown");
    let mut out = format!("# {name}");
    let label = type_label(doc);
    if !label.is_empty() {
        out.push_str(&format!("  ({label}"));
        if let Some(level) = doc.level {
            out.push_str(&format!(" {level}"));
        }
        out.push(')');
    }
    out.push('\n');

    let legacy_name = extra_vec(doc, "legacy_name");
    if !legacy_name.is_empty() {
        out.push_str(&format!("Formerly: {}\n", legacy_name.join(", ")));
    }
    if let Some(actions) = extra_str(doc, "actions") {
        out.push_str(&format!("Actions: {actions}\n"));
    }
    if !doc.traits.is_empty() {
        out.push_str(&format!("Traits: {}\n", doc.traits.join(", ")));
    }
    let mut meta = Vec::new();
    if let Some(rarity) = &doc.rarity {
        meta.push(format!("Rarity: {rarity}"));
    }
    if let Some(pfs) = &doc.pfs {
        meta.push(format!("PFS: {pfs}"));
    }
    if !meta.is_empty() {
        out.push_str(&format!("{}\n", meta.join(" | ")));
    }
    let source_raw = extra_vec(doc, "source_raw");
    let source = if !source_raw.is_empty() {
        source_raw.join("; ")
    } else {
        doc.source.join("; ")
    };
    if !source.is_empty() {
        out.push_str(&format!("Source: {source}\n"));
    }
    let url = absolute_url(doc, base_url);
    if !url.is_empty() {
        out.push_str(&format!("URL: {url}\n"));
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

#[cfg(test)]
mod tests {
    use super::{format_detail, format_summary};
    use serde_json::json;
    use wayfinder_core::aon::Document;

    fn doc(v: serde_json::Value) -> Document {
        serde_json::from_value(v).unwrap()
    }

    #[test]
    fn summary_includes_type_level_traits_rarity_summary_url() {
        let d = doc(json!({
            "name": "Fireball", "type": "Spell", "category": "spell", "level": 3,
            "trait": ["Fire", "Evocation"], "rarity": "uncommon",
            "summary": "Boom.", "url": "/Spells.aspx?ID=1"
        }));
        let s = format_summary(1, &d, "https://2e.aonprd.com");
        assert!(s.contains("1. Fireball - Spell 3"), "{s}");
        assert!(s.contains("[Fire, Evocation]"));
        assert!(s.contains("(uncommon)"));
        assert!(s.contains("Boom."));
        assert!(s.contains("https://2e.aonprd.com/Spells.aspx?ID=1"));
    }

    #[test]
    fn summary_hides_common_rarity_and_keeps_absolute_url() {
        let d = doc(json!({
            "name": "X", "category": "feat", "rarity": "common",
            "url": "https://example.test/y"
        }));
        let s = format_summary(2, &d, "https://2e.aonprd.com");
        assert!(!s.contains("(common)"));
        assert!(s.contains("https://example.test/y"));
    }

    #[test]
    fn summary_uses_category_label_and_level_when_no_type() {
        let d = doc(json!({"name": "Y", "category": "action", "level": 5}));
        let s = format_summary(1, &d, "b");
        assert!(s.contains("1. Y - action 5"), "{s}");
    }

    #[test]
    fn summary_unknown_name_and_bare_level() {
        let d = doc(json!({"category": "", "level": 2}));
        let s = format_summary(3, &d, "b");
        assert!(s.contains("3. Unknown (level 2)"), "{s}");
    }

    #[test]
    fn detail_renders_all_sections_and_prefers_source_raw() {
        let d = doc(json!({
            "name": "Force Barrage", "type": "Spell", "category": "spell", "level": 1,
            "trait": ["Force"], "rarity": "common", "pfs": "Standard",
            "source": ["Player Core"], "source_raw": ["Player Core pg. 1"],
            "legacy_name": ["Magic Missile"], "actions": "Single Action",
            "text": "Darts of force.", "url": "/Spells.aspx?ID=2"
        }));
        let s = format_detail(&d, "https://2e.aonprd.com");
        assert!(s.contains("# Force Barrage  (Spell 1)"), "{s}");
        assert!(s.contains("Formerly: Magic Missile"));
        assert!(s.contains("Actions: Single Action"));
        assert!(s.contains("Traits: Force"));
        assert!(s.contains("Rarity: common | PFS: Standard"));
        assert!(s.contains("Source: Player Core pg. 1"));
        assert!(s.contains("URL: https://2e.aonprd.com/Spells.aspx?ID=2"));
        assert!(s.contains("Darts of force."));
    }

    #[test]
    fn detail_falls_back_to_summary_and_plain_source_and_omits_missing() {
        let d = doc(json!({
            "name": "Z", "category": "feat", "source": ["Core"], "summary": "Short."
        }));
        let s = format_detail(&d, "b");
        assert!(s.starts_with("# Z  (feat)"), "{s}");
        assert!(s.contains("Source: Core"));
        assert!(s.contains("Short."));
        assert!(!s.contains("URL:"));
        assert!(!s.contains("Traits:"));
    }
}
