use serde_json::json;
use wayfinder_core::aon::query::{DEFAULT_LIMIT, MAX_INPUT_LEN, MAX_LIMIT};
use wayfinder_core::aon::{Document, Edition, Lookup, Search, Sort, pick};

const BASE: &str = "https://2e.aonprd.com";

#[test]
fn empty_search_is_match_all_remastered_first_page() {
    let q = Search::default().body().unwrap();
    assert_eq!(q["size"], json!(DEFAULT_LIMIT));
    assert_eq!(q["from"], json!(0));
    assert_eq!(q["query"]["bool"]["must"], json!([{ "match_all": {} }]));
    assert_eq!(q["query"]["bool"]["filter"], json!([]));
    assert_eq!(
        q["query"]["bool"]["must_not"],
        json!([{ "exists": { "field": "remaster_id" } }])
    );
    assert_eq!(q["sort"], json!(["_score"]));
    assert!(q["_source"].is_array(), "summary projection by default");
}

#[test]
fn every_filter_composes() {
    let s = Search {
        text: Some("fireball".into()),
        name: Some("Fire".into()),
        category: Some("Spell".into()),
        traits: vec!["Fire".into(), " concentrate ".into(), " ".into()],
        fields: vec![("tradition".into(), "arcane".into())],
        min_level: Some(1),
        max_level: Some(5),
        source: Some("Player Core".into()),
        rarity: Some("Common".into()),
        sort: Sort::Level,
        limit: Some(3),
        offset: 6,
        ..Default::default()
    };
    let q = s.body().unwrap();
    assert_eq!(q["size"], json!(3));
    assert_eq!(q["from"], json!(6));
    let must = q["query"]["bool"]["must"].as_array().unwrap();
    assert_eq!(must[0]["multi_match"]["query"], json!("fireball"));
    assert!(must.contains(&json!({ "match_phrase": { "name": "Fire" } })));
    let filters = q["query"]["bool"]["filter"].as_array().unwrap();
    for f in [
        json!({ "term": { "category": "spell" } }),
        json!({ "term": { "trait": "fire" } }),
        json!({ "term": { "trait": "concentrate" } }),
        json!({ "term": { "tradition": "arcane" } }),
        json!({ "range": { "level": { "gte": 1, "lte": 5 } } }),
        // Regression: a plain `match` let "Player Core" match "Core Rulebook".
        json!({ "match_phrase": { "source": "Player Core" } }),
        json!({ "term": { "rarity": "common" } }),
    ] {
        assert!(filters.contains(&f), "missing {f}");
    }
    assert_eq!(filters.len(), 7, "blank trait must be skipped");
    assert_eq!(q["sort"], json!([{ "level": "asc" }, "_score"]));
}

#[test]
fn text_search_matches_legacy_names() {
    let s = Search {
        text: Some("magic missile".into()),
        ..Default::default()
    };
    let fields = &s.body().unwrap()["query"]["bool"]["must"][0]["multi_match"]["fields"];
    assert!(fields.as_array().unwrap().contains(&json!("legacy_name^5")));
}

#[test]
fn legacy_edition_hides_remastered_entries() {
    let s = Search {
        edition: Edition::Legacy,
        ..Default::default()
    };
    assert_eq!(
        s.body().unwrap()["query"]["bool"]["must_not"],
        json!([{ "exists": { "field": "legacy_id" } }])
    );
}

#[test]
fn name_sort_uses_keyword_field_and_full_drops_projection() {
    let s = Search {
        sort: Sort::Name,
        full: true,
        ..Default::default()
    };
    let q = s.body().unwrap();
    assert_eq!(q["sort"], json!([{ "name.keyword": "asc" }]));
    assert!(q.get("_source").is_none());
}

#[test]
fn limit_and_offset_are_clamped() {
    let s = Search {
        limit: Some(1000),
        offset: 20_000,
        ..Default::default()
    };
    assert_eq!(s.effective_limit(), MAX_LIMIT);
    assert_eq!(s.effective_offset(), 10_000 - MAX_LIMIT);
}

