//! The MCP server: tool definitions and the `ServerHandler` implementation.
//!
//! All AON network I/O and the document model come from `wayfinder_core`; this
//! crate only builds queries, selects a game, and formats results for the model.

use anyhow::{Context, bail};
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerInfo};
use rmcp::{ErrorData, ServerHandler, tool, tool_handler, tool_router};

use wayfinder_core::aon::{AonClient, GameSystem, parse_documents, parse_total};

use crate::format::{format_alternatives, format_detail, format_summary};
use crate::game::Game;
use crate::params::{GameParams, GetParams, SearchParams, common_categories_hint, game_system};
use crate::pick::pick;
use crate::query::{build_get_query, build_search_query};

/// Pathfinder 2e / Starfinder 2e MCP server backed by Archives of Nethys.
#[derive(Clone)]
pub struct WayfinderServer {
    pf2e: Game,
    sf2e: Game,
    tool_router: ToolRouter<Self>,
}

/// Turn a tool body's outcome into a result the model sees. Failures (bad
/// input, an unknown category, AON being unreachable) are tool errors with a
/// message, not protocol errors, so the model can correct and retry.
fn respond(outcome: anyhow::Result<String>) -> Result<CallToolResult, ErrorData> {
    Ok(match outcome {
        Ok(text) => CallToolResult::success(vec![ContentBlock::text(text)]),
        Err(e) => CallToolResult::error(vec![ContentBlock::text(format!("{e:#}"))]),
    })
}

#[tool_router]
impl WayfinderServer {
    /// Construct the server with AoN clients for both games.
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            pf2e: Game::new(AonClient::from_env(GameSystem::Pathfinder)?),
            sf2e: Game::new(AonClient::from_env(GameSystem::Starfinder)?),
            tool_router: Self::tool_router(),
        })
    }

    /// Resolve the requested game (defaults to Pathfinder 2e).
    fn game(&self, game: Option<&str>) -> anyhow::Result<&Game> {
        Ok(match game_system(game).map_err(anyhow::Error::msg)? {
            GameSystem::Pathfinder => &self.pf2e,
            GameSystem::Starfinder => &self.sf2e,
        })
    }

    #[tool(
        description = "Search Pathfinder 2e or Starfinder 2e game data on Archives of Nethys. Set \
        `game` to \"pf2e\" (default) or \"sf2e\". Combine free-text `query` with optional filters \
        (category, traits, level range, source, rarity). Returns a compact list of matches with \
        names, levels, traits, summaries, and URLs; page with `offset`. Remastered entries replace \
        their legacy versions unless `legacy` is true. Use `get` to read an entry's full text.",
        annotations(read_only_hint = true, open_world_hint = true)
    )]
    async fn search(
        &self,
        Parameters(params): Parameters<SearchParams>,
    ) -> Result<CallToolResult, ErrorData> {
        respond(self.run_search(params).await)
    }

    async fn run_search(&self, mut params: SearchParams) -> anyhow::Result<String> {
        let game = self.game(params.game.as_deref())?;
        if let Some(category) = &params.category {
            params.category = game.resolve_category(category).await?;
        }
        if let (Some(min), Some(max)) = (params.min_level, params.max_level)
            && min > max
        {
            bail!("min_level ({min}) is greater than max_level ({max})");
        }
        let raw = game.client.search_raw(&build_search_query(&params)).await?;
        let entries = parse_documents(&raw)?;
        if entries.is_empty() {
            return Ok("No results found. Try a broader query or fewer filters.".to_string());
        }

        let total = parse_total(&raw).unwrap_or(0);
        let first = params.effective_offset() as usize + 1;
        let last = first + entries.len() - 1;
        let mut out = format!("Found {total} match(es); showing {first}-{last}");
        if (last as i64) < total {
            out.push_str(&format!(" (pass offset={last} for more)"));
        }
        out.push_str(":\n");
        let base = game.client.system.base_url();
        for (i, e) in entries.iter().enumerate() {
            out.push('\n');
            out.push_str(&format_summary(first + i, e, base));
        }
        Ok(out)
    }

    #[tool(
        description = "Fetch the full rules text of one Archives of Nethys entry, by exact `name` \
        (optionally narrowed by `category`) or by AoN `url`. Set `game` to \"pf2e\" (default) or \
        \"sf2e\". Legacy pre-remaster names are matched too (\"Magic Missile\" finds \"Force \
        Barrage\"); set `legacy` for the pre-remaster version. When several entries share the \
        name, the most likely one is returned and the others are listed.",
        annotations(read_only_hint = true, open_world_hint = true)
    )]
    async fn get(
        &self,
        Parameters(params): Parameters<GetParams>,
    ) -> Result<CallToolResult, ErrorData> {
        respond(self.run_get(params).await)
    }

    async fn run_get(&self, mut params: GetParams) -> anyhow::Result<String> {
        let game = self.game(params.game.as_deref())?;
        if let Some(category) = &params.category {
            params.category = game.resolve_category(category).await?;
        }
        let base = game.client.system.base_url();
        let body = build_get_query(&params, base).map_err(anyhow::Error::msg)?;
        let candidates = parse_documents(&game.client.search_raw(&body).await?)?;

        let by_url = params.url.as_deref().is_some_and(|u| !u.trim().is_empty());
        let name = params.name.as_deref().unwrap_or_default();
        let picked = if by_url {
            candidates.into_iter().next().map(|best| (best, Vec::new()))
        } else {
            pick(name, candidates).map(|p| (p.best, p.same_name))
        };
        let Some((best, same_name)) = picked else {
            return Ok(
                "No matching entry found. Check the spelling, try the `search` tool, \
                       or drop `category`."
                    .to_string(),
            );
        };

        let mut out = format_detail(&best, base);
        if !same_name.is_empty() {
            out.push_str(&format!(
                "\n---\nOther entries named \"{}\" (pass `category` or `url` to get one):\n{}",
                name.trim(),
                format_alternatives(&same_name, base)
            ));
        }
        Ok(out)
    }

    #[tool(
        description = "List the available Archives of Nethys content categories (e.g. spell, feat, \
        creature, equipment) with how many entries each has, for the chosen `game` (\"pf2e\" default \
        or \"sf2e\"). Use these values for the `category` filter on `search` and `get`.",
        annotations(read_only_hint = true, open_world_hint = true)
    )]
    async fn list_categories(
        &self,
        Parameters(params): Parameters<GameParams>,
    ) -> Result<CallToolResult, ErrorData> {
        respond(self.run_list_categories(params).await)
    }

    async fn run_list_categories(&self, params: GameParams) -> anyhow::Result<String> {
        let game = self.game(params.game.as_deref())?;
        let cats = game
            .categories()
            .await
            .context("could not fetch categories from Archives of Nethys")?;
        let mut out = format!(
            "{} categories on {} (name: entry count):\n",
            cats.len(),
            game.client.system.label()
        );
        for (name, count) in cats {
            out.push_str(&format!("\n- {name}: {count}"));
        }
        Ok(out)
    }
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
             `list_categories` to discover valid categories. Results follow the Remaster: an \
             entry it replaced shows as its remastered version unless `legacy` is true. Common \
             categories: {}.",
            common_categories_hint()
        ))
    }
}
