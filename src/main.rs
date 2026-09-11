mod cli;
mod commands;
mod configs;
mod cpkg;
mod creators;
mod gitignore;
mod initializers;
mod patches;
mod render;
mod stm32cubemx;
mod templates;
mod update;
mod utils;

use crate::cli::Cli;
use clap::Parser;
use tracing::{Level, debug};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose);
    debug!(?cli, "Parsed CLI arguments");

    commands::run(cli.command)
}

fn init_tracing(verbose: bool) {
    let max_level = if verbose { Level::DEBUG } else { Level::INFO };

    tracing_subscriber::fmt().with_max_level(max_level).init();
}
