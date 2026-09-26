# Wayfinder -- Archives of Nethys data tools (PF2e / SF2e)

A Rust workspace of tools for querying Archives of Nethys Pathfinder 2e and
Starfinder 2e game data. One AON client library, two frontends (a CLI and an
MCP server).

## Project Structure
- **wayfinder-core** -- library: AON Elasticsearch client, the `Wayfinder`
  service, SQLite response cache, rendering, domain types. The single source of
  truth for AON access AND policy (what a search/lookup sends, remaster
  handling, category resolution, picking among same-named entries, caching);
  the CLI and MCP server only map their input onto it and lay out its output.
  New behavior goes in core so both frontends get it.
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
cargo run -p wayfinder-cli -- search deity/ -f domain=Dragon
cargo run -p wayfinder-cli -- show spell Fireball
cargo run -p wayfinder-cli -- categories
cargo run -p wayfinder-cli -- --sf2e search class/
cargo run -p wayfinder-cli -- --format json search spell/Fireball
cargo run -p wayfinder-cli -- cache status
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

## Error handling
- **Library (wayfinder-core): typed errors, not anyhow.** The public API returns
  `wayfinder_core::Result<T>` / `Error` (a `thiserror` enum in `error.rs`;
  `aon::parse::CategoryError` is a separate typed error). Do not reintroduce
  `anyhow` into core's `[dependencies]`.
- **Binaries (wf, wayfinder-mcp): anyhow.** They `?` core's typed errors (anyhow
  absorbs any `std::error::Error`). Both binaries use `clap` (so `--version` /
  `--help` work); the MCP's `Cli` is an empty struct that just exits on those.

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
  public `parse_documents` / `parse_total` helpers; `from_env` honors
  `WAYFINDER_AON_ENDPOINT`), `GameSystem` (PF2e/SF2e, `FromStr` for game names)
- `aon::query` -- `Search` (every search filter, `Edition`, `Sort`, paging,
  validation) → ES body; `categories_body`
- `aon::lookup` -- `Lookup` (by name or URL) and `pick` (exact current name,
  then legacy name, then likely category; keeps the other same-named entries)
- `aon::models` -- `Document` serde struct with `#[serde(flatten)]` extra fields,
  plus shared helpers (`is_legacy`, `edition_note`, `absolute_url`,
  `notable_rarity`, `content`)
- `aon::categories` -- known categories, grouped hierarchy, filterable fields
- `aon::parse` -- category resolution, fuzzy suggestion, compound parsing
- `cache` -- `ResponseCache`: SQLite (WAL) cache of raw responses keyed by
  endpoint + index + request body, 24h TTL, shared by every process;
  `WAYFINDER_CACHE` = a path or `off`. Tests must set it (a temp path or `off`).
- `render` -- AON HTML/markdown → content blocks / `Vec<Span>`; colorize is opt-in
- `service` -- `Wayfinder` (search, lookup, cached live categories,
  `resolve_category` with built-in fallback offline) and `group_broad_results`

## wayfinder-mcp notes
- `params.rs` holds the schemars-described tool params and maps them onto core
  `Search`/`Lookup` (core's `schemars` feature derives `Sort`'s schema);
  `format.rs` lays results out for a model. No queries or AON logic here.
- Tool bodies return `anyhow::Result<String>`; `respond` turns failures into
  `isError` tool results (which the model sees), not JSON-RPC errors.
- `rmcp` 2.x: tool results use `ContentBlock` (not `Content`).
- Verify tool changes against LIVE AON by driving stdio JSON-RPC, not just a
  compile -- the tool surface must keep matching real Nethys results.
- stdio-only today; cloud clients (Claude.ai web/mobile, ChatGPT) need a remote
  HTTP transport (rmcp ships one) that is not wired yet. Per-client setup lives
  in `docs/mcp-setup.md`.

## References
`references/` (.gitignored) holds AON exploration aids: `category_fields.json`,
`sample_documents.json`, AON frontend HTML/JS, SF2e exemplars.
