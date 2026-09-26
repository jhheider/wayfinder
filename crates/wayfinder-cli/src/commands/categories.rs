//! `wf categories` and `wf fields`.

use std::collections::HashMap;

use anyhow::Result;
use colored::Colorize;

use wayfinder_core::aon::categories::{CATEGORY_GROUPS, category_icon, filterable_fields};

use super::Ctx;
use crate::cli::OutputFormat;
use crate::resolve::cli_resolve_category;

/// List the game's live categories with entry counts, in the built-in groups
/// (anything new to AON lands under "Other"). Offline, list the built-in
/// groups without counts.
pub async fn categories(ctx: &Ctx) -> Result<()> {
    let live = ctx.wf.categories().await.ok();
    if let (OutputFormat::Json, Some(cats)) = (ctx.format, live) {
        let map: serde_json::Map<_, _> = cats
            .iter()
            .map(|(name, count)| (name.clone(), (*count).into()))
            .collect();
        println!("{}", serde_json::to_string_pretty(&map)?);
        return Ok(());
    }

    let counts: Option<HashMap<&str, i64>> =
        live.map(|cats| cats.iter().map(|(n, c)| (n.as_str(), *c)).collect());
    println!("{} Categories:\n", ctx.sys_label);
    let print_group = |name: &str, members: &mut dyn Iterator<Item = &str>| {
        let lines: Vec<String> = members
            .filter_map(|cat| {
                let count = match &counts {
                    Some(c) => format!(" {}", c.get(cat)?.to_string().dimmed()),
                    None => String::new(),
                };
                Some(format!("  {} {cat}{count}", category_icon(cat)))
            })
            .collect();
        if !lines.is_empty() {
            println!("{}:\n{}\n", name.bold().underline(), lines.join("\n"));
        }
    };
    for group in CATEGORY_GROUPS {
        print_group(group.name, &mut group.members.iter().copied());
    }
    if let Some(counts) = &counts {
        let grouped = |c: &str| CATEGORY_GROUPS.iter().any(|g| g.members.contains(&c));
        let mut other: Vec<&str> = counts.keys().copied().filter(|c| !grouped(c)).collect();
        other.sort_unstable();
        print_group("Other", &mut other.into_iter());
    } else {
        eprintln!("{} Offline: showing built-in categories.", "ℹ".blue());
    }
    Ok(())
}

pub async fn fields(ctx: &Ctx, category: &str) -> Result<()> {
    let category = cli_resolve_category(&ctx.wf, category).await?;
    println!(
        "{} Fields for {}:\n",
        ctx.sys_label,
        category.bold().green()
    );
    match filterable_fields(&category) {
        Some(fields) => {
            for &f in fields {
                println!("  {} {f}", "•".dimmed());
            }
            println!(
                "\n{}",
                format!("Usage: wf search {category}/ -f field=value [-f field2=value2]").dimmed()
            );
        }
        None => {
            println!(
                "  {} No field info available. Common filters (rarity, source) may still work.",
                "ℹ".blue()
            );
        }
    }
    Ok(())
}
