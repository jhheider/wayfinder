use serde_json::json;
use std::time::Duration;
use wayfinder_core::cache::ResponseCache;

fn open() -> (tempfile::TempDir, ResponseCache) {
    let dir = tempfile::tempdir().unwrap();
    let cache = ResponseCache::open(&dir.path().join("nested/cache.db")).unwrap();
    (dir, cache)
}

#[test]
fn roundtrip_and_status_per_game() {
    let (_dir, cache) = open();
    assert_eq!(cache.get("k"), None);
    cache.put("k", "PF2e", &json!({"hits": 1})).unwrap();
    cache.put("k2", "SF2e", &json!({})).unwrap();
    cache.put("k3", "SF2e", &json!({})).unwrap();
    assert_eq!(cache.get("k"), Some(json!({"hits": 1})));
    assert_eq!(
        cache.status().unwrap(),
        vec![("PF2e".to_string(), 1), ("SF2e".to_string(), 2)]
    );
}

#[test]
fn put_replaces() {
    let (_dir, cache) = open();
    cache.put("k", "PF2e", &json!(1)).unwrap();
    cache.put("k", "PF2e", &json!(2)).unwrap();
    assert_eq!(cache.get("k"), Some(json!(2)));
}

#[test]
fn expired_entries_miss_and_purge() {
    let (_dir, cache) = open();
    let cache = cache.with_ttl(Duration::ZERO);
    cache.put("k", "PF2e", &json!(1)).unwrap();
    assert_eq!(cache.get("k"), None);
    assert!(cache.status().unwrap().is_empty());
    assert_eq!(cache.purge_expired().unwrap(), 1);
}

#[test]
fn clear_removes_everything() {
    let (_dir, cache) = open();
    cache.put("a", "PF2e", &json!(1)).unwrap();
    cache.put("b", "PF2e", &json!(1)).unwrap();
    assert_eq!(cache.clear().unwrap(), 2);
    assert_eq!(cache.get("a"), None);
}

#[test]
fn reopening_drops_the_old_documents_table() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("c.db");
    {
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute_batch("CREATE TABLE documents (id TEXT);")
            .unwrap();
    }
    ResponseCache::open(&path).unwrap();
    let conn = rusqlite::Connection::open(&path).unwrap();
    let n: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE name = 'documents'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(n, 0);
}

#[test]
fn two_handles_share_one_file() {
    // The CLI and the MCP server open the same cache concurrently.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("c.db");
    let a = ResponseCache::open(&path).unwrap();
    let b = ResponseCache::open(&path).unwrap();
    a.put("k", "PF2e", &json!("from a")).unwrap();
    assert_eq!(b.get("k"), Some(json!("from a")));
}

#[test]
fn writes_purge_expired_rows_without_an_explicit_purge() {
    // Regression: expired rows were only deleted by `wf cache purge`, so a
    // long-running MCP server's cache file grew forever.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("c.db");
    let old = ResponseCache::open(&path).unwrap();
    old.put("stale", "PF2e", &json!(1)).unwrap();
    // A later handle (a later process) with every row already expired.
    let later = ResponseCache::open(&path).unwrap().with_ttl(Duration::ZERO);
    later.put("fresh", "PF2e", &json!(2)).unwrap();
    let conn = rusqlite::Connection::open(&path).unwrap();
    let keys: Vec<String> = conn
        .prepare("SELECT key FROM responses")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(keys, ["fresh"]);
}
