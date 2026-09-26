//! `wf cache`.

use anyhow::Result;
use colored::Colorize;

use super::Ctx;
use crate::cli::CacheAction;

pub fn run(ctx: &Ctx, action: CacheAction) -> Result<()> {
    let Some(cache) = ctx.wf.cache() else {
        println!("  {} Caching is off (WAYFINDER_CACHE).", "ℹ".blue());
        return Ok(());
    };
    match action {
        CacheAction::Purge => {
            let n = cache.purge_expired()?;
            println!(
                "  {} Purged {} expired response(s).",
                "✓".green(),
                n.to_string().bold()
            );
        }
        CacheAction::Clear => {
            let n = cache.clear()?;
            println!(
                "  {} Cleared {} response(s).",
                "✓".green(),
                n.to_string().bold()
            );
        }
        CacheAction::Status => {
            let status = cache.status()?;
            println!("Cache: {}", cache.path().display().to_string().dimmed());
            if status.is_empty() {
                println!("  {} Cache is empty.", "ℹ".blue());
            }
            for (game, count) in &status {
                println!(
                    "  {}: {} fresh response(s)",
                    game.bold(),
                    count.to_string().cyan()
                );
            }
        }
    }
    Ok(())
}
