mod cli;
mod commands;
mod creators;
mod generate_gitignore;
mod initializers;
mod patches;
mod render;
mod stm32cubemx;
mod utils;

use crate::cli::Cli;
use clap::Parser;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    commands::run(cli.command)
}
