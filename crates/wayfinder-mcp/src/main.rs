//! wayfinder-mcp: an MCP server for Pathfinder 2e and Starfinder 2e game data.
//!
//! Queries Archives of Nethys via its Elasticsearch backend, reusing
//! `wayfinder_core`'s AON client, document model, and TLS (rustls + ring). It
//! exposes three tools over stdio JSON-RPC: `search`, `get`, `list_categories`.

mod format;
mod params;
mod query;
mod server;

use anyhow::Context;
use clap::Parser;
use rmcp::ServiceExt;
use rmcp::transport::stdio;

use crate::server::WayfinderServer;

/// MCP server exposing Archives of Nethys PF2e / SF2e data over stdio JSON-RPC.
///
/// Takes no options; run it and connect an MCP client to its stdio. `--version`
/// and `--help` are handled by clap (and exit before the server starts, which
/// otherwise blocks waiting for the JSON-RPC handshake on stdin).
#[derive(Parser)]
#[command(name = "wayfinder-mcp", version, about)]
struct Cli {}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Cli::parse();

    // All logging goes to stderr; stdout is reserved for the MCP JSON-RPC stream.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    tracing::info!("starting wayfinder-mcp server (stdio)");

    let service = WayfinderServer::new()?
        .serve(stdio())
        .await
        .context("failed to start MCP server over stdio")?;

    service.waiting().await?;
    Ok(())
}
