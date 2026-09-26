//! Choosing which of a `get`'s candidate documents to return.
//!
//! AON's phrase match ranks every entry named exactly "Shield" (a spell, a
//! thaumaturge implement, a weapon group) the same, so Elasticsearch order
//! alone picks one arbitrarily. Prefer an exact name, then the category a
//! player most likely means, and report the rest so the model can ask again.

use wayfinder_core::aon::Document;

/// When several entries share a name and no category was given, earlier
/// categories here win; unlisted ones come after, in search order.
const CATEGORY_PRIORITY: &[&str] = &[
    "spell",
    "feat",
    "action",
    "condition",
    "equipment",
    "weapon",
    "armor",
    "shield",
    "creature",
    "ancestry",
    "heritage",
    "class",
    "archetype",
    "background",
    "deity",
    "ritual",
    "hazard",
    "skill",
    "trait",
    "rules",
];

/// The chosen entry, plus the other candidates that share its exact name.
pub struct Pick {
    pub best: Document,
    pub same_name: Vec<Document>,
}

fn names_match(doc: &Document, wanted: &str) -> bool {
    let current = doc.name.iter();
    let legacy = doc
        .extra
        .get("legacy_name")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str());
    current
        .map(String::as_str)
        .chain(legacy)
        .any(|n| n.trim().eq_ignore_ascii_case(wanted))
}

fn priority(doc: &Document) -> usize {
    let cat = doc.category.as_deref().unwrap_or("");
    CATEGORY_PRIORITY
        .iter()
        .position(|&c| c == cat)
        .unwrap_or(CATEGORY_PRIORITY.len())
}

/// Pick the best of `candidates` (in search-relevance order) for `name`.
/// Returns `None` when there are no candidates.
pub fn pick(name: &str, candidates: Vec<Document>) -> Option<Pick> {
    let wanted = name.trim();
    let (mut exact, rest): (Vec<_>, Vec<_>) =
        candidates.into_iter().partition(|d| names_match(d, wanted));
    if exact.is_empty() {
        // No exact name: trust search relevance.
        let mut rest = rest.into_iter();
        return rest.next().map(|best| Pick {
            best,
            same_name: Vec::new(),
        });
    }
    // Stable, so equal priorities keep search order.
    exact.sort_by_key(priority);
    let best = exact.remove(0);
    Some(Pick {
        best,
        same_name: exact,
    })
}

#[cfg(test)]
mod tests {
    use super::pick;
    use serde_json::json;
    use wayfinder_core::aon::Document;

    fn docs(v: serde_json::Value) -> Vec<Document> {
        serde_json::from_value(v).unwrap()
    }

    fn cat(d: &Document) -> &str {
        d.category.as_deref().unwrap()
    }

    #[test]
    fn prefers_likely_category_among_exact_names() {
        // Regression: `get Shield` returned the thaumaturge implement.
        let c = docs(json!([
            {"name": "Shield", "category": "implement"},
            {"name": "Shield", "category": "spell"},
            {"name": "Shield", "category": "weapon-group"},
            {"name": "Shield Block", "category": "feat"}
        ]));
        let p = pick("shield", c).unwrap();
        assert_eq!(cat(&p.best), "spell");
        let others: Vec<_> = p.same_name.iter().map(cat).collect();
        assert_eq!(others, ["implement", "weapon-group"]);
    }

    #[test]
    fn exact_name_beats_higher_ranked_partial_match() {
        let c = docs(json!([
            {"name": "Fireball Volley", "category": "feat"},
            {"name": "Fireball", "category": "spell"}
        ]));
        assert_eq!(
            pick("Fireball", c).unwrap().best.name.as_deref(),
            Some("Fireball")
        );
    }

    #[test]
    fn legacy_name_counts_as_exact() {
        let c = docs(json!([
            {"name": "Magic Missile Staff", "category": "equipment"},
            {"name": "Force Barrage", "category": "spell", "legacy_name": ["Magic Missile"]}
        ]));
        let p = pick("magic missile", c).unwrap();
        assert_eq!(p.best.name.as_deref(), Some("Force Barrage"));
    }

    #[test]
    fn falls_back_to_search_order_without_exact_match() {
        let c = docs(json!([
            {"name": "A B C", "category": "rules"},
            {"name": "A B", "category": "spell"}
        ]));
        let p = pick("B", c).unwrap();
        assert_eq!(p.best.name.as_deref(), Some("A B C"));
        assert!(p.same_name.is_empty());
    }

    #[test]
    fn empty_is_none() {
        assert!(pick("x", Vec::new()).is_none());
    }
}
