# wayfinder-core

[![crates.io](https://img.shields.io/crates/v/wayfinder-core.svg)](https://crates.io/crates/wayfinder-core)
[![docs.rs](https://docs.rs/wayfinder-core/badge.svg)](https://docs.rs/wayfinder-core)

The engine behind the [`wf`](https://crates.io/crates/wayfinder-cli) CLI and the
[`wayfinder-mcp`](https://crates.io/crates/wayfinder-mcp) server: a client,
response cache, search service, and renderer for
[Archives of Nethys](https://2e.aonprd.com) Pathfinder 2e and Starfinder 2e game
data.

- **`Wayfinder`** -- the service both frontends use: searches, lookups by name
  or URL (picking among same-named entries), live categories and category
  resolution, remaster handling, all through the response cache.
- **`aon`** -- the HTTP client, the `Search` and `Lookup` requests, document
  models, and the known categories with their filterable fields.
- **`cache`** -- a SQLite response cache keyed by the exact request, shared by
  every wayfinder process (`WAYFINDER_CACHE`: a path, or `off`).
- **`render`** -- AON HTML/markdown into structured content blocks, then
  markdown or terminal spans; colorization is opt-in.

TLS uses rustls with the ring provider, so there is no dependency on OpenSSL or
aws-lc.

```rust,no_run
use std::sync::Arc;
use wayfinder_core::Wayfinder;
use wayfinder_core::aon::{AonClient, GameSystem, Lookup, Search};
use wayfinder_core::cache::ResponseCache;

# async fn run() -> anyhow::Result<()> {
let cache = ResponseCache::from_env()?.map(Arc::new);
let wf = Wayfinder::new(AonClient::new(GameSystem::Pathfinder)?, cache);

let page = wf
    .search(&Search {
        category: Some("spell".into()),
        traits: vec!["fire".into()],
        max_level: Some(3),
        ..Search::default()
    })
    .await?;
for doc in &page.docs {
    println!("{}", doc.name.as_deref().unwrap_or("?"));
}

if let Some(pick) = wf.lookup(&Lookup::named("Fireball", Some("spell"))).await? {
    let md = wayfinder_core::render::render_markdown(&pick.best.content("https://2e.aonprd.com"));
    println!("{md}");
}
# Ok(())
# }
```

## License

MIT.
