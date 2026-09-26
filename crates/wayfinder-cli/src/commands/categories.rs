//! `wf categories` and `wf fields`.

use anyhow::{Result, bail};
use colored::Colorize;

use wayfinder_core::aon::categories::{CATEGORY_GROUPS, category_icon, filterable_fields};

use super::Ctx;
use crate::cli::MAX_INPUT_LEN;
use crate::resolve::cli_resolve_category;

pub fn categories(ctx: &Ctx) {
    println!("{} Categories:\n", ctx.sys_label);
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

pub fn fields(ctx: &Ctx, category: &str) -> Result<()> {
    if category.len() > MAX_INPUT_LEN {
        bail!("Category name exceeds maximum length of {MAX_INPUT_LEN} characters.");
    }
    let category = cli_resolve_category(category)?;
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
                format!("Usage: wf search {category} -f field=value [-f field2=value2]").dimmed()
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
