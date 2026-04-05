//! Extract ancestry data from AON documents.

use wayfinder_core::aon::Document;

use crate::build::choices::AncestryData;

use super::boosts::extract_boost_spec;

/// Extract ancestry HP, size, speed, and boost spec from an ancestry document.
pub fn extract_ancestry_data(doc: &Document) -> AncestryData {
    let size = doc.extra.get("size").and_then(|v| {
        v.as_array()
            .and_then(|a| a.first())
            .and_then(|s| s.as_str())
            .map(String::from)
    });
    let speed = doc
        .extra
        .get("speed")
        .and_then(|v| v.get("land"))
        .and_then(|v| v.as_u64())
        .unwrap_or(25) as u32;
    let hp = doc.extra.get("hp").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let boost_spec = extract_boost_spec(doc);
    let granted_languages = doc
        .extra
        .get("language")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    AncestryData {
        size,
        speed,
        hp,
        boost_spec,
        granted_languages,
    }
}
