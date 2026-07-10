//! End-to-end tests that run the built `wf` binary against an in-process HTTP
//! mock (via WAYFINDER_AON_ENDPOINT), exercising the command dispatch, output
//! formats, and client wiring without touching live Nethys.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;

const HIT: &str = r#"{"hits":{"total":{"value":1},"hits":[{"_source":{
    "id":"spell-119","name":"Fireball","category":"spell","type":"Spell","level":3,
    "trait":["Fire"],"url":"/Spells.aspx?ID=119","text":"An explosion of fire.",
    "markdown":"An explosion of fire."}}]}}"#;

/// Spawn a mock `_search` endpoint that answers every request with one canned
/// hit. Loops for the life of the test process (harmless detached thread).
fn spawn_mock() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        while let Ok((mut sock, _)) = listener.accept() {
            let mut buf = [0u8; 4096];
            let _ = sock.read(&mut buf);
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                HIT.len(),
                HIT
            );
            let _ = sock.write_all(resp.as_bytes());
        }
    });
    format!("http://{addr}/_search")
}

fn run_wf(tag: &str, endpoint: Option<&str>, args: &[&str]) -> String {
    let home = std::env::temp_dir().join(format!("wayfinder-cli-it-{tag}"));
    let _ = std::fs::remove_dir_all(&home);
    std::fs::create_dir_all(&home).unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_wf"));
    cmd.args(args)
        .env("HOME", &home)
        .env("XDG_DATA_HOME", &home);
    if let Some(ep) = endpoint {
        cmd.env("WAYFINDER_AON_ENDPOINT", ep);
    }
    let out = cmd.output().expect("failed to run wf");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn version_flag() {
    assert!(run_wf("version", None, &["--version"]).contains("wf 0.1.0"));
}

#[test]
fn categories_lists_groups_offline() {
    let out = run_wf("cats", None, &["categories"]);
    assert!(out.contains("Categories"), "{out}");
}

#[test]
fn fields_lists_a_category_offline() {
    let out = run_wf("fields", None, &["fields", "deity"]);
    assert!(out.to_lowercase().contains("deity"), "{out}");
}

#[test]
fn cache_status_on_empty_cache() {
    let out = run_wf("cachestatus", None, &["cache", "status"]);
    assert!(out.to_lowercase().contains("cache"), "{out}");
}

#[test]
fn search_json_via_mock() {
    let ep = spawn_mock();
    let out = run_wf(
        "searchjson",
        Some(&ep),
        &[
            "--format",
            "json",
            "search",
            "spellfire",
            "--name",
            "Fireball",
        ],
    );
    assert!(out.contains("Fireball"), "{out}");
}

#[test]
fn show_md_via_mock() {
    let ep = spawn_mock();
    let out = run_wf(
        "showmd",
        Some(&ep),
        &["--format", "md", "show", "spell", "Fireball"],
    );
    assert!(out.to_lowercase().contains("fire"), "{out}");
}

#[test]
fn sf2e_search_pretty_via_mock() {
    let ep = spawn_mock();
    let out = run_wf(
        "sf2e",
        Some(&ep),
        &["--sf2e", "search", "spellfire", "--name", "Fireball"],
    );
    assert!(out.contains("Fireball"), "{out}");
}
