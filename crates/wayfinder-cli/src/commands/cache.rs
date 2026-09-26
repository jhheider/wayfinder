//! `wf cache`.

use anyhow::Result;
use colored::Colorize;

use wayfinder_core::aon::categories::category_icon;

use super::Ctx;
use crate::cli::CacheAction;

pub fn run(ctx: &Ctx, action: CacheAction) -> Result<()> {
    match action {
        CacheAction::Purge => {
            let deleted = ctx.svc.purge_expired()?;
            println!(
                "  {} Purged {} expired document(s).",
                "✓".green(),
                deleted.to_string().bold()
            );
        }
        CacheAction::Status => {
            let status = ctx.svc.cache_status()?;
            if status.is_empty() {
                println!("  {} Cache is empty.", "ℹ".blue());
            } else {
                println!("{} Cache status:\n", ctx.sys_label);
                for (cat, count) in &status {
                    let icon = category_icon(cat);
                    println!("  {icon} {}: {}", cat.bold(), count.to_string().cyan());
                }
            }
        }
    }
    Ok(())
}
