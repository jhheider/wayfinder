//! `wf show`.

use anyhow::{Result, bail};
use colored::Colorize;

use wayfinder_core::render::{
    ContentBlock, parse_content, render_markdown, render_spans, render_spans_colored,
};
use wayfinder_core::search::{filter_legacy_duplicates, is_legacy};

use super::Ctx;
use crate::cli::{MAX_INPUT_LEN, OutputFormat};
use crate::resolve::{cli_resolve_category, resolve_show_query};

pub async fn run(ctx: &Ctx, query: &[String]) -> Result<()> {
    if query.is_empty() {
        bail!("show requires a query (e.g. 'spell/fireball' or 'fireball')");
    }
    if query.join(" ").len() > MAX_INPUT_LEN {
        bail!("Show query exceeds maximum length of {MAX_INPUT_LEN} characters.");
    }
    let (cat, name) = resolve_show_query(query);
    let category = cat.map(|c| cli_resolve_category(&c)).transpose()?;
    let show_name = name.ok_or_else(|| anyhow::anyhow!("show requires a name"))?;
    let mut results = ctx.svc.show(&show_name, category.as_deref()).await?;
    results = filter_legacy_duplicates(results, ctx.legacy);
    // Sort: prefer remaster (non-legacy) unless --legacy
    results.sort_by_key(|d| if is_legacy(d) == ctx.legacy { 0 } else { 1 });

    match results.first() {
        Some(doc) => {
            let raw = doc
                .markdown
                .as_deref()
                .or(doc.text.as_deref())
                .unwrap_or("");
            let blocks = parse_content(raw, ctx.system.base_url());

            match ctx.format {
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
    Ok(())
}
