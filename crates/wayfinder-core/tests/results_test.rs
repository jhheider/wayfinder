use std::collections::HashMap;

use wayfinder_core::aon::Document;
use wayfinder_core::search::{filter_legacy_duplicates, group_broad_results, is_legacy};

fn make_doc(id: &str, name: &str, category: &str) -> Document {
    Document {
        id: Some(id.into()),
        name: Some(name.into()),
        category: Some(category.into()),
        doc_type: None,
        url: None,
        text: None,
        markdown: None,
        summary: None,
        source: vec![],
        rarity: None,
        traits: vec![],
        trait_group: vec![],
        pfs: None,
        level: None,
        tradition: vec![],
        domain: vec![],
        favored_weapon: vec![],
        sanctification: vec![],
        attribute: vec![],
        deity: vec![],
        remaster_id: vec![],
        legacy_id: vec![],
        extra: HashMap::new(),
    }
}

fn make_legacy_pair() -> (Document, Document) {
    let mut legacy = make_doc("spell-old", "Fireball", "spell");
    legacy.remaster_id = vec!["spell-new".into()];

    let mut remaster = make_doc("spell-new", "Fireball", "spell");
    remaster.legacy_id = vec!["spell-old".into()];

    (legacy, remaster)
}

#[test]
fn is_legacy_true_when_remaster_id_present() {
    let (legacy, _) = make_legacy_pair();
    assert!(is_legacy(&legacy));
}

#[test]
fn is_legacy_false_for_remaster() {
    let (_, remaster) = make_legacy_pair();
    assert!(!is_legacy(&remaster));
}

#[test]
fn filter_legacy_default_drops_legacy() {
    let (legacy, remaster) = make_legacy_pair();
    let result = filter_legacy_duplicates(vec![legacy, remaster.clone()], false);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id.as_deref(), Some("spell-new"));
}

#[test]
fn filter_legacy_prefer_legacy_drops_remaster() {
    let (legacy, remaster) = make_legacy_pair();
    let result = filter_legacy_duplicates(vec![legacy.clone(), remaster], true);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id.as_deref(), Some("spell-old"));
}

#[test]
fn filter_legacy_passthrough_no_duplicates() {
    let docs = vec![
        make_doc("spell-1", "Fireball", "spell"),
        make_doc("spell-2", "Lightning Bolt", "spell"),
    ];
    let result = filter_legacy_duplicates(docs, false);
    assert_eq!(result.len(), 2);
}

#[test]
fn group_broad_exact_matches_first() {
    let docs = vec![
        make_doc("feat-1", "Fire Shield", "feat"),
        make_doc("spell-1", "Fire", "spell"),
        make_doc("deity-1", "Fire God", "deity"),
    ];
    let result = group_broad_results(docs, "fire");
    // All contain "fire", so all go to exact bucket in original order
    assert_eq!(result[0].id.as_deref(), Some("feat-1"));
    assert_eq!(result[1].id.as_deref(), Some("spell-1"));
    assert_eq!(result[2].id.as_deref(), Some("deity-1"));
}

#[test]
fn group_broad_separates_non_matches() {
    let docs = vec![
        make_doc("feat-1", "Unrelated", "feat"),
        make_doc("spell-1", "Fireball", "spell"),
        make_doc("feat-2", "Also Unrelated", "feat"),
    ];
    let result = group_broad_results(docs, "fire");
    // Fireball matches, the others don't
    assert_eq!(result[0].id.as_deref(), Some("spell-1"));
    assert_eq!(result[1].id.as_deref(), Some("feat-1"));
    assert_eq!(result[2].id.as_deref(), Some("feat-2"));
}

#[test]
fn group_broad_empty_results() {
    let result = group_broad_results(vec![], "fire");
    assert!(result.is_empty());
}
