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
mod utils;

use crate::cli::Cli;
use clap::Parser;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    commands::run(cli.command)
}
