# wayfinder-core

[![crates.io](https://img.shields.io/crates/v/wayfinder-core.svg)](https://crates.io/crates/wayfinder-core)
[![docs.rs](https://docs.rs/wayfinder-core/badge.svg)](https://docs.rs/wayfinder-core)

The engine behind the [`wf`](https://crates.io/crates/wayfinder-cli) CLI: a
client, cache, search layer, and renderer for [Archives of Nethys](https://2e.aonprd.com)
Pathfinder 2e and Starfinder 2e game data.

- **`aon`** — Elasticsearch client, query builder, document models, and the known
  categories with their filterable fields.
- **`cache`** — a SQLite-backed store with per-category TTLs.
- **`search`** — a unified service that merges cache and live client, with
  legacy/remaster handling.
- **`render`** — AON HTML/markdown into structured content blocks and spans;
  colorization is opt-in.

TLS uses rustls with the ring provider, so there is no dependency on OpenSSL or
aws-lc.

```rust,no_run
use wayfinder_core::aon::{AonClient, GameSystem, SearchQuery};

# async fn run() -> anyhow::Result<()> {
let client = AonClient::new(GameSystem::Pathfinder)?;
let results = client.search(&SearchQuery::new().category("spell").name("Fireball")).await?;
for doc in &results {
    println!("{}", doc.name.as_deref().unwrap_or("?"));
}
# Ok(())
# }
```

## License

MIT.
