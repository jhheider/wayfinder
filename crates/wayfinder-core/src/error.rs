//! The crate's error type.
//!
//! Public APIs return [`Result`]/[`enum@Error`] so downstream code can match on
//! specific failure modes. Binaries (the `wf` CLI, the MCP server) still use
//! `anyhow` and absorb these via `?`; only the library exposes typed errors.

use thiserror::Error;

/// An error from querying, caching, or parsing Archives of Nethys data.
#[derive(Debug, Error)]
pub enum Error {
    /// The HTTP request to Archives of Nethys failed (network, TLS, timeout).
    #[error("HTTP error talking to Archives of Nethys: {0}")]
    Http(#[from] reqwest::Error),

    /// Archives of Nethys returned a non-success HTTP status.
    #[error("Archives of Nethys returned HTTP {status}")]
    HttpStatus {
        /// The HTTP status code returned.
        status: u16,
    },

    /// A response could not be deserialized into the expected shape.
    #[error("failed to parse Archives of Nethys response: {0}")]
    Parse(#[from] serde_json::Error),

    /// A response was valid JSON but missing an expected field.
    #[error("unexpected Archives of Nethys response: {0}")]
    UnexpectedResponse(String),

    /// The local SQLite document cache failed.
    #[error("cache error: {0}")]
    Cache(#[from] rusqlite::Error),
}

/// Convenience alias for results from this crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;
