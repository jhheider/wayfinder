//! The MCP server: tool definitions and the `ServerHandler` implementation.
//!
//! Everything about talking to AON (requests, caching, remaster handling,
//! category resolution, picking among same-named entries) is
//! `wayfinder_core::Wayfinder`, shared with the `wf` CLI. This crate maps
//! tool parameters onto it and lays results out for a model.

use std::sync::Arc;

use anyhow::{Context, bail};
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerInfo};
use rmcp::{ErrorData, ServerHandler, tool, tool_handler, tool_router};

use wayfinder_core::Wayfinder;
use wayfinder_core::aon::{AonClient, CategoryError, GameSystem};
use wayfinder_core::cache::ResponseCache;

use crate::format::{format_alternatives, format_detail, format_summary};
use crate::params::{GameParams, GetParams, SearchParams, common_categories_hint, game_system};

/// Pathfinder 2e / Starfinder 2e MCP server backed by Archives of Nethys.
#[derive(Clone)]
pub struct WayfinderServer {
    pf2e: Arc<Wayfinder>,
    sf2e: Arc<Wayfinder>,
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
    /// Construct the server: a service per game, sharing one response cache
    /// (and the file the `wf` CLI uses, unless `WAYFINDER_CACHE` says otherwise).
    pub fn new() -> anyhow::Result<Self> {
        let cache = match ResponseCache::from_env() {
            Ok(cache) => cache.map(Arc::new),
            Err(e) => {
                tracing::warn!("response cache unavailable, continuing without: {e}");
                None
            }
        };
        let game = |system| -> anyhow::Result<Arc<Wayfinder>> {
            Ok(Arc::new(Wayfinder::new(
                AonClient::from_env(system)?,
                cache.clone(),
            )))
        };
        Ok(Self {
            pf2e: game(GameSystem::Pathfinder)?,
            sf2e: game(GameSystem::Starfinder)?,
            tool_router: Self::tool_router(),
        })
    }

    /// Resolve the requested game (defaults to Pathfinder 2e).
    fn game(&self, game: Option<&str>) -> anyhow::Result<&Wayfinder> {
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

    async fn run_search(&self, params: SearchParams) -> anyhow::Result<String> {
        let wf = self.game(params.game.as_deref())?;
        let category = resolve_category(wf, params.category.as_deref()).await?;
        let page = wf.search(&params.to_search(category)).await?;
        if page.docs.is_empty() {
            return Ok("No results found. Try a broader query or fewer filters.".to_string());
        }

        let first = page.offset as usize + 1;
        let last = page.offset as usize + page.docs.len();
        let mut out = format!("Found {} match(es); showing {first}-{last}", page.total);
        if let Some(next) = page.next_offset() {
            out.push_str(&format!(" (pass offset={next} for more)"));
        }
        out.push_str(":\n");
        let base = wf.system().base_url();
        for (i, doc) in page.docs.iter().enumerate() {
            out.push('\n');
            out.push_str(&format_summary(first + i, doc, base));
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

    async fn run_get(&self, params: GetParams) -> anyhow::Result<String> {
        let wf = self.game(params.game.as_deref())?;
        let category = resolve_category(wf, params.category.as_deref()).await?;
        let Some(picked) = wf.lookup(&params.to_lookup(category)).await? else {
            return Ok(
                "No matching entry found. Check the spelling, try the `search` tool, \
                       or drop `category`."
                    .to_string(),
            );
        };

        let base = wf.system().base_url();
        let mut out = format_detail(&picked.best, base);
        if !picked.same_name.is_empty() {
            out.push_str(&format!(
                "\n---\nOther entries named \"{}\" (pass `category` or `url` to get one):\n{}",
                params.name.as_deref().unwrap_or_default().trim(),
                format_alternatives(&picked.same_name, base)
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
        let wf = self.game(params.game.as_deref())?;
        let cats = wf
            .categories()
            .await
            .context("could not fetch categories from Archives of Nethys")?;
        let mut out = format!(
            "{} categories on {} (name: entry count):\n",
            cats.len(),
            wf.system().label()
        );
        for (name, count) in cats {
            out.push_str(&format!("\n- {name}: {count}"));
        }
        Ok(out)
    }
}

/// Resolve an optional `category` argument. A near miss is an error naming
/// the suggestion rather than a silent substitution: the model should know
/// what it actually searched.
async fn resolve_category(wf: &Wayfinder, input: Option<&str>) -> anyhow::Result<Option<String>> {
    let Some(input) = input.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    let label = wf.system().label();
    match wf.resolve_category(input).await {
        Ok(cat) => Ok(Some(cat)),
        Err(CategoryError::Suggested { input, suggestion }) => bail!(
            "unknown {label} category {input:?}; did you mean {suggestion:?}? \
             `list_categories` lists them all"
        ),
        Err(CategoryError::Unknown(input)) => {
            bail!("unknown {label} category {input:?}; `list_categories` lists them all")
        }
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
