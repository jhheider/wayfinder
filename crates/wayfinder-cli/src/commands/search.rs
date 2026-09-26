//! `wf search`.

use anyhow::{Result, bail};
use colored::Colorize;

use wayfinder_core::aon::Search;
use wayfinder_core::aon::categories::{is_valid_filter_field, is_valid_filter_for_category};
use wayfinder_core::aon::parse::parse_compound;
use wayfinder_core::aon::query::MAX_LIMIT;
use wayfinder_core::render::display_short_colored;
use wayfinder_core::service::group_broad_results;

use super::Ctx;
use crate::cli::{OutputFormat, SearchArgs};
use crate::resolve::cli_resolve_category;

/// Broad (uncategorized) terms shorter than this match too much to be useful.
const MIN_BROAD_TERM: usize = 3;

pub async fn run(ctx: &Ctx, args: SearchArgs) -> Result<()> {
    if args.limit > MAX_LIMIT {
        bail!("Result limit cannot exceed {MAX_LIMIT}.");
    }
    for (field, _) in &args.filters {
        if !is_valid_filter_field(field) {
            bail!(
                "Unknown filter field '{}'. Run {} to see valid fields for a category.",
                field.red(),
                "wf fields <category>".cyan()
            );
        }
    }

    let (category, term) = parse_compound(&args.term);
    let broad = category.is_none();
    let mut search = Search {
        traits: args.traits,
        fields: args.filters,
        min_level: args.level.or(args.min_level),
        max_level: args.level.or(args.max_level),
        source: args.source,
        rarity: args.rarity,
        edition: ctx.edition,
        sort: args.sort.into(),
        limit: Some(args.limit),
        offset: args.offset,
        full: matches!(ctx.format, OutputFormat::Json),
        ..Search::default()
    };

    if let Some(cat) = &category {
        let cat = cli_resolve_category(&ctx.wf, cat).await?;
        for (field, _) in &search.fields {
            if !is_valid_filter_for_category(field, &cat) {
                bail!(
                    "Field '{}' is not a valid filter for category '{}'. Run {} to see valid fields.",
                    field.red(),
                    cat.green(),
                    format!("wf fields {cat}").cyan()
                );
            }
        }
        search.category = Some(cat);
        // Scoped: the term is a name phrase.
        search.name = term.clone();
    } else if let Some(t) = &term {
        if t.len() < MIN_BROAD_TERM {
            bail!(
                "Search query '{}' is too short (minimum {MIN_BROAD_TERM} characters for broad \
                 searches). Use category/name syntax (e.g. 'deity/{}') for short queries.",
                t.red(),
                t
            );
        }
        // Broad: the term is free text.
        search.text = term.clone();
    }
    if let Some(n) = args.name {
        if search.name.is_some() {
            bail!("Give the name once: either category/name or --name.");
        }
        search.name = Some(n);
    }
    if let Some(t) = args.text {
        if search.text.is_some() {
            bail!("Give the text once: either a broad term or --text.");
        }
        search.text = Some(t);
    }

    let page = ctx.wf.search(&search).await?;
    let (offset, total, next) = (page.offset, page.total_label(), page.next_offset());
    let mut results = page.docs;
    if broad && let Some(t) = &term {
        results = group_broad_results(results, t);
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
                return Ok(());
            }
            for doc in &results {
                println!("{}", display_short_colored(doc));
            }
            let first = offset + 1;
            let last = offset as usize + results.len();
            let mut footer = format!("showing {first}-{last} of {total}");
            if let Some(next) = next {
                footer.push_str(&format!(" (--offset {next} for more)"));
            }
            println!("\n{}", footer.dimmed());
        }
    }
    Ok(())
}
