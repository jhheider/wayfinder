use serde_json::json;
use wayfinder_core::aon::{GameSystem, parse_documents, parse_total};

#[test]
fn parse_documents_extracts_each_source() {
    let resp = json!({"hits": {"hits": [
        {"_source": {"name": "Fireball", "category": "spell", "level": 3}},
        {"_source": {"name": "Shield", "category": "spell", "level": "1"}}
    ]}});
    let (docs, skipped) = parse_documents(&resp).unwrap();
    assert_eq!(docs.len(), 2);
    assert_eq!(skipped, 0);
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
fn parse_documents_skips_a_malformed_hit() {
    // `name` is a String field; a number cannot deserialize into it. One such
    // hit (as an uncovered AON category can produce) must not sink the page.
    let resp = json!({"hits": {"hits": [
        {"_source": {"name": "Fireball", "category": "spell"}},
        {"_source": {"name": 123}},
        {"_source": {"name": "Shield", "category": "spell"}}
    ]}});
    let (docs, skipped) = parse_documents(&resp).unwrap();
    assert_eq!(skipped, 1);
    assert_eq!(docs.len(), 2);
    assert_eq!(docs[0].name.as_deref(), Some("Fireball"));
    assert_eq!(docs[1].name.as_deref(), Some("Shield"));
}

#[test]
fn parse_documents_skips_a_bare_string_array_field() {
    // `source` is a `Vec<String>`; AON returns a bare string for some
    // categories, the same inconsistency `level` already needs tolerance for.
    let resp = json!({"hits": {"hits": [
        {"_source": {"name": "Shield", "source": "Core Rulebook"}},
        {"_source": {"name": "Fireball", "source": ["Core Rulebook"]}}
    ]}});
    let (docs, skipped) = parse_documents(&resp).unwrap();
    assert_eq!(skipped, 1);
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].name.as_deref(), Some("Fireball"));
}

#[test]
fn parse_documents_is_empty_when_no_hit_parses() {
    // All bad hits: an empty page, not an error, and every hit is counted.
    let resp = json!({"hits": {"hits": [{"_id": "1"}, {"_source": {"name": 123}}]}});
    let (docs, skipped) = parse_documents(&resp).unwrap();
    assert!(docs.is_empty());
    assert_eq!(skipped, 2);
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

#[test]
fn game_system_parses_aliases_case_insensitively() {
    for v in ["pf2e", "PF2E", " Pathfinder ", "pathfinder2e", "aonprd"] {
        assert_eq!(v.parse::<GameSystem>(), Ok(GameSystem::Pathfinder), "{v}");
    }
    for v in ["sf2e", "SF2E", "Starfinder", "aonsf", "aonsrd"] {
        assert_eq!(v.parse::<GameSystem>(), Ok(GameSystem::Starfinder), "{v}");
    }
    assert!("dnd".parse::<GameSystem>().is_err());
}
