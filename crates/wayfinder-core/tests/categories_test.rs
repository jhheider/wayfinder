use wayfinder_core::aon::categories::{
    ALL_CATEGORIES, CATEGORY_GROUPS, filterable_fields, suggest_category,
};

#[test]
fn all_categories_sorted() {
    let mut sorted = ALL_CATEGORIES.to_vec();
    sorted.sort();
    assert_eq!(ALL_CATEGORIES, &sorted[..]);
}

#[test]
fn suggest_exact_match() {
    assert_eq!(suggest_category("deity"), Some("deity"));
    assert_eq!(suggest_category("spell"), Some("spell"));
}

#[test]
fn suggest_prefix_match() {
    assert_eq!(suggest_category("dei"), Some("deity"));
    assert_eq!(suggest_category("spe"), Some("spell"));
}

#[test]
fn suggest_edit_distance() {
    assert_eq!(suggest_category("diety"), Some("deity"));
    assert_eq!(suggest_category("spel"), Some("spell"));
    assert_eq!(suggest_category("feat"), Some("feat"));
}

#[test]
fn suggest_returns_none_for_garbage() {
    assert_eq!(suggest_category("xyzzyplugh"), None);
}

#[test]
fn filterable_fields_known_category() {
    let fields = filterable_fields("spell").unwrap();
    assert!(fields.contains(&"tradition"));
    assert!(fields.contains(&"level"));
}

#[test]
fn filterable_fields_unknown_category() {
    assert!(filterable_fields("nonexistent").is_none());
}

#[test]
fn all_group_members_are_valid_categories() {
    for group in CATEGORY_GROUPS {
        for &member in group.members {
            assert!(
                ALL_CATEGORIES.contains(&member),
                "Group '{}' has unknown category '{}'",
                group.name,
                member
            );
        }
    }
}
