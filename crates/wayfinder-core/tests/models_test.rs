use wayfinder_core::aon::Document;

#[test]
fn parse_minimal_document() {
    let json = r#"{"name":"Fireball","category":"spell","level":3}"#;
    let doc: Document = serde_json::from_str(json).unwrap();
    assert_eq!(doc.name.as_deref(), Some("Fireball"));
    assert_eq!(doc.level, Some(3));
}

#[test]
fn display_short_with_level() {
    let json = r#"{"name":"Fireball","category":"spell","level":3}"#;
    let doc: Document = serde_json::from_str(json).unwrap();
    assert_eq!(doc.display_short(), "[spell] Fireball (Level 3)");
}

#[test]
fn display_short_without_level() {
    let json = r#"{"name":"Apsu","category":"deity"}"#;
    let doc: Document = serde_json::from_str(json).unwrap();
    assert_eq!(doc.display_short(), "[deity] Apsu");
}

#[test]
fn parse_array_fields() {
    let json = r#"{"name":"Apsu","category":"deity","domain":["Dragon","Fire"],"trait":["Good"]}"#;
    let doc: Document = serde_json::from_str(json).unwrap();
    assert_eq!(doc.domain, vec!["Dragon", "Fire"]);
    assert_eq!(doc.traits, vec!["Good"]);
}

#[test]
fn remaster_id_deserializes_to_typed_field() {
    let json = r#"{"name":"Fireball","category":"spell","remaster_id":["spell-new"]}"#;
    let doc: Document = serde_json::from_str(json).unwrap();
    assert_eq!(doc.remaster_id, vec!["spell-new"]);
    assert!(!doc.extra.contains_key("remaster_id"));
}
