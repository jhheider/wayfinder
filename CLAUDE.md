# Wayfinder -- Archives of Nethys data tools (PF2e / SF2e)

A Rust workspace of tools for querying Archives of Nethys Pathfinder 2e and
Starfinder 2e game data. One AON client, three frontends.

## Project Structure
- **wayfinder-core** -- library: AON Elasticsearch client, SQLite cache, search,
  rendering, domain types. The single source of truth for AON access; the CLI
  and MCP server both consume it.
- **wayfinder-cli** (bin: `wf`) -- human-facing terminal tool for searching and
  browsing AON data (colorized output, cache management).
- **wayfinder-mcp** (bin: `wayfinder-mcp`) -- MCP server exposing AON data to LLM
  tools (`search`, `get`, `list_categories`) over stdio JSON-RPC.

> The `waybuilder` TUI character builder was shelved (it chased Pathbuilder 2e
> parity, a moving target). Its history lives on the `archive/waybuilder`
> branch -- do not re-add it to the workspace.

## Build & Test
```
cargo build --workspace
cargo clippy --workspace --all-targets --all-features   # -D warnings in CI
cargo test --workspace
cargo run -p wayfinder-cli -- search deity -f domain=Dragon
cargo run -p wayfinder-cli -- show spell Fireball
cargo run -p wayfinder-cli -- categories
cargo run -p wayfinder-cli -- --sf2e search class
cargo run -p wayfinder-cli -- --format json search spell --name Fireball
cargo run -p wayfinder-cli -- cache fetch spell
# Drive the MCP server (stdio JSON-RPC):
cargo run -p wayfinder-mcp
```

## Code Style
- `rustfmt` and `clippy` always, default settings; CI treats warnings as errors.
- Edition 2024, workspace dep inheritance (version/edition/authors/repository/license).
- Tests: pure logic in per-crate `tests/`; small unit tests inline where they
  document a function (e.g. the MCP query builders).
- Target ~100-120 lines per file; split into submodules when larger.
- Use `cargo add` for new dependencies (ensures latest versions).

## Dependency preferences
- **TLS: rustls + ring, never openssl/aws-lc.** reqwest uses
  `default-features = false, features = ["json", "rustls-no-provider"]`; the
  workspace pins `rustls` with `default-features = false, features = ["ring", ...]`;
  `AonClient::new` installs the ring provider. After any dep change, confirm
  `cargo tree -i aws-lc-sys` and `-i openssl-sys` both find nothing.
- Avoid super-bloat crate trees and crates that shell out to external builds.

## CI / Release
Four thin caller workflows use the reusable `jhheider/rust-ci@v1` workflows:
`ci.yml`, `style.yml`, `audit.yml`, `release.yml`. Release publishes all three
crates to crates.io (dependency order: core → cli → mcp), ships prebuilt `wf`
binaries + a Homebrew formula; `wayfinder-mcp` is distributed via
`cargo install wayfinder-mcp`. `wf` implements `--manpage`/`--completions` for
packager doc generation (`gen-docs: true`).

## Data Sources
- **PF2e**: `POST https://elasticsearch.aonprd.com/aon/_search` → index `aon70`
  (~39k docs, 93 categories), base site `https://2e.aonprd.com`.
- **SF2e**: `POST https://elasticsearch.aonprd.com/aonsf/_search` → index
  `aonsf10` (~6k docs, 52 categories), base site `https://2e.aonsrd.com`.
- Category field is `keyword` (use `term` queries, lowercase singular: `spell`,
  `deity`). `level` may arrive as a number OR a numeric string -- `Document`
  deserializes it tolerantly.

## CLI Output Formats
- `--format pretty` (default): colorized terminal with emoji, styled text
- `--format json`: raw JSON for piping/scripting
- `--format md`: raw AON markdown

## Key Modules (wayfinder-core)
- `aon::client` -- `AonClient` (+ `search_raw` for custom ES bodies, and the
  public `parse_documents` / `parse_total` helpers), `GameSystem` (PF2e/SF2e)
- `aon::query` -- `SearchQuery` builder (CLI-oriented)
- `aon::models` -- `Document` serde struct with `#[serde(flatten)]` extra fields
- `aon::categories` -- known categories, grouped hierarchy, filterable fields
- `aon::parse` -- category resolution, fuzzy suggestion, compound parsing
- `cache::store` -- `CacheStore` SQLite layer with TTL
- `render` -- AON HTML/markdown → content blocks / `Vec<Span>`; colorize is opt-in
- `search` -- unified `SearchService` (cache + client), category fetch

## wayfinder-mcp notes
- Keeps its own MCP-tuned ES query builders (`query.rs`: sort, level range,
  `_source` projection, category aggregation) and param structs (`params.rs`,
  schemars-described), but routes ALL network I/O and the document model through
  `wayfinder-core` -- no duplicated AON client.
- `rmcp` 2.x: tool results use `ContentBlock` (not `Content`).
- Verify tool changes against LIVE AON by driving stdio JSON-RPC, not just a
  compile -- the tool surface must keep matching real Nethys results.

## References
`references/` (.gitignored) holds AON exploration aids: `category_fields.json`,
`sample_documents.json`, AON frontend HTML/JS, SF2e exemplars.
