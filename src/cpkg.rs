use crate::utils::{command_status, command_status_live, find_single_ioc_file};
use anyhow::{Context, anyhow};
use dialoguer::Confirm;
use std::ffi::OsString;
use std::io::{self, IsTerminal};
use std::process::Command;
use tracing::{debug, info, warn};

const SYNC_HINT: &str =
    "Please run `cpkg sync` manually to finish project dependency synchronization.";

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
    debug!(
        project_name,
        ioc_file = %ioc_file.display(),
        force,
        "Preparing cpkg bootstrap"
    );

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

    offer_sync();

    Ok(())
}

/// Asks whether to run `cpkg sync` right away.
///
/// The terminal check is mandatory, not cosmetic: `console`'s `Term::read_key`
/// returns `Key::Unknown` forever when the prompt target is not a tty, and
/// `dialoguer::Confirm::interact` loops on `Key::Unknown`, so prompting from a
/// redirected stderr would spin instead of failing.
fn offer_sync() {
    if !io::stderr().is_terminal() {
        debug!("Skipping cpkg sync prompt: stderr is not a terminal");
        warn!("{SYNC_HINT}");
        return;
    }

    let confirmed = Confirm::new()
        .with_prompt(
            "Run `cpkg sync` now to download dependencies and generate cmake/wtr_modules.cmake?",
        )
        .default(true)
        .interact()
        .unwrap_or(false);

    if !confirmed {
        info!("cpkg sync skipped");
        warn!("{SYNC_HINT}");
        return;
    }

    info!("Synchronizing cpkg dependencies (network access may be required)");
    let mut command = Command::new("cpkg");
    command.arg("sync");

    match command_status_live(command, "cpkg sync") {
        Ok(status) if status.success() => info!("cpkg sync finished"),
        Ok(status) => warn!("cpkg sync failed with status: {status}; {SYNC_HINT}"),
        Err(error) => warn!("Failed to execute cpkg sync: {error}; {SYNC_HINT}"),
    }
}

fn is_available() -> bool {
    let mut command = Command::new("cpkg");
    command.arg("--version");

    command_status(command, "cpkg --version").is_ok_and(|status| status.success())
}

fn run_command(command: Command, label: &str) -> anyhow::Result<()> {
    let status =
        command_status(command, label).with_context(|| format!("Failed to execute {label}"))?;

    if status.success() {
        return Ok(());
    }

    Err(anyhow!("{label} failed with status: {status}"))
}
