//! `wf show`.

use anyhow::{Result, bail};
use colored::Colorize;
use serde::Serialize;

use wayfinder_core::aon::query::MAX_INPUT_LEN;
use wayfinder_core::aon::{Document, Lookup};
use wayfinder_core::render::{ContentBlock, render_markdown, render_spans, render_spans_colored};

use super::Ctx;
use crate::cli::OutputFormat;
use crate::resolve::{cli_resolve_category, resolve_show_query};

/// Another entry sharing the shown entry's name.
#[derive(Serialize)]
struct Also<'a> {
    name: Option<&'a str>,
    category: Option<&'a str>,
    url: Option<String>,
}

pub async fn run(ctx: &Ctx, query: &[String]) -> Result<()> {
    if query.is_empty() {
        bail!("show requires a query (e.g. 'spell/fireball' or 'fireball')");
    }
    if query.join(" ").len() > MAX_INPUT_LEN {
        bail!("Show query exceeds maximum length of {MAX_INPUT_LEN} characters.");
    }
    let (cat, name) = resolve_show_query(query);
    let category = match cat {
        Some(c) => Some(cli_resolve_category(&ctx.wf, &c).await?),
        None => None,
    };
    let name = name.ok_or_else(|| anyhow::anyhow!("show requires a name"))?;
    let lookup = Lookup {
        edition: ctx.edition,
        ..Lookup::named(&name, category.as_deref())
    };
    let Some(picked) = ctx.wf.lookup(&lookup).await? else {
        println!("  {} Not found: {}", "✗".red(), name.yellow());
        return Ok(());
    };

    let base = ctx.wf.system().base_url();
    let doc = &picked.best;
    let blocks = doc.content(base);
    let also: Vec<Also> = picked
        .same_name
        .iter()
        .map(|d| Also {
            name: d.name.as_deref(),
            category: d.category.as_deref(),
            url: d.absolute_url(base),
        })
        .collect();

    match ctx.format {
        OutputFormat::Json => {
            #[derive(Serialize)]
            struct JsonDoc<'a> {
                #[serde(flatten)]
                doc: &'a Document,
                content: &'a [ContentBlock],
                #[serde(skip_serializing_if = "<[_]>::is_empty")]
                also: &'a [Also<'a>],
            }
            let out = JsonDoc {
                doc,
                content: &blocks,
                also: &also,
            };
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
        OutputFormat::Md => println!("{}", render_markdown(&blocks)),
        OutputFormat::Pretty => print!("{}", render_spans_colored(&render_spans(&blocks))),
    }
    // A note, not output: keep stdout clean for piping.
    if !also.is_empty() && !matches!(ctx.format, OutputFormat::Json) {
        let cats: Vec<&str> = also.iter().filter_map(|a| a.category).collect();
        eprintln!(
            "\n{} Also named \"{}\": {} ({})",
            "ℹ".blue(),
            name,
            cats.join(", "),
            "wf show <category> <name>".cyan()
        );
    }
    Ok(())
}
