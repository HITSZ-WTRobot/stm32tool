mod create;
mod init;
mod purge;

use crate::cli::Commands;

pub fn run(command: Commands) -> anyhow::Result<()> {
    match command {
        Commands::Init(args) => init::run(args),
        Commands::Create(args) => create::run(args),
        Commands::Purge => purge::run(),
        Commands::Generate => crate::stm32cubemx::generate_code(),
        Commands::Update(args) => crate::update::run(args),
    }
}
