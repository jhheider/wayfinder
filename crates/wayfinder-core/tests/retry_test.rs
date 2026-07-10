//! Exercises `AonClient`'s live request path -- the 429/503 retry loop, backoff,
//! and status handling -- against a tiny in-process HTTP mock (no extra deps).

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use wayfinder_core::aon::{AonClient, GameSystem, SearchQuery};

const HIT: &str =
    r#"{"hits":{"total":{"value":1},"hits":[{"_source":{"name":"Fireball","category":"spell"}}]}}"#;

/// Serve one canned response per status in `statuses`, in order, then stop.
/// `Retry-After: 0` keeps the client's backoff instant so tests stay fast.
async fn mock_endpoint(statuses: Vec<u16>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        for code in statuses {
            let Ok((mut sock, _)) = listener.accept().await else {
                return;
            };
            let mut buf = [0u8; 2048];
            let _ = sock.read(&mut buf).await;
            let (line, body) = match code {
                200 => ("200 OK", HIT),
                429 => ("429 Too Many Requests", "{}"),
                503 => ("503 Service Unavailable", "{}"),
                _ => ("500 Internal Server Error", "{}"),
            };
            let resp = format!(
                "HTTP/1.1 {line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nRetry-After: 0\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = sock.write_all(resp.as_bytes()).await;
            let _ = sock.shutdown().await;
        }
    });
    format!("http://{addr}/_search")
}

fn client(url: String) -> AonClient {
    AonClient::with_endpoint(GameSystem::Pathfinder, url).unwrap()
}

#[tokio::test]
async fn succeeds_on_first_try() {
    let c = client(mock_endpoint(vec![200]).await);
    let docs = c
        .search(&SearchQuery::new().name("Fireball"))
        .await
        .unwrap();
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].name.as_deref(), Some("Fireball"));
}

#[tokio::test]
async fn retries_on_429_then_succeeds() {
    let c = client(mock_endpoint(vec![429, 200]).await);
    let docs = c
        .search(&SearchQuery::new().name("Fireball"))
        .await
        .unwrap();
    assert_eq!(docs.len(), 1);
}

#[tokio::test]
async fn retries_on_503_then_succeeds() {
    let c = client(mock_endpoint(vec![503, 200]).await);
    assert!(c.search(&SearchQuery::new().name("x")).await.is_ok());
}

#[tokio::test]
async fn gives_up_after_max_attempts() {
    // MAX_ATTEMPTS is 3, so three 429s exhaust the retries and surface an error.
    let c = client(mock_endpoint(vec![429, 429, 429]).await);
    let err = c.search(&SearchQuery::new().name("x")).await.unwrap_err();
    assert!(err.to_string().contains("429"), "{err}");
}

#[tokio::test]
async fn non_retriable_status_errors_immediately() {
    let c = client(mock_endpoint(vec![500]).await);
    let err = c.search(&SearchQuery::new().name("x")).await.unwrap_err();
    assert!(err.to_string().contains("500"), "{err}");
}
