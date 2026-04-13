use crate::utils::command_status;
use std::process::Command;
use tracing::{error, info};

pub fn run() -> anyhow::Result<()> {
    let mut command = Command::new("git");
    command.args(["clean", "-fdX"]);
    let status = command_status(command, "git clean -fdX")?;

    if status.success() {
        info!("purge successfully!");
    } else {
        error!("purge failed!, {}", status);
    }

    Ok(())
}
