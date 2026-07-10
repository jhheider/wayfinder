use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tempfile::NamedTempFile;
use wayfinder_core::aon::client::SearchClient;
use wayfinder_core::aon::{Document, SearchQuery};
use wayfinder_core::cache::CacheStore;
use wayfinder_core::search::SearchService;

fn make_doc(id: &str, name: &str, category: &str) -> Document {
    Document {
        id: Some(id.into()),
        name: Some(name.into()),
        category: Some(category.into()),
        doc_type: None,
        url: None,
        text: Some("sample text".into()),
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

#[test]
fn cache_get_by_name_hit() {
    let tmp = NamedTempFile::new().unwrap();
    let store = CacheStore::open(tmp.path()).unwrap();

    let doc = make_doc("spell-119", "Fireball", "spell");
    let json = serde_json::to_string(&doc).unwrap();
    store.put("spell-119", "spell", "Fireball", &json).unwrap();

    let results = store.get_by_name("Fireball", Some("spell")).unwrap();
    assert_eq!(results.len(), 1);
    let parsed: Document = serde_json::from_str(&results[0]).unwrap();
    assert_eq!(parsed.name.as_deref(), Some("Fireball"));
}

#[test]
fn cache_get_by_name_no_category() {
    let tmp = NamedTempFile::new().unwrap();
    let store = CacheStore::open(tmp.path()).unwrap();

    let doc = make_doc("spell-119", "Fireball", "spell");
    let json = serde_json::to_string(&doc).unwrap();
    store.put("spell-119", "spell", "Fireball", &json).unwrap();

    let results = store.get_by_name("Fireball", None).unwrap();
    assert_eq!(results.len(), 1);
}

#[test]
fn cache_get_by_name_miss() {
    let tmp = NamedTempFile::new().unwrap();
    let store = CacheStore::open(tmp.path()).unwrap();

    let results = store.get_by_name("Nonexistent", None).unwrap();
    assert!(results.is_empty());
}

#[test]
fn cache_status_with_data() {
    let tmp = NamedTempFile::new().unwrap();
    let store = CacheStore::open(tmp.path()).unwrap();

    let doc1 = make_doc("spell-1", "Fireball", "spell");
    let doc2 = make_doc("spell-2", "Magic Missile", "spell");
    let doc3 = make_doc("deity-1", "Sarenrae", "deity");

    for doc in [&doc1, &doc2, &doc3] {
        let json = serde_json::to_string(doc).unwrap();
        store
            .put(
                doc.id.as_deref().unwrap(),
                doc.category.as_deref().unwrap(),
                doc.name.as_deref().unwrap(),
                &json,
            )
            .unwrap();
    }

    let status = store.status().unwrap();
    assert_eq!(status.len(), 2);
    // Sorted alphabetically
    assert_eq!(status[0], ("deity".into(), 1));
    assert_eq!(status[1], ("spell".into(), 2));
}

struct MockClient {
    docs: Vec<Document>,
    call_count: Arc<Mutex<u32>>,
}

impl MockClient {
    fn new(docs: Vec<Document>) -> Self {
        Self {
            docs,
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    fn calls(&self) -> u32 {
        *self.call_count.lock().unwrap()
    }
}

impl SearchClient for MockClient {
    async fn search(&self, _query: &SearchQuery) -> wayfinder_core::Result<Vec<Document>> {
        *self.call_count.lock().unwrap() += 1;
        Ok(self.docs.clone())
    }
}

#[tokio::test]
async fn service_show_cache_hit_skips_client() {
    let tmp = NamedTempFile::new().unwrap();
    let store = CacheStore::open(tmp.path()).unwrap();

    let doc = make_doc("spell-119", "Fireball", "spell");
    let json = serde_json::to_string(&doc).unwrap();
    store.put("spell-119", "spell", "Fireball", &json).unwrap();

    let mock = MockClient::new(vec![]);
    let svc = SearchService::new(mock, tmp.path());
    let results = svc.show("Fireball", Some("spell")).await.unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name.as_deref(), Some("Fireball"));
    // Mock client was never called since cache had the result
    assert_eq!(svc.client().calls(), 0);
}

#[tokio::test]
async fn service_show_cache_miss_calls_client() {
    let tmp = NamedTempFile::new().unwrap();

    let doc = make_doc("spell-119", "Fireball", "spell");
    let mock = MockClient::new(vec![doc]);
    let svc = SearchService::new(mock, tmp.path());
    let results = svc.show("Fireball", Some("spell")).await.unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name.as_deref(), Some("Fireball"));
    assert_eq!(svc.client().calls(), 1);

    // Verify it was cached
    let store = CacheStore::open(tmp.path()).unwrap();
    let cached = store.get_by_name("Fireball", Some("spell")).unwrap();
    assert_eq!(cached.len(), 1);
}

#[tokio::test]
async fn service_search_caches_opportunistically() {
    let tmp = NamedTempFile::new().unwrap();

    let doc = make_doc("spell-119", "Fireball", "spell");
    let mock = MockClient::new(vec![doc]);
    let svc = SearchService::new(mock, tmp.path());
    let q = SearchQuery::new().name("Fireball");
    let results = svc.search(&q).await.unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(svc.client().calls(), 1);

    // Verify cached
    let store = CacheStore::open(tmp.path()).unwrap();
    let cached = store.get_by_name("Fireball", Some("spell")).unwrap();
    assert_eq!(cached.len(), 1);
}
