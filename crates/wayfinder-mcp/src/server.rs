//! The MCP server: tool definitions and the `ServerHandler` implementation.
//!
//! All AON network I/O and the document model come from `wayfinder_core`; this
//! crate only builds queries, selects a game, and formats results for the model.

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerInfo};
use rmcp::{ErrorData, ServerHandler, tool, tool_handler, tool_router};
use serde_json::Value;

use wayfinder_core::aon::{AonClient, GameSystem, parse_documents, parse_total};

use crate::format::{format_detail, format_summary};
use crate::params::{GameParams, GetParams, SearchParams, common_categories_hint, game_system};
use crate::query::{build_categories_query, build_get_query, build_search_query};

/// Build an AON client, honoring `WAYFINDER_AON_ENDPOINT` if set (points the
/// client at a mirror/proxy or a test server instead of live Nethys).
fn build_client(system: GameSystem) -> anyhow::Result<AonClient> {
    Ok(match std::env::var("WAYFINDER_AON_ENDPOINT") {
        Ok(ep) if !ep.trim().is_empty() => AonClient::with_endpoint(system, ep)?,
        _ => AonClient::new(system)?,
    })
}

/// Pathfinder 2e / Starfinder 2e MCP server backed by Archives of Nethys.
#[derive(Clone)]
pub struct WayfinderServer {
    pf2e: AonClient,
    sf2e: AonClient,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl WayfinderServer {
    /// Construct the server with AoN clients for both games.
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            pf2e: build_client(GameSystem::Pathfinder)?,
            sf2e: build_client(GameSystem::Starfinder)?,
            tool_router: Self::tool_router(),
        })
    }

    /// Resolve the client for the requested game (defaults to Pathfinder 2e).
    fn client(&self, game: Option<&str>) -> Result<&AonClient, ErrorData> {
        match game_system(game).map_err(|e| ErrorData::invalid_params(e, None))? {
            GameSystem::Pathfinder => Ok(&self.pf2e),
            GameSystem::Starfinder => Ok(&self.sf2e),
        }
    }

    #[tool(
        description = "Search Pathfinder 2e or Starfinder 2e game data on Archives of Nethys. Set \
        `game` to \"pf2e\" (default) or \"sf2e\". Combine free-text `query` with optional filters \
        (category, traits, level range, source, rarity). Returns a compact list of matches with \
        names, levels, traits, summaries, and URLs. Use `get` to retrieve the full text of a \
        specific entry."
    )]
    async fn search(
        &self,
        Parameters(params): Parameters<SearchParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let client = self.client(params.game.as_deref())?;
        let raw = client
            .search_raw(&build_search_query(&params))
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let total = parse_total(&raw).unwrap_or(0);
        let entries =
            parse_documents(&raw).map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        let base = client.system.base_url();
        let text = if entries.is_empty() {
            "No results found. Try a broader query, fewer filters, or check the category name \
             with `list_categories`."
                .to_string()
        } else {
            let mut out = format!("Found {} match(es); showing {}:\n", total, entries.len());
            for (i, e) in entries.iter().enumerate() {
                out.push('\n');
                out.push_str(&format_summary(i + 1, e, base));
            }
            out
        };
        Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
    }

    #[tool(
        description = "Fetch the full details of a single entry from Archives of Nethys by exact \
        `name` (optionally narrowed by `category`) or by AoN `url`. Set `game` to \"pf2e\" (default) \
        or \"sf2e\". Legacy pre-remaster names are matched too (e.g. \"Magic Missile\" resolves to \
        \"Force Barrage\"). Returns the complete rules text."
    )]
    async fn get(
        &self,
        Parameters(params): Parameters<GetParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let client = self.client(params.game.as_deref())?;
        let base = client.system.base_url();
        let body =
            build_get_query(&params, base).map_err(|e| ErrorData::invalid_params(e, None))?;
        let raw = client
            .search_raw(&body)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let entry = parse_documents(&raw)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?
            .into_iter()
            .next();

        let text = match entry {
            Some(e) => format_detail(&e, base),
            None => "No matching entry found. Check the spelling, try the `search` tool, or \
                     provide a `category` to disambiguate."
                .to_string(),
        };
        Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
    }

    #[tool(
        description = "List the available Archives of Nethys content categories (e.g. spell, feat, \
        creature, equipment) with how many entries each has, for the chosen `game` (\"pf2e\" default \
        or \"sf2e\"). Use these values for the `category` filter on `search`."
    )]
    async fn list_categories(
        &self,
        Parameters(params): Parameters<GameParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let client = self.client(params.game.as_deref())?;
        let raw = client
            .search_raw(&build_categories_query())
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let cats = parse_category_buckets(&raw).map_err(|e| ErrorData::internal_error(e, None))?;

        let mut out = format!(
            "{} categories on {} (name: entry count):\n",
            cats.len(),
            client.system.label()
        );
        for (name, count) in &cats {
            out.push_str(&format!("\n- {name}: {count}"));
        }
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }
}

/// Parse the terms-aggregation buckets from a `list_categories` response.
fn parse_category_buckets(response: &Value) -> Result<Vec<(String, i64)>, String> {
    let buckets = response
        .pointer("/aggregations/cats/buckets")
        .and_then(Value::as_array)
        .ok_or_else(|| "unexpected aggregation response from AoN".to_string())?;
    Ok(buckets
        .iter()
        .filter_map(|b| {
            let key = b.get("key")?.as_str()?.to_string();
            let count = b.get("doc_count")?.as_i64().unwrap_or(0);
            Some((key, count))
        })
        .collect())
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for WayfinderServer {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::new(ServerCapabilities::builder().enable_tools().build());
        info.server_info = Implementation::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        info.with_instructions(format!(
            "Query Pathfinder 2e and Starfinder 2e game data from Archives of Nethys. Every tool \
             takes an optional `game` parameter: \"pf2e\" (Pathfinder 2e, the default) or \"sf2e\" \
             (Starfinder 2e). Use `search` to find entries (filter by category, traits, level, \
             source, rarity), `get` to read the full text of a specific entry, and \
             `list_categories` to discover valid categories. Common categories: {}.",
            common_categories_hint()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::parse_category_buckets;
    use serde_json::json;

    #[test]
    fn parses_aggregation_buckets() {
        let resp = json!({"aggregations": {"cats": {"buckets": [
            {"key": "spell", "doc_count": 405},
            {"key": "feat", "doc_count": 2130}
        ]}}});
        let cats = parse_category_buckets(&resp).unwrap();
        assert_eq!(
            cats,
            vec![("spell".to_string(), 405), ("feat".to_string(), 2130)]
        );
    }

    #[test]
    fn errors_on_missing_aggregation() {
        assert!(parse_category_buckets(&json!({"hits": {}})).is_err());
    }
}
