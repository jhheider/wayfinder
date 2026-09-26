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

fn doc(v: serde_json::Value) -> Document {
    serde_json::from_value(v).unwrap()
}

#[test]
fn edition_helpers() {
    let legacy = doc(serde_json::json!({"name": "Heal", "remaster_id": ["spell-1554"]}));
    assert!(legacy.is_legacy());
    assert_eq!(legacy.edition_note().as_deref(), Some("legacy"));

    let renamed =
        doc(serde_json::json!({"name": "Force Barrage", "legacy_name": ["Magic Missile"]}));
    assert!(!renamed.is_legacy());
    assert_eq!(
        renamed.edition_note().as_deref(),
        Some("formerly Magic Missile")
    );
    assert!(renamed.is_named(" magic missile "));
    assert!(renamed.is_named("FORCE BARRAGE"));
    assert!(!renamed.is_named("Force"));
}

#[test]
fn absolute_url_resolves_relative_and_keeps_absolute() {
    let base = "https://2e.aonprd.com";
    let rel = doc(serde_json::json!({"url": "/Spells.aspx?ID=1"}));
    assert_eq!(
        rel.absolute_url(base).as_deref(),
        Some("https://2e.aonprd.com/Spells.aspx?ID=1")
    );
    let abs = doc(serde_json::json!({"url": "https://2e.aonsrd.com/x"}));
    assert_eq!(
        abs.absolute_url(base).as_deref(),
        Some("https://2e.aonsrd.com/x")
    );
    assert_eq!(
        doc(serde_json::json!({"url": " "})).absolute_url(base),
        None
    );
}

#[test]
fn notable_rarity_skips_common_and_trait_duplicates() {
    let r = |v| doc(v).notable_rarity().map(str::to_string);
    assert_eq!(r(serde_json::json!({"rarity": "common"})), None);
    assert_eq!(
        r(serde_json::json!({"rarity": "rare", "trait": ["Rare"]})),
        None
    );
    assert_eq!(
        r(serde_json::json!({"rarity": "rare"})).as_deref(),
        Some("rare")
    );
}

#[test]
fn content_prefers_markdown_over_text() {
    let d = doc(serde_json::json!({"markdown": "**AC** 16", "text": "flat"}));
    let md = wayfinder_core::render::render_markdown(&d.content("b"));
    assert!(md.contains("**AC** 16"), "{md}");
    let t = doc(serde_json::json!({"markdown": " ", "text": "flat"}));
    assert_eq!(
        wayfinder_core::render::render_markdown(&t.content("b")),
        "flat"
    );
}

#[test]
fn extra_accessors() {
    let d = doc(serde_json::json!({"actions": " Reaction ", "source_raw": ["Core pg. 3"]}));
    assert_eq!(d.extra_str("actions"), Some("Reaction"));
    assert_eq!(d.extra_strings("source_raw"), vec!["Core pg. 3"]);
    assert!(d.extra_strings("missing").is_empty());
}
