use std::collections::HashSet;

use indexmap::IndexMap;

use crate::aon::Document;

/// Check if a document is a legacy (pre-remaster) version.
pub fn is_legacy(doc: &Document) -> bool {
    !doc.remaster_id.is_empty()
}

/// Filter legacy/remaster duplicates from results.
/// Default: drop legacy when remaster counterpart is present.
/// With `prefer_legacy`: drop remaster when legacy counterpart is present.
pub fn filter_legacy_duplicates(docs: Vec<Document>, prefer_legacy: bool) -> Vec<Document> {
    let ids: HashSet<String> = docs.iter().filter_map(|d| d.id.clone()).collect();

    docs.into_iter()
        .filter(|doc| {
            if prefer_legacy {
                for legacy_id in &doc.legacy_id {
                    if ids.contains(legacy_id.as_str()) {
                        return false;
                    }
                }
            } else {
                for remaster_id in &doc.remaster_id {
                    if ids.contains(remaster_id.as_str()) {
                        return false;
                    }
                }
            }
            true
        })
        .collect()
}

/// Reorder broad search results: exact/partial name matches first,
/// then the rest grouped by category in first-appearance order.
pub fn group_broad_results(results: Vec<Document>, query: &str) -> Vec<Document> {
    let q = query.to_lowercase();
    let mut exact = Vec::new();
    let mut by_cat: IndexMap<String, Vec<Document>> = IndexMap::new();

    for doc in results {
        let name = doc.name.as_deref().unwrap_or("").to_lowercase();
        if name == q || name.contains(&q) {
            exact.push(doc);
        } else {
            let cat = doc.category.as_deref().unwrap_or("").to_string();
            by_cat.entry(cat).or_default().push(doc);
        }
    }

    for (_cat, docs) in by_cat {
        exact.extend(docs);
    }

    exact
}
