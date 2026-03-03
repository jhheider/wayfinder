use wayfinder_core::cache::CacheStore;

#[test]
fn roundtrip_put_get() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");
    let store = CacheStore::open(&path).unwrap();

    store
        .put("1", "deity", "Apsu", r#"{"name":"Apsu"}"#)
        .unwrap();
    let data = store.get("1").unwrap();
    assert_eq!(data.unwrap(), r#"{"name":"Apsu"}"#);
}

#[test]
fn get_by_category() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");
    let store = CacheStore::open(&path).unwrap();

    store
        .put("1", "deity", "Apsu", r#"{"name":"Apsu"}"#)
        .unwrap();
    store
        .put("2", "deity", "Dahak", r#"{"name":"Dahak"}"#)
        .unwrap();
    store
        .put("3", "spell", "Fireball", r#"{"name":"Fireball"}"#)
        .unwrap();

    let deities = store.get_by_category("deity").unwrap();
    assert_eq!(deities.len(), 2);
}

#[test]
fn cache_ttl_expiration() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");
    let mut store = CacheStore::open(&path).unwrap();

    store
        .put("1", "deity", "Apsu", r#"{"name":"Apsu"}"#)
        .unwrap();
    store.set_ttl(0);
    let data = store.get("1").unwrap();
    assert!(data.is_none());
}

#[test]
fn cache_get_by_name_no_wildcard_leak() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");
    let store = CacheStore::open(&path).unwrap();

    store
        .put("1", "spell", "Fireball", r#"{"name":"Fireball"}"#)
        .unwrap();

    // Wildcard characters should not match anything
    let results = store.get_by_name("%", None).unwrap();
    assert!(results.is_empty(), "% should not match any documents");

    let results = store.get_by_name("_ireball", None).unwrap();
    assert!(results.is_empty(), "_ should not act as wildcard");

    // Exact case-insensitive match still works
    let results = store.get_by_name("fireball", None).unwrap();
    assert_eq!(results.len(), 1);

    // Literal % in a document name is retrievable by exact match
    store
        .put("2", "spell", "Fire%Ball", r#"{"name":"Fire%Ball"}"#)
        .unwrap();
    let results = store.get_by_name("Fire%Ball", None).unwrap();
    assert_eq!(results.len(), 1, "literal % in name should match exactly");
}

#[test]
fn cache_purge_expired() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");
    let mut store = CacheStore::open(&path).unwrap();

    store
        .put("1", "deity", "Apsu", r#"{"name":"Apsu"}"#)
        .unwrap();
    store.set_ttl(0); // everything is now expired
    let deleted = store.purge_expired().unwrap();
    assert_eq!(deleted, 1);

    // Verify the document is gone
    store.set_ttl(86400 * 7);
    let status = store.status().unwrap();
    assert!(status.is_empty());
}

#[test]
fn bulk_put_and_status() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");
    let mut store = CacheStore::open(&path).unwrap();

    let docs = vec![
        ("1".into(), "deity".into(), "Apsu".into(), "{}".into()),
        ("2".into(), "deity".into(), "Dahak".into(), "{}".into()),
    ];
    let count = store.bulk_put(&docs).unwrap();
    assert_eq!(count, 2);

    let status = store.status().unwrap();
    assert_eq!(status.len(), 1);
    assert_eq!(status[0], ("deity".to_string(), 2));
}
