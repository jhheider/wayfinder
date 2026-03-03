use anyhow::{Result, bail};
use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use std::path::PathBuf;

use wayfinder_core::aon::categories::{CATEGORY_GROUPS, category_icon, filterable_fields};
use wayfinder_core::aon::client::GameSystem;
use wayfinder_core::aon::parse::{CategoryError, parse_compound, resolve_category};
use wayfinder_core::aon::query::{is_valid_filter_field, is_valid_filter_for_category};
use wayfinder_core::aon::{AonClient, SearchQuery};
use wayfinder_core::render::{
    ContentBlock, display_short_colored, parse_content, render_markdown, render_spans,
    render_spans_colored,
};
use wayfinder_core::search::{
    SearchService, filter_legacy_duplicates, group_broad_results, is_legacy,
};

#[derive(Parser)]
#[command(
    name = "wf",
    about = "⚔️  Search and browse Pathfinder 2e / Starfinder 2e data"
)]
struct Cli {
    /// Use Starfinder 2e data instead of Pathfinder 2e
    #[arg(long, global = true)]
    sf2e: bool,
    /// Prefer legacy (pre-remaster) versions of documents
    #[arg(long, global = true)]
    legacy: bool,
    /// Output format
    #[arg(long, global = true, default_value = "pretty")]
    format: OutputFormat,
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Copy, ValueEnum)]
enum OutputFormat {
    /// Colorized terminal output (default)
    Pretty,
    /// Raw JSON
    Json,
    /// Raw AON markdown (no color processing)
    Md,
}

#[derive(Subcommand)]
enum Command {
    /// Search AON by category and filters
    Search {
        /// Search term: "sarenrae" (broad) or "deity/sarenrae" (scoped)
        term: String,
        /// Filter by name (additional)
        #[arg(long)]
        name: Option<String>,
        /// Full-text search
        #[arg(long)]
        text: Option<String>,
        /// Generic field filter: field=value (repeatable)
        #[arg(long = "filter", short = 'f', value_parser = parse_filter)]
        filters: Vec<(String, String)>,
        /// Filter by level
        #[arg(long)]
        level: Option<i32>,
        /// Maximum number of results
        #[arg(long, default_value = "50")]
        limit: u32,
    },
    /// Show a specific document by name
    Show {
        /// Query: "deity/sarenrae" or "deity sarenrae"
        query: Vec<String>,
    },
    /// List all categories (grouped)
    Categories,
    /// Show filterable fields for a category
    Fields {
        /// Category to inspect
        category: String,
    },
    /// Cache management
    Cache {
        #[command(subcommand)]
        action: CacheAction,
    },
}

#[derive(Subcommand)]
enum CacheAction {
    /// Fetch an entire category into the local cache
    Fetch { category: String },
    /// Show cache status
    Status,
    /// Remove expired entries from the cache
    Purge,
}

const MAX_INPUT_LEN: usize = 500;
const MAX_FILTERS: usize = 20;
const MAX_RESULT_LIMIT: u32 = 500;

fn parse_filter(s: &str) -> Result<(String, String), String> {
    let (k, v) = s
        .split_once('=')
        .ok_or_else(|| format!("expected field=value, got '{s}'"))?;
    if k.len() > MAX_INPUT_LEN {
        return Err(format!(
            "filter field name exceeds maximum length of {MAX_INPUT_LEN} characters"
        ));
    }
    if v.len() > MAX_INPUT_LEN {
        return Err(format!(
            "filter value exceeds maximum length of {MAX_INPUT_LEN} characters"
        ));
    }
    Ok((k.to_string(), v.to_string()))
}

/// Resolve a category string, printing warnings/errors with color.
fn cli_resolve_category(input: &str) -> Result<String> {
    match resolve_category(input) {
        Ok(cat) => Ok(cat),
        Err(CategoryError::Suggested { input, suggestion }) => {
            eprintln!(
                "{} Unknown category '{}', using '{}'",
                "⚠".yellow(),
                input.red(),
                suggestion.green()
            );
            Ok(suggestion)
        }
        Err(CategoryError::Unknown(input)) => {
            bail!(
                "Unknown category '{}'. Run {} to see all available categories.",
                input.red(),
                "wf categories".cyan()
            );
        }
    }
}

