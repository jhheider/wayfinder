//! Command-line definition: the clap structs and argument parsers.

use clap::{Parser, Subcommand, ValueEnum};
use wayfinder_core::aon::Sort;

#[derive(Parser)]
#[command(
    name = "wf",
    version,
    about = "⚔️  Search and browse Pathfinder 2e / Starfinder 2e data"
)]
pub struct Cli {
    /// Use Starfinder 2e data instead of Pathfinder 2e
    #[arg(long, global = true)]
    pub sf2e: bool,
    /// Prefer legacy (pre-remaster) versions of documents
    #[arg(long, global = true)]
    pub legacy: bool,
    /// Output format
    #[arg(long, global = true, default_value = "pretty")]
    pub format: OutputFormat,
    /// Print shell completions to stdout (for packagers; bash|zsh|fish|...).
    #[arg(long, value_name = "SHELL", hide = true)]
    pub completions: Option<clap_complete::Shell>,
    /// Print the man page (roff) to stdout (for packagers).
    #[arg(long, hide = true)]
    pub manpage: bool,
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    /// Colorized terminal output (default)
    Pretty,
    /// Raw JSON
    Json,
    /// Raw AON markdown (no color processing)
    Md,
}

#[derive(Subcommand, Clone)]
pub enum Command {
    /// Search AON by category and filters
    Search(Box<SearchArgs>),
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

#[derive(Subcommand, Clone)]
pub enum CacheAction {
    /// Show cache status
    Status,
    /// Remove expired entries from the cache
    Purge,
    /// Remove every entry from the cache
    Clear,
}

#[derive(clap::Args, Clone)]
pub struct SearchArgs {
    /// Search term: "sarenrae" (broad) or "deity/sarenrae" (scoped; "spell/"
    /// alone lists a category)
    pub term: String,
    /// Phrase the name must contain
    #[arg(long)]
    pub name: Option<String>,
    /// Free text (names, summaries, rules text)
    #[arg(long)]
    pub text: Option<String>,
    /// Required trait (repeatable)
    #[arg(long = "trait", short = 't', value_name = "TRAIT")]
    pub traits: Vec<String>,
    /// Generic field filter: field=value (repeatable)
    #[arg(long = "filter", short = 'f', value_parser = parse_filter)]
    pub filters: Vec<(String, String)>,
    /// Exact level/rank
    #[arg(long, conflicts_with_all = ["min_level", "max_level"])]
    pub level: Option<i64>,
    /// Lowest level/rank
    #[arg(long)]
    pub min_level: Option<i64>,
    /// Highest level/rank
    #[arg(long)]
    pub max_level: Option<i64>,
    /// Source book, matched as a phrase ("Player Core")
    #[arg(long)]
    pub source: Option<String>,
    /// Rarity: common, uncommon, rare, unique
    #[arg(long)]
    pub rarity: Option<String>,
    /// Result order
    #[arg(long, value_enum, default_value = "relevance")]
    pub sort: SortArg,
    /// Maximum number of results
    #[arg(long, default_value = "50")]
    pub limit: u32,
    /// Results to skip (for paging)
    #[arg(long, default_value = "0")]
    pub offset: u32,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum SortArg {
    /// Best match first
    Relevance,
    /// Lowest level first
    Level,
    /// Alphabetical
    Name,
}

impl From<SortArg> for Sort {
    fn from(s: SortArg) -> Self {
        match s {
            SortArg::Relevance => Sort::Relevance,
            SortArg::Level => Sort::Level,
            SortArg::Name => Sort::Name,
        }
    }
}

fn parse_filter(s: &str) -> Result<(String, String), String> {
    let (k, v) = s
        .split_once('=')
        .ok_or_else(|| format!("expected field=value, got '{s}'"))?;
    Ok((k.to_string(), v.to_string()))
}

#[cfg(test)]
mod tests {
    use super::parse_filter;

    #[test]
    fn parse_filter_splits_on_equals() {
        assert_eq!(
            parse_filter("domain=Dragon").unwrap(),
            ("domain".into(), "Dragon".into())
        );
        assert!(parse_filter("no-equals-sign").is_err());
    }
}
