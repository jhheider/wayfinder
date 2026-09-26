mod cli;
mod commands;
mod resolve;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser};
use colored::Colorize;

use wayfinder_core::aon::AonClient;
use wayfinder_core::search::SearchService;

use crate::cli::{Cli, Command};
use crate::commands::{Ctx, cache, categories, search, show};
use crate::resolve::{cache_path, game_system};

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

    let system = game_system(&cli);
    let cache = cache_path();
    if let Some(parent) = cache.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let client = AonClient::from_env(system)?;
    let ctx = Ctx {
        svc: SearchService::new(client, &cache),
        system,
        format: cli.format,
        legacy: cli.legacy,
        sys_label: if cli.sf2e {
            "🚀 SF2e".cyan().bold().to_string()
        } else {
            "⚔️  PF2e".red().bold().to_string()
        },
    };

    match command {
        Command::Categories => categories::categories(&ctx),
        Command::Fields { category } => categories::fields(&ctx, &category)?,
        Command::Search {
            term,
            name,
            text,
            filters,
            level,
            limit,
        } => {
            let args = search::Args {
                term,
                name,
                text,
                filters,
                level,
                limit,
            };
            search::run(&ctx, args).await?
        }
        Command::Show { query } => show::run(&ctx, &query).await?,
        Command::Cache { action } => cache::run(&ctx, action)?,
    }

    Ok(())
}
