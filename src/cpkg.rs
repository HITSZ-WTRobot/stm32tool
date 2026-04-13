use crate::utils::find_single_ioc_file;
use anyhow::{Context, anyhow};
use std::ffi::OsString;
use std::process::Command;
use tracing::{info, warn};

pub fn bootstrap_project(force: bool) -> anyhow::Result<()> {
    if !is_available() {
        warn!(
            "cpkg is not available in PATH; please add `BasicComponents/utils` to the project manually to satisfy `UserCode/arena.cpp` dependencies."
        );
        return Ok(());
    }

    let ioc_file = find_single_ioc_file()?;
    let project_name = ioc_file
        .file_stem()
        .ok_or_else(|| anyhow!("Invalid ioc file name: {}", ioc_file.display()))?
        .to_string_lossy()
        .to_string();
    let ioc_arg: OsString = ioc_file
        .file_name()
        .ok_or_else(|| anyhow!("Invalid ioc file path: {}", ioc_file.display()))?
        .to_os_string();

    info!("Initializing cpkg project metadata");
    let mut init_command = Command::new("cpkg");
    init_command.arg("init");
    if force {
        init_command.arg("--force");
    }
    init_command
        .arg("--name")
        .arg(&project_name)
        .arg("--ioc")
        .arg(&ioc_arg);
    run_command(init_command, "cpkg init")?;

    info!("Adding cpkg dependency utils in offline mode");
    let mut add_command = Command::new("cpkg");
    add_command.args(["add", "--offline", "utils"]);
    run_command(
        add_command,
        "cpkg add --offline utils（若本地或缓存索引不可用，则不支持离线完成）",
    )?;

    Ok(())
}

fn is_available() -> bool {
    Command::new("cpkg")
        .arg("--version")
        .status()
        .is_ok_and(|status| status.success())
}

fn run_command(mut command: Command, label: &str) -> anyhow::Result<()> {
    let status = command
        .status()
        .with_context(|| format!("Failed to execute {label}"))?;

    if status.success() {
        return Ok(());
    }

    Err(anyhow!("{label} failed with status: {status}"))
}
