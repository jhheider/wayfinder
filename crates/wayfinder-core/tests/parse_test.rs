use wayfinder_core::aon::parse::{
    CategoryError, normalize_category, parse_compound, resolve_category,
};

#[test]
fn parse_compound_with_category_and_name() {
    let (cat, name) = parse_compound("deity/sarenrae");
    assert_eq!(cat.as_deref(), Some("deity"));
    assert_eq!(name.as_deref(), Some("sarenrae"));
}

#[test]
fn parse_compound_name_only() {
    let (cat, name) = parse_compound("sarenrae");
    assert_eq!(cat, None);
    assert_eq!(name.as_deref(), Some("sarenrae"));
}

#[test]
fn parse_compound_slash_prefix() {
    let (cat, name) = parse_compound("/sarenrae");
    assert_eq!(cat, None);
    assert_eq!(name.as_deref(), Some("sarenrae"));
}

#[test]
fn parse_compound_trailing_slash() {
    let (cat, name) = parse_compound("deity/");
    assert_eq!(cat.as_deref(), Some("deity"));
    assert_eq!(name, None);
}

#[test]
fn normalize_category_plural_s() {
    assert_eq!(normalize_category("spells"), "spell");
}

#[test]
fn normalize_category_plural_ies() {
    assert_eq!(normalize_category("deities"), "deity");
}

#[test]
fn normalize_category_no_change() {
    assert_eq!(normalize_category("class"), "class");
}

#[test]
fn normalize_category_double_s() {
    assert_eq!(normalize_category("class"), "class");
}

#[test]
fn resolve_category_exact_match() {
    assert_eq!(resolve_category("spell"), Ok("spell".to_string()));
}

#[test]
fn resolve_category_normalizes_plural() {
    assert_eq!(resolve_category("spells"), Ok("spell".to_string()));
}

#[test]
fn resolve_category_suggests_close_match() {
    let result = resolve_category("spl");
    assert!(
        matches!(result, Err(CategoryError::Suggested { suggestion, .. }) if suggestion == "spell")
    );
}

#[test]
fn resolve_category_unknown() {
    let result = resolve_category("zzzznotacategory");
    assert!(matches!(result, Err(CategoryError::Unknown(_))));
}