#[test]
fn validation_rejects_bad_input() {
    let inverted = Search {
        min_level: Some(5),
        max_level: Some(2),
        ..Default::default()
    };
    assert!(inverted.body().is_err());
    let long = Search {
        traits: vec!["x".repeat(MAX_INPUT_LEN + 1)],
        ..Default::default()
    };
    assert!(long.body().is_err());
}

#[test]
fn lookup_by_name_fetches_candidates_of_one_edition() {
    let q = Lookup::named("Magic Missile", Some("Spell"))
        .body(BASE)
        .unwrap();
    assert_eq!(q["size"], json!(wayfinder_core::aon::lookup::CANDIDATES));
    let b = &q["query"]["bool"];
    assert_eq!(b["must"][0]["multi_match"]["type"], json!("phrase"));
    assert_eq!(b["filter"], json!([{ "term": { "category": "spell" } }]));
    assert_eq!(
        b["must_not"],
        json!([{ "exists": { "field": "remaster_id" } }])
    );
}

#[test]
fn lookup_by_url_strips_either_scheme() {
    for url in [
        "https://2e.aonprd.com/Spells.aspx?ID=119",
        "http://2e.aonprd.com/Spells.aspx?ID=119",
        "/Spells.aspx?ID=119",
    ] {
        let l = Lookup {
            url: Some(url.into()),
            name: Some("ignored".into()),
            ..Default::default()
        };
        assert_eq!(
            l.body(BASE).unwrap()["query"],
            json!({ "term": { "url": "/Spells.aspx?ID=119" } }),
            "{url}"
        );
    }
}

#[test]
fn lookup_requires_name_or_url() {
    assert!(Lookup::default().body(BASE).is_err());
}

fn docs(v: serde_json::Value) -> Vec<Document> {
    serde_json::from_value(v).unwrap()
}

#[test]
fn pick_prefers_likely_category_among_exact_names() {
    // Regression: `get Shield` returned the thaumaturge implement.
    let c = docs(json!([
        {"name": "Shield", "category": "implement"},
        {"name": "Shield", "category": "spell"},
        {"name": "Shield", "category": "weapon-group"},
        {"name": "Shield Block", "category": "feat"}
    ]));
    let p = pick("shield", c).unwrap();
    assert_eq!(p.best.category.as_deref(), Some("spell"));
    let others: Vec<_> = p
        .same_name
        .iter()
        .filter_map(|d| d.category.as_deref())
        .collect();
    assert_eq!(others, ["implement", "weapon-group"]);
}

#[test]
fn pick_exact_and_legacy_names_beat_partial_matches() {
    let c = docs(json!([
        {"name": "Magic Missile Staff", "category": "equipment"},
        {"name": "Force Barrage", "category": "spell", "legacy_name": ["Magic Missile"]}
    ]));
    assert_eq!(
        pick("magic missile", c).unwrap().best.name.as_deref(),
        Some("Force Barrage")
    );
}

#[test]
fn pick_falls_back_to_search_order() {
    let c = docs(json!([
        {"name": "A B C", "category": "rules"},
        {"name": "A B", "category": "spell"}
    ]));
    let p = pick("B", c).unwrap();
    assert_eq!(p.best.name.as_deref(), Some("A B C"));
    assert!(p.same_name.is_empty());
    assert!(pick("x", Vec::new()).is_none());
}

#[test]
fn pick_prefers_current_name_over_legacy_name() {
    // Regression: `wf show rules Flanking` returned "3D Flanking", whose
    // legacy name is "Flanking", because it ranked higher in search.
    let c = docs(json!([
        {"name": "3D Flanking", "category": "rules", "legacy_name": ["Flanking"]},
        {"name": "Flanking", "category": "rules"}
    ]));
    let p = pick("Flanking", c).unwrap();
    assert_eq!(p.best.name.as_deref(), Some("Flanking"));
    assert_eq!(p.same_name[0].name.as_deref(), Some("3D Flanking"));
}
