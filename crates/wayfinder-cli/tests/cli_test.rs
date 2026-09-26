//! End-to-end tests that run the built `wf` binary against an in-process HTTP
//! mock (via WAYFINDER_AON_ENDPOINT), exercising the command dispatch, output
//! formats, and client wiring without touching live Nethys.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Nothing listens here, so "offline" commands exercise their fallbacks
/// instead of reaching live Nethys.
const DEAD: &str = "http://127.0.0.1:9/_search";

const HIT: &str = r#"{"hits":{"total":{"value":1},"hits":[{"_source":{
    "id":"spell-119","name":"Fireball","category":"spell","type":"Spell","level":3,
    "trait":["Fire"],"url":"/Spells.aspx?ID=119","text":"An explosion of fire.",
    "markdown":"An explosion of fire."}}]}}"#;

/// Each game's live categories, as the mock serves them (a slice of the real
/// lists: shared, PF2e-only and SF2e-only entries).
const PF_CATS: &str = r#"{"aggregations":{"cats":{"buckets":[
    {"key":"spell","doc_count":2762},{"key":"hellknight-order","doc_count":9},
    {"key":"eidolon","doc_count":14}]}}}"#;
const SF_CATS: &str = r#"{"aggregations":{"cats":{"buckets":[
    {"key":"spell","doc_count":405},{"key":"starship-scene","doc_count":10},
    {"key":"planet","doc_count":20}]}}}"#;

/// Spawn a mock `_search` endpoint that answers every request with one canned
/// hit, counting requests. Loops for the life of the test process.
fn spawn_counting_mock() -> (String, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let count = Arc::new(AtomicUsize::new(0));
    let seen = count.clone();
    std::thread::spawn(move || {
        while let Ok((mut sock, _)) = listener.accept() {
            seen.fetch_add(1, Ordering::SeqCst);
            let mut buf = [0u8; 4096];
            let n = sock.read(&mut buf).unwrap_or(0);
            let req = String::from_utf8_lossy(&buf[..n]);
            // Category aggregations get the game's list (the index is in the
            // query string); everything else gets the canned hit.
            let body = match (req.contains("\"aggs\""), req.contains("index=aonsf")) {
                (true, true) => SF_CATS,
                (true, false) => PF_CATS,
                _ => HIT,
            };
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

fn spawn_mock() -> String {
    spawn_counting_mock().0
}

/// A fresh, empty home directory for one test.
fn fresh_home(tag: &str) -> PathBuf {
    let home = std::env::temp_dir().join(format!("wayfinder-cli-it-{tag}"));
    let _ = std::fs::remove_dir_all(&home);
    std::fs::create_dir_all(&home).unwrap();
    home
}

/// Run `wf` with its cache in `home`, against `endpoint` (or a dead one).
fn run_wf_in(home: &Path, endpoint: Option<&str>, args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(args)
        .env("WAYFINDER_CACHE", home.join("cache.db"))
        .env("WAYFINDER_AON_ENDPOINT", endpoint.unwrap_or(DEAD))
        .output()
        .expect("failed to run wf");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn run_wf(tag: &str, endpoint: Option<&str>, args: &[&str]) -> String {
    run_wf_in(&fresh_home(tag), endpoint, args)
}

#[test]
fn version_flag() {
    let want = concat!("wf ", env!("CARGO_PKG_VERSION"));
    let out = run_wf("version", None, &["--version"]);
    assert!(out.contains(want), "wanted {want:?} in {out:?}");
}

#[test]
fn categories_lists_groups_offline() {
    let out = run_wf("cats", None, &["categories"]);
    assert!(out.contains("Categories"), "{out}");
}

#[test]
fn categories_list_only_the_selected_games_categories() {
    // Regression (PR #20's bug): `wf --sf2e categories` printed the merged
    // static list, so PF2e-only categories appeared under the SF2e header and
    // SF2e's own were missing. It now lists the game's live categories.
    let ep = spawn_mock();
    let sf = run_wf("cats-sf", Some(&ep), &["--sf2e", "categories"]);
    assert!(
        sf.contains("starship-scene") && sf.contains("planet"),
        "{sf}"
    );
    assert!(
        !sf.contains("hellknight-order") && !sf.contains("eidolon"),
        "{sf}"
    );
    let pf = run_wf("cats-pf", Some(&ep), &["categories"]);
    assert!(
        pf.contains("hellknight-order") && pf.contains("eidolon"),
        "{pf}"
    );
    assert!(
        !pf.contains("starship-scene") && !pf.contains("planet"),
        "{pf}"
    );
}

#[test]
fn fields_lists_a_category_offline() {
    let out = run_wf("fields", None, &["fields", "deity"]);
    assert!(out.to_lowercase().contains("deity"), "{out}");
}

#[test]
fn cache_status_on_empty_cache() {
    let out = run_wf("cachestatus", None, &["cache", "status"]);
    assert!(out.contains("Cache is empty"), "{out}");
}

#[test]
fn a_second_process_is_served_from_the_cache() {
    let (ep, requests) = spawn_counting_mock();
    let home = fresh_home("cachehit");
    let args = ["--format", "json", "search", "spell/Fireball"];
    // First run resolves the category (1 request) and searches (1 request).
    assert!(run_wf_in(&home, Some(&ep), &args).contains("Fireball"));
    let first = requests.load(Ordering::SeqCst);
    assert!(run_wf_in(&home, Some(&ep), &args).contains("Fireball"));
    assert_eq!(
        requests.load(Ordering::SeqCst),
        first,
        "second run hit the network"
    );
    let status = run_wf_in(&home, Some(&ep), &["cache", "status"]);
    assert!(status.contains("PF2e"), "{status}");
    assert!(run_wf_in(&home, Some(&ep), &["cache", "clear"]).contains("Cleared"));
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
