mod cli;
mod commands;
mod resolve;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser};
use colored::Colorize;

use wayfinder_core::Wayfinder;
use wayfinder_core::aon::{AonClient, Edition, GameSystem};
use wayfinder_core::cache::ResponseCache;

use crate::cli::{Cli, Command};
use crate::commands::{Ctx, cache, categories, search, show};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Packager outputs (hidden flags): the binary is its own doc generator, so
    // release archives and package builds need no extra tooling. Handled before
    // any network/cache setup so they work in a sandbox.
    if let Some(shell) = cli.completions {
        clap_complete::generate(shell, &mut Cli::command(), "wf", &mut std::io::stdout());
        return Ok(());
    }
    if cli.manpage {
        clap_mangen::Man::new(Cli::command())
            .render(&mut std::io::stdout())
            .context("rendering man page")?;
        return Ok(());
    }

    let Some(command) = cli.command.clone() else {
        Cli::command().print_help()?;
        return Ok(());
    };

    let system = if cli.sf2e {
        GameSystem::Starfinder
    } else {
        GameSystem::Pathfinder
    };
    // The cache is an optimization: if it cannot open, say so and carry on.
    let cache = ResponseCache::from_env().unwrap_or_else(|e| {
        eprintln!("{} response cache unavailable: {e}", "⚠".yellow());
        None
    });
    let ctx = Ctx {
        wf: Wayfinder::new(AonClient::from_env(system)?, cache.map(Into::into)),
        format: cli.format,
        edition: Edition::legacy_if(cli.legacy),
        sys_label: if cli.sf2e {
            "🚀 SF2e".cyan().bold().to_string()
        } else {
            "⚔️  PF2e".red().bold().to_string()
        },
    };

    match command {
        Command::Categories => categories::categories(&ctx).await?,
        Command::Fields { category } => categories::fields(&ctx, &category).await?,
        Command::Search(args) => search::run(&ctx, *args).await?,
        Command::Show { query } => show::run(&ctx, &query).await?,
        Command::Cache { action } => cache::run(&ctx, action)?,
    }

    Ok(())
}
