use wayfinder_core::aon::SearchQuery;
use wayfinder_core::aon::query::{is_valid_filter_field, is_valid_filter_for_category};

#[test]
fn builds_category_query() {
    let q = SearchQuery::new().category("deity");
    let body = q.build();
    let musts = body["query"]["bool"]["must"].as_array().unwrap();
    assert_eq!(musts.len(), 1);
    assert_eq!(musts[0]["term"]["category"], "deity");
}

#[test]
fn builds_filtered_query() {
    let q = SearchQuery::new()
        .category("deity")
        .filter("domain", "Dragon")
        .filter("favored_weapon", "Whip");
    let body = q.build();
    let musts = body["query"]["bool"]["must"].as_array().unwrap();
    assert_eq!(musts.len(), 3);
}

#[test]
fn builds_text_query() {
    let q = SearchQuery::new().text("fireball");
    let body = q.build();
    let musts = body["query"]["bool"]["must"].as_array().unwrap();
    assert_eq!(musts[0]["match"]["text"], "fireball");
}

#[test]
fn respects_size_and_from() {
    let q = SearchQuery::new().size(10).from(20);
    let body = q.build();
    assert_eq!(body["size"], 10);
    assert_eq!(body["from"], 20);
}

#[test]
fn is_valid_filter_field_accepts_known() {
    assert!(is_valid_filter_field("rarity"));
    assert!(is_valid_filter_field("level"));
    assert!(is_valid_filter_field("tradition"));
    assert!(is_valid_filter_field("domain"));
}

#[test]
fn is_valid_filter_field_rejects_unknown() {
    assert!(!is_valid_filter_field("foobar"));
    assert!(!is_valid_filter_field("drop_table"));
    assert!(!is_valid_filter_field(""));
}

#[test]
fn category_aware_filter_validation() {
    // "domain" is valid for deity but not for feat
    assert!(is_valid_filter_for_category("domain", "deity"));
    assert!(!is_valid_filter_for_category("domain", "feat"));
    // "rarity" is valid for most categories
    assert!(is_valid_filter_for_category("rarity", "spell"));
    assert!(is_valid_filter_for_category("rarity", "feat"));
    // Unknown category falls back to global whitelist
    assert!(is_valid_filter_for_category("rarity", "unknown-cat"));
    assert!(!is_valid_filter_for_category("foobar", "unknown-cat"));
}
