use serde_json::json;
use wayfinder_core::aon::{GameSystem, parse_documents, parse_total};

#[test]
fn parse_documents_extracts_each_source() {
    let resp = json!({"hits": {"hits": [
        {"_source": {"name": "Fireball", "category": "spell", "level": 3}},
        {"_source": {"name": "Shield", "category": "spell", "level": "1"}}
    ]}});
    let docs = parse_documents(&resp).unwrap();
    assert_eq!(docs.len(), 2);
    assert_eq!(docs[0].name.as_deref(), Some("Fireball"));
    assert_eq!(docs[0].level, Some(3));
    // level "1" as a string is tolerated by the model.
    assert_eq!(docs[1].level, Some(1));
}

#[test]
fn parse_documents_errors_when_hits_missing() {
    assert!(parse_documents(&json!({"nope": true})).is_err());
}

#[test]
fn parse_documents_errors_on_malformed_source() {
    // `name` is a String field; a number cannot deserialize into it.
    let resp = json!({"hits": {"hits": [{"_source": {"name": 123}}]}});
    assert!(parse_documents(&resp).is_err());
}

#[test]
fn parse_total_reads_hits_total_value() {
    assert_eq!(
        parse_total(&json!({"hits": {"total": {"value": 42}}})),
        Some(42)
    );
    assert_eq!(parse_total(&json!({"hits": {"hits": []}})), None);
    assert_eq!(parse_total(&json!({})), None);
}

#[test]
fn game_system_endpoints_and_labels() {
    let pf = GameSystem::Pathfinder;
    assert!(pf.endpoint().contains("/aon/"));
    assert_eq!(pf.index(), "aon70");
    assert_eq!(pf.label(), "PF2e");
    assert_eq!(pf.base_url(), "https://2e.aonprd.com");

    let sf = GameSystem::Starfinder;
    assert!(sf.endpoint().contains("/aonsf/"));
    assert_eq!(sf.index(), "aonsf10");
    assert_eq!(sf.label(), "SF2e");
    assert_eq!(sf.base_url(), "https://2e.aonsrd.com");
}
