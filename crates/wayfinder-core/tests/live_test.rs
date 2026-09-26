//! Live checks against Archives of Nethys: the assumptions wayfinder makes
//! about AON's Elasticsearch indexes (index names, fields, category keys)
//! that no mock can catch going stale.
//!
//! Ignored by default. The weekly `aon-canary` workflow runs them and opens
//! an issue when they fail:
//!
//! ```sh
//! cargo test -p wayfinder-core --test live_test -- --ignored
//! ```
//!
//! They honor `WAYFINDER_AON_ENDPOINT`, and never use the response cache.

use wayfinder_core::Wayfinder;
use wayfinder_core::aon::categories::ALL_CATEGORIES;
use wayfinder_core::aon::{AonClient, Edition, GameSystem, Lookup, Search};

fn live(system: GameSystem) -> Wayfinder {
    Wayfinder::new(AonClient::from_env(system).expect("client"), None)
}

async fn assert_index_is_populated(system: GameSystem, at_least: u64) {
    let page = live(system).search(&Search::default()).await.unwrap();
    // Elasticsearch stops counting at 10,000, so larger indexes report that.
    assert!(
        page.total >= at_least.min(10_000),
        "{}: only {} entries (index renamed or emptied?)",
        system.label(),
        page.total
    );
}

#[tokio::test]
#[ignore = "live AON"]
async fn indexes_answer_with_documents() {
    assert_index_is_populated(GameSystem::Pathfinder, 30_000).await;
    assert_index_is_populated(GameSystem::Starfinder, 4_000).await;
}

#[tokio::test]
#[ignore = "live AON"]
async fn every_live_category_is_known() {
    // A new AON category still works, but `wf categories` files it under
    // "Other" and offline resolution rejects it until it is added here.
    for system in [GameSystem::Pathfinder, GameSystem::Starfinder] {
        let wf = live(system);
        let cats = wf.categories().await.unwrap();
        assert!(
            cats.len() > 20,
            "{}: {} categories",
            system.label(),
            cats.len()
        );
        let unknown: Vec<&str> = cats
            .iter()
            .map(|(name, _)| name.as_str())
            .filter(|name| !ALL_CATEGORIES.contains(name))
            .collect();
        assert!(
            unknown.is_empty(),
            "{}: add to ALL_CATEGORIES and CATEGORY_GROUPS: {unknown:?}",
            system.label()
        );
    }
}

#[tokio::test]
#[ignore = "live AON"]
async fn lookup_returns_structured_markdown() {
    let wf = live(GameSystem::Pathfinder);
    let pick = wf
        .lookup(&Lookup::named("Fireball", Some("spell")))
        .await
        .unwrap()
        .expect("Fireball");
    let doc = pick.best;
    assert_eq!(doc.level, Some(3));
    assert!(doc.url.is_some(), "no url field");
    let md = doc.markdown.as_deref().unwrap_or_default();
    assert!(
        md.contains("**"),
        "markdown missing or unstructured: {md:.200}"
    );
}

#[tokio::test]
#[ignore = "live AON"]
async fn remaster_links_still_exist() {
    // Edition filtering relies on `remaster_id` / `legacy_id` and lookups on
    // `legacy_name`; if AON renames them, legacy and remastered entries mix.
    let wf = live(GameSystem::Pathfinder);
    let remastered = wf
        .lookup(&Lookup::named("Magic Missile", Some("spell")))
        .await
        .unwrap()
        .expect("Magic Missile resolves to its remaster")
        .best;
    assert_eq!(remastered.name.as_deref(), Some("Force Barrage"));
    assert!(!remastered.legacy_id.is_empty(), "legacy_id gone");
    let legacy = Lookup {
        edition: Edition::Legacy,
        ..Lookup::named("Heal", Some("spell"))
    };
    let legacy = wf.lookup(&legacy).await.unwrap().expect("legacy Heal").best;
    assert!(legacy.is_legacy(), "remaster_id gone");
}

#[tokio::test]
#[ignore = "live AON"]
async fn filters_still_match() {
    let wf = live(GameSystem::Pathfinder);
    let count = |s: Search| {
        let wf = &wf;
        async move { wf.search(&s).await.unwrap().total }
    };
    let by_source = count(Search {
        category: Some("spell".into()),
        source: Some("Player Core".into()),
        ..Search::default()
    })
    .await;
    assert!(by_source > 100, "source phrase: {by_source}");
    let by_trait_and_level = count(Search {
        category: Some("spell".into()),
        traits: vec!["fire".into()],
        min_level: Some(3),
        max_level: Some(3),
        ..Search::default()
    })
    .await;
    assert!(by_trait_and_level > 0, "trait + level range");
    let by_field = count(Search {
        category: Some("deity".into()),
        fields: vec![("domain".into(), "dragon".into())],
        ..Search::default()
    })
    .await;
    assert!(by_field > 0, "field filter");
}
