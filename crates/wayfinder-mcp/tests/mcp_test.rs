//! End-to-end test: run the built `wayfinder-mcp` binary, speak MCP JSON-RPC
//! over stdio, and drive its tools against an in-process HTTP mock (via
//! WAYFINDER_AON_ENDPOINT) so no live Nethys call is made.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

const HIT: &str = r#"{"hits":{"total":{"value":1},"hits":[{"_source":{
    "id":"spell-119","name":"Fireball","category":"spell","type":"Spell","level":3,
    "trait":["Fire"],"url":"/Spells.aspx?ID=119","text":"An explosion of fire."}}]}}"#;

const CATS: &str = r#"{"aggregations":{"cats":{"buckets":[{"key":"spell","doc_count":405}]}}}"#;

/// A mock `_search` endpoint and a count of the requests it has answered.
fn spawn_mock() -> (String, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let count = Arc::new(AtomicUsize::new(0));
    let seen = count.clone();
    std::thread::spawn(move || {
        while let Ok((mut sock, _)) = listener.accept() {
            seen.fetch_add(1, Ordering::SeqCst);
            let mut buf = [0u8; 8192];
            let n = sock.read(&mut buf).unwrap_or(0);
            // list_categories posts an aggregation body; everything else is a hit list.
            let req = String::from_utf8_lossy(&buf[..n]);
            let body = if req.contains("aggs") { CATS } else { HIT };
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = sock.write_all(resp.as_bytes());
        }
    });
    (format!("http://{addr}/_search"), count)
}

/// Run the server with the given JSON-RPC request lines on stdin, caching
/// off; return stdout.
fn drive(requests: &[&str]) -> String {
    drive_with(&spawn_mock().0, "off", requests)
}

/// Run the server against `endpoint` with `WAYFINDER_CACHE=cache`.
fn drive_with(endpoint: &str, cache: &str, requests: &[&str]) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_wayfinder-mcp"))
        .env("WAYFINDER_AON_ENDPOINT", endpoint)
        .env("WAYFINDER_CACHE", cache)
        .env("RUST_LOG", "error")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn wayfinder-mcp");
    {
        let mut stdin = child.stdin.take().unwrap();
        for r in requests {
            writeln!(stdin, "{r}").unwrap();
        }
    } // drop stdin -> EOF -> server exits
    let out = child.wait_with_output().expect("wait wayfinder-mcp");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

const INIT: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"t","version":"0"}}}"#;
const INITED: &str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;

#[test]
fn initialize_reports_server_info_and_tools() {
    let out = drive(&[
        INIT,
        INITED,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
    ]);
    assert!(out.contains("wayfinder-mcp"), "{out}");
    assert!(out.contains("search") && out.contains("get") && out.contains("list_categories"));
}

#[test]
fn search_tool_returns_a_hit() {
    let out = drive(&[
        INIT,
        INITED,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"search","arguments":{"query":"fireball","category":"spell"}}}"#,
    ]);
    assert!(out.contains("Fireball"), "{out}");
}

#[test]
fn get_tool_returns_detail() {
    let out = drive(&[
        INIT,
        INITED,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"get","arguments":{"name":"Fireball","category":"spell"}}}"#,
    ]);
    assert!(out.contains("# Fireball"), "{out}");
}

#[test]
fn list_categories_returns_counts() {
    let out = drive(&[
        INIT,
        INITED,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"list_categories","arguments":{"game":"sf2e"}}}"#,
    ]);
    assert!(out.contains("spell") && out.contains("405"), "{out}");
}

#[test]
fn unknown_category_is_a_tool_error_with_a_suggestion() {
    // Regression: an unknown category (e.g. a typo) filtered to zero hits and
    // looked like "no results"; it must say so and suggest the real one.
    let out = drive(&[
        INIT,
        INITED,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"search","arguments":{"category":"spel"}}}"#,
    ]);
    assert!(out.contains(r#""isError":true"#), "{out}");
    assert!(out.contains(r#"did you mean \"spell\"?"#), "{out}");
}

#[test]
fn plural_category_resolves_and_search_reports_paging() {
    let out = drive(&[
        INIT,
        INITED,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"search","arguments":{"category":"Spells","offset":0}}}"#,
    ]);
    assert!(!out.contains(r#""isError":true"#), "{out}");
    assert!(out.contains("showing 1-1"), "{out}");
}

#[test]
fn tools_advertise_read_only() {
    let out = drive(&[
        INIT,
        INITED,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
    ]);
    assert!(out.contains(r#""readOnlyHint":true"#), "{out}");
}

#[test]
fn a_second_server_is_served_from_the_shared_cache() {
    let (ep, requests) = spawn_mock();
    let dir = std::env::temp_dir().join(format!("wayfinder-mcp-it-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let cache = dir.join("cache.db");
    let call = r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"search","arguments":{"query":"fireball"}}}"#;
    let first = drive_with(&ep, cache.to_str().unwrap(), &[INIT, INITED, call]);
    assert!(first.contains("Fireball"), "{first}");
    assert_eq!(requests.load(Ordering::SeqCst), 1);
    let second = drive_with(&ep, cache.to_str().unwrap(), &[INIT, INITED, call]);
    assert!(second.contains("Fireball"), "{second}");
    assert_eq!(
        requests.load(Ordering::SeqCst),
        1,
        "second server hit the network"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
