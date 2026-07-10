//! wayfinder-core.
//!
//! The engine behind the `wf` CLI: a client, cache, search layer, and renderer
//! for [Archives of Nethys](https://2e.aonprd.com) Pathfinder 2e and Starfinder
//! 2e game data. It talks to AON's Elasticsearch backend, caches documents in a
//! local SQLite store with TTLs, and turns AON's HTML/markdown into structured
//! content blocks that consumers can render however they like.
//!
//! The four modules mirror that flow:
//!
//! - [`aon`] -- the Elasticsearch [`client`](aon::client), the
//!   [`query`](aon::query) builder, the document [`models`](aon::models), and
//!   the known [`categories`](aon::categories) with their filterable fields.
//! - [`cache`] -- a SQLite-backed [`CacheStore`](cache::store::CacheStore) with
//!   per-category TTLs.
//! - [`search`] -- the unified [`SearchService`](search::SearchService) that
//!   merges cache and live client, plus legacy/remaster handling.
//! - [`render`] -- AON HTML/markdown into [`ContentBlock`](render::ContentBlock)s
//!   and `Span`s; colorization is opt-in so non-terminal consumers stay clean.
//!
//! TLS uses rustls with the ring provider; [`AonClient::new`](aon::AonClient::new)
//! installs it, so there is no dependency on OpenSSL or aws-lc.

pub mod aon;
pub mod cache;
pub mod render;
pub mod search;