fn cache_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("wayfinder")
        .join("wayfinder_cache.db")
}

fn game_system(cli: &Cli) -> GameSystem {
    if cli.sf2e {
        GameSystem::Starfinder
    } else {
        GameSystem::Pathfinder
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let system = game_system(&cli);
    let cache = cache_path();
    if let Some(parent) = cache.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let client = AonClient::new(system)?;
    let svc = SearchService::new(client, &cache);

    let sys_label = if cli.sf2e {
        "🚀 SF2e".cyan().bold().to_string()
    } else {
        "⚔️  PF2e".red().bold().to_string()
    };

    match cli.command {
        Command::Categories => {
            println!("{} Categories:\n", sys_label);
            for group in CATEGORY_GROUPS {
                println!("{}:", group.name.bold().underline());
                for &cat in group.members {
                    let icon = category_icon(cat);
                    let filters = match filterable_fields(cat) {
                        Some(f) => format!(" {}", format!("({} filters)", f.len()).dimmed()),
                        None => String::new(),
                    };
                    println!("  {icon} {cat}{filters}");
                }
                println!();
            }
        }
        Command::Fields { category } => {
            if category.len() > MAX_INPUT_LEN {
                bail!("Category name exceeds maximum length of {MAX_INPUT_LEN} characters.");
            }
            let category = cli_resolve_category(&category)?;
            println!("{} Fields for {}:\n", sys_label, category.bold().green());
            match filterable_fields(&category) {
                Some(fields) => {
                    for &f in fields {
                        println!("  {} {f}", "•".dimmed());
                    }
                    println!(
                        "\n{}",
                        format!("Usage: wf search {category} -f field=value [-f field2=value2]")
                            .dimmed()
                    );
                }
                None => {
                    println!(
                        "  {} No field info available. Common filters (rarity, source) may still work.",
                        "ℹ".blue()
                    );
                }
            }
        }
        Command::Search {
            term,
            name,
            text,
            filters,
            level,
            limit,
        } => {
            if limit > MAX_RESULT_LIMIT {
                bail!("Result limit cannot exceed {MAX_RESULT_LIMIT}.");
            }
            if term.len() > MAX_INPUT_LEN {
                bail!("Search term exceeds maximum length of {MAX_INPUT_LEN} characters.");
            }
            if let Some(n) = &name
                && n.len() > MAX_INPUT_LEN
            {
                bail!("--name value exceeds maximum length of {MAX_INPUT_LEN} characters.");
            }
            if let Some(t) = &text
                && t.len() > MAX_INPUT_LEN
            {
                bail!("--text value exceeds maximum length of {MAX_INPUT_LEN} characters.");
            }
            if filters.len() > MAX_FILTERS {
                bail!("Too many filters (maximum {MAX_FILTERS}).");
            }
            for (field, _) in &filters {
                if !is_valid_filter_field(field) {
                    bail!(
                        "Unknown filter field '{}'. Run {} to see valid fields for a category.",
                        field.red(),
                        "wf fields <category>".cyan()
                    );
                }
            }

            let (category, search_name) = parse_compound(&term);

            // Minimum query length for broad (unscoped) searches
            if category.is_none()
                && let Some(n) = &search_name
                && n.len() < 3
            {
                bail!(
                    "Search query '{}' is too short (minimum 3 characters for broad searches). \
                     Use category/name syntax (e.g. 'deity/{}') for short queries.",
                    n.red(),
                    n
                );
            }

            let mut q = SearchQuery::new().size(limit);
            if let Some(cat) = &category {
                let cat = cli_resolve_category(cat)?;
                // Category-aware filter validation
                for (field, _) in &filters {
                    if !is_valid_filter_for_category(field, &cat) {
                        bail!(
                            "Field '{}' is not a valid filter for category '{}'. Run {} to see valid fields.",
                            field.red(),
                            cat.green(),
                            format!("wf fields {cat}").cyan()
                        );
                    }
                }
                q = q.category(&cat);
                // Category-scoped: search by name
                if let Some(n) = &search_name {
                    q = q.name(n);
                }
            } else if let Some(n) = &search_name {
                // Broad search: match across name and text fields
                q = q.broad(n);
            }
            if let Some(n) = &name {
                q = q.name(n);
            }
            if let Some(t) = &text {
                q = q.text(t);
            }
            for (field, value) in &filters {
                q = q.filter(field, value);
            }
            if let Some(l) = level {
                q = q.filter("level", &l.to_string());
            }

            let mut results = svc.search(&q).await?;

            results = filter_legacy_duplicates(results, cli.legacy);

            // For broad searches, group: exact name matches first (in ES order),
            // then remaining results grouped by category (first-appearance order),
            // preserving ES order within each category group.
            if category.is_none()
                && let Some(n) = &search_name
            {
                results = group_broad_results(results, n);
            }

            match cli.format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&results)?);
                }
                OutputFormat::Md => {
                    for doc in &results {
                        println!("{}", doc.name.as_deref().unwrap_or("Unknown"));
                    }
                }
                OutputFormat::Pretty => {
                    if results.is_empty() {
                        println!("  {} No results found.", "✗".red());
                    } else {
                        for doc in &results {
                            println!("{}", display_short_colored(doc));
                        }
                        println!("\n{}", format!("{} result(s)", results.len()).dimmed());
                    }
                }
            }
        }
        Command::Show { query } => {
            if query.is_empty() {
                bail!("show requires a query (e.g. 'spell/fireball' or 'fireball')");
            }
            let joined = query.join(" ");
            if joined.len() > MAX_INPUT_LEN {
                bail!("Show query exceeds maximum length of {MAX_INPUT_LEN} characters.");
            }
            let (cat, name) = parse_compound(&joined);
            let category = cat.map(|c| cli_resolve_category(&c)).transpose()?;
            let show_name = name.ok_or_else(|| anyhow::anyhow!("show requires a name"))?;
            let mut results = svc.show(&show_name, category.as_deref()).await?;
            results = filter_legacy_duplicates(results, cli.legacy);
            // Sort: prefer remaster (non-legacy) unless --legacy
            results.sort_by_key(|d| if is_legacy(d) == cli.legacy { 0 } else { 1 });

            match results.first() {
                Some(doc) => {
                    let raw = doc
                        .markdown
                        .as_deref()
                        .or(doc.text.as_deref())
                        .unwrap_or("");
                    let blocks = parse_content(raw, system.base_url());

                    match cli.format {
                        OutputFormat::Json => {
                            #[derive(serde::Serialize)]
                            struct JsonDoc<'a> {
                                #[serde(flatten)]
                                doc: &'a wayfinder_core::aon::Document,
                                content: &'a [ContentBlock],
                            }
                            let out = JsonDoc {
                                doc,
                                content: &blocks,
                            };
                            println!("{}", serde_json::to_string_pretty(&out)?);
                        }
                        OutputFormat::Md => {
                            println!("{}", render_markdown(&blocks));
                        }
                        OutputFormat::Pretty => {
                            let spans = render_spans(&blocks);
                            print!("{}", render_spans_colored(&spans));
                        }
                    }
                }
                None => {
                    println!("  {} Not found: {}", "✗".red(), show_name.yellow());
                }
            }
        }
        Command::Cache { action } => match action {
            CacheAction::Fetch { category } => {
                if category.len() > MAX_INPUT_LEN {
                    bail!("Category name exceeds maximum length of {MAX_INPUT_LEN} characters.");
                }
                let category = cli_resolve_category(&category)?;
                println!("  {} Fetching {}...", "↓".cyan(), category.bold());
                let count = svc.fetch_category(&category).await?;
                println!(
                    "  {} Cached {} {} documents.",
                    "✓".green(),
                    count.to_string().bold(),
                    category
                );
            }
            CacheAction::Purge => {
                let deleted = svc.purge_expired()?;
                println!(
                    "  {} Purged {} expired document(s).",
                    "✓".green(),
                    deleted.to_string().bold()
                );
            }
            CacheAction::Status => {
                let status = svc.cache_status()?;
                if status.is_empty() {
                    println!("  {} Cache is empty.", "ℹ".blue());
                } else {
                    println!("{} Cache status:\n", sys_label);
                    for (cat, count) in &status {
                        let icon = category_icon(cat);
                        println!("  {icon} {}: {}", cat.bold(), count.to_string().cyan());
                    }
                }
            }
        },
    }

    Ok(())
}
