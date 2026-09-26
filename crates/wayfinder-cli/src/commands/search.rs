//! `wf search`.

use anyhow::{Result, bail};
use colored::Colorize;

use wayfinder_core::aon::SearchQuery;
use wayfinder_core::aon::parse::parse_compound;
use wayfinder_core::aon::query::{is_valid_filter_field, is_valid_filter_for_category};
use wayfinder_core::render::display_short_colored;
use wayfinder_core::search::{filter_legacy_duplicates, group_broad_results};

use super::Ctx;
use crate::cli::{MAX_FILTERS, MAX_INPUT_LEN, MAX_RESULT_LIMIT, OutputFormat};
use crate::resolve::cli_resolve_category;

/// The `search` subcommand's arguments.
pub struct Args {
    pub term: String,
    pub name: Option<String>,
    pub text: Option<String>,
    pub filters: Vec<(String, String)>,
    pub level: Option<i32>,
    pub limit: u32,
}

pub async fn run(ctx: &Ctx, args: Args) -> Result<()> {
    let Args {
        term,
        name,
        text,
        filters,
        level,
        limit,
    } = args;
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

    let mut results = ctx.svc.search(&q).await?;

    results = filter_legacy_duplicates(results, ctx.legacy);

    // For broad searches, group: exact name matches first (in ES order),
    // then remaining results grouped by category (first-appearance order),
    // preserving ES order within each category group.
    if category.is_none()
        && let Some(n) = &search_name
    {
        results = group_broad_results(results, n);
    }

    match ctx.format {
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
    Ok(())
}
