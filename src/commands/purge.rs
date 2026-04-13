use std::process::{Command, Stdio};
use tracing::{error, info};

pub fn run() -> anyhow::Result<()> {
    let status = Command::new("git")
        .args(["clean", "-fdX"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if status.success() {
        info!("purge successfully!");
    } else {
        error!("purge failed!, {}", status);
    }

    Ok(())
}
