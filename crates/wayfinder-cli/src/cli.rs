//! Command-line definition: the clap structs and argument parsers.

use clap::{Parser, Subcommand, ValueEnum};

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

#[derive(Subcommand, Clone)]
pub enum CacheAction {
    /// Show cache status
    Status,
    /// Remove expired entries from the cache
    Purge,
}

pub const MAX_INPUT_LEN: usize = 500;
pub const MAX_FILTERS: usize = 20;
// Hard cap on a single result set. AON's own UI pages in blocks of 50; keeping
// the ceiling low nudges toward narrowed, filtered queries instead of bulk pulls.
pub const MAX_RESULT_LIMIT: u32 = 100;

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

#[cfg(test)]
mod tests {
    use super::{MAX_INPUT_LEN, parse_filter};

    #[test]
    fn parse_filter_splits_on_equals() {
        assert_eq!(
            parse_filter("domain=Dragon").unwrap(),
            ("domain".into(), "Dragon".into())
        );
        assert!(parse_filter("no-equals-sign").is_err());
    }

    #[test]
    fn parse_filter_rejects_overlong_parts() {
        let long = "x".repeat(MAX_INPUT_LEN + 1);
        assert!(parse_filter(&format!("{long}=v")).is_err());
        assert!(parse_filter(&format!("k={long}")).is_err());
    }
}
