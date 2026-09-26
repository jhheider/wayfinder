//! `Wayfinder` against an in-process mock of AON's `_search` endpoint that
//! counts requests, so caching and lookup policy are observable.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use wayfinder_core::Wayfinder;
use wayfinder_core::aon::{AonClient, CategoryError, GameSystem, Lookup, Search};
use wayfinder_core::cache::ResponseCache;

const HITS: &str = r#"{"hits":{"total":{"value":25},"hits":[
    {"_source":{"name":"Shield","category":"implement","url":"/Implements.aspx?ID=11"}},
    {"_source":{"name":"Shield","category":"spell","url":"/Spells.aspx?ID=1671"}}]}}"#;
const CATS: &str = r#"{"aggregations":{"cats":{"buckets":[
    {"key":"spell","doc_count":405},{"key":"rules","doc_count":9}]}}}"#;

/// Serve forever: category aggregations get CATS, everything else HITS, and
/// a path containing "fail" gets a 500. Returns the endpoint and a counter.
async fn mock(path: &str) -> (String, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let hits = Arc::new(AtomicUsize::new(0));
    let counter = hits.clone();
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = listener.accept().await {
            counter.fetch_add(1, Ordering::SeqCst);
            let mut buf = vec![0u8; 16384];
            let n = sock.read(&mut buf).await.unwrap_or(0);
            let req = String::from_utf8_lossy(&buf[..n]).to_string();
            let (status, body) = if req.starts_with("POST /fail") {
                ("500 Internal Server Error", "{}")
            } else if req.contains("\"aggs\"") {
                ("200 OK", CATS)
            } else {
                ("200 OK", HITS)
            };
            let resp = format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = sock.write_all(resp.as_bytes()).await;
            let _ = sock.shutdown().await;
        }
    });
    (format!("http://{addr}/{path}"), hits)
}

fn service(endpoint: String, cache: Option<Arc<ResponseCache>>) -> Wayfinder {
    Wayfinder::new(
        AonClient::with_endpoint(GameSystem::Pathfinder, endpoint).unwrap(),
        cache,
    )
}

fn temp_cache() -> (tempfile::TempDir, Arc<ResponseCache>) {
    let dir = tempfile::tempdir().unwrap();
    let cache = ResponseCache::open(&dir.path().join("c.db")).unwrap();
    (dir, Arc::new(cache))
}

#[tokio::test]
async fn repeated_requests_are_served_from_cache() {
    let (ep, requests) = mock("_search").await;
    let (_dir, cache) = temp_cache();
    let wf = service(ep.clone(), Some(cache.clone()));
    let s = Search {
        text: Some("shield".into()),
        ..Default::default()
    };
    wf.search(&s).await.unwrap();
    wf.search(&s).await.unwrap();
    assert_eq!(requests.load(Ordering::SeqCst), 1);

    // A second process (a fresh service on the same file) hits the cache too.
    let other = service(ep, Some(cache));
    other.search(&s).await.unwrap();
    assert_eq!(requests.load(Ordering::SeqCst), 1);

    // A different request is a miss.
    let s2 = Search {
        text: Some("heal".into()),
        ..Default::default()
    };
    wf.search(&s2).await.unwrap();
    assert_eq!(requests.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn without_a_cache_every_request_goes_out() {
    let (ep, requests) = mock("_search").await;
    let wf = service(ep, None);
    let s = Search::default();
    wf.search(&s).await.unwrap();
    wf.search(&s).await.unwrap();
    assert_eq!(requests.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn failures_are_not_cached() {
    let (ep, requests) = mock("fail").await;
    let (_dir, cache) = temp_cache();
    let wf = service(ep, Some(cache.clone()));
    assert!(wf.search(&Search::default()).await.is_err());
    assert!(cache.status().unwrap().is_empty());
    assert_eq!(requests.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn search_reports_total_offset_and_next_page() {
    let (ep, _) = mock("_search").await;
    let wf = service(ep, None);
    let page = wf
        .search(&Search {
            offset: 4,
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!((page.total, page.offset, page.docs.len()), (25, 4, 2));
    assert_eq!(page.next_offset(), Some(6));
}

#[tokio::test]
async fn lookup_by_name_picks_and_lists_same_name() {
    let (ep, _) = mock("_search").await;
    let wf = service(ep, None);
    let p = wf
        .lookup(&Lookup::named("Shield", None))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(p.best.category.as_deref(), Some("spell"));
    assert_eq!(p.same_name.len(), 1);
}

#[tokio::test]
async fn lookup_by_url_takes_the_first_hit() {
    let (ep, _) = mock("_search").await;
    let wf = service(ep, None);
    let l = Lookup {
        url: Some("/Implements.aspx?ID=11".into()),
        ..Default::default()
    };
    let p = wf.lookup(&l).await.unwrap().unwrap();
    assert_eq!(p.best.category.as_deref(), Some("implement"));
    assert!(p.same_name.is_empty());
}

#[tokio::test]
async fn categories_are_fetched_once_and_resolve_input() {
    let (ep, requests) = mock("_search").await;
    let wf = service(ep, None);
    assert_eq!(wf.categories().await.unwrap().len(), 2);
    assert_eq!(wf.resolve_category("Spells").await, Ok("spell".into()));
    assert_eq!(wf.resolve_category("rules").await, Ok("rules".into()));
    assert!(matches!(
        wf.resolve_category("spel").await,
        Err(CategoryError::Suggested { suggestion, .. }) if suggestion == "spell"
    ));
    assert_eq!(requests.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn resolve_category_falls_back_to_builtin_list_offline() {
    let (ep, _) = mock("fail").await;
    let wf = service(ep, None);
    // "deity" is not in the mock's live list but is built in.
    assert_eq!(wf.resolve_category("deities").await, Ok("deity".into()));
}

#[test]
fn page_total_label_marks_elasticsearch_lower_bounds() {
    // Regression: an unfiltered PF2e search said "Found 10000 match(es)";
    // Elasticsearch stops counting there (the index holds ~39k).
    let page = |total| wayfinder_core::Page {
        total,
        offset: 0,
        docs: Vec::new(),
    };
    assert_eq!(page(634).total_label(), "634");
    assert!(!page(634).total_is_lower_bound());
    assert_eq!(page(10_000).total_label(), "10000+");
}
