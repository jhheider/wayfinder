//! wayfinder-core.
//!
//! The engine behind the `wf` CLI and the `wayfinder-mcp` server: a client,
//! response cache, search service, and renderer for
//! [Archives of Nethys](https://2e.aonprd.com) Pathfinder 2e and Starfinder 2e
//! game data.
//!
//! Frontends go through [`Wayfinder`], which owns the shared policy: the
//! [`Search`](aon::Search) and [`Lookup`](aon::Lookup) requests it sends to
//! AON's Elasticsearch backend, remaster handling, category resolution
//! against the live index, and a SQLite [`ResponseCache`](cache::ResponseCache)
//! that every wayfinder process shares.
//!
//! - [`aon`] -- the HTTP [`client`](aon::client), requests
//!   ([`query`](aon::query), [`lookup`](aon::lookup)), the document
//!   [`models`](aon::models), and the known [`categories`](aon::categories).
//! - [`cache`] -- the response cache.
//! - [`service`] -- [`Wayfinder`] and result helpers.
//! - [`render`] -- AON HTML/markdown into [`ContentBlock`](render::ContentBlock)s,
//!   then markdown or terminal `Span`s; colorization is opt-in.
//!
//! TLS uses rustls with the ring provider; [`AonClient::new`](aon::AonClient::new)
//! installs it, so there is no dependency on OpenSSL or aws-lc.

pub mod aon;
pub mod cache;
pub mod error;
pub mod render;
pub mod service;

pub use error::{Error, Result};
pub use service::{Page, Wayfinder};
