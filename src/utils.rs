use anyhow::anyhow;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Output, Stdio};
use std::{env, fs};
use tracing::{Level, debug};

pub fn get_author() -> String {
    let mut command = Command::new("git");
    command.args(["config", "user.name"]);

    command_output(command, "git config user.name")
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "unknown".into())
        .trim()
        .to_string()
}

pub fn find_single_ioc_file() -> anyhow::Result<PathBuf> {
    let current_dir = env::current_dir()?;
    debug!("Searching for .ioc files in {}", current_dir.display());
    let mut ioc_files = fs::read_dir(&current_dir)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("ioc"))
        .collect::<Vec<_>>();
    ioc_files.sort();
    let discovered_files = ioc_files
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>();
    debug!(files = ?discovered_files, "Discovered .ioc files");

    match ioc_files.as_slice() {
        [ioc_file] => Ok(ioc_file.clone()),
        [] => Err(anyhow!("No .ioc file found in {}", current_dir.display())),
        _ => Err(anyhow!(
            "Multiple .ioc files found in {}",
            current_dir.display()
        )),
    }
}

pub fn command_status(mut command: Command, label: &str) -> std::io::Result<ExitStatus> {
    debug!("Running command: {label}");

    if tracing::enabled!(Level::DEBUG) {
        let output = command.output();
        log_command_result(label, &output);
        return output.map(|output| output.status);
    }

    command.stdout(Stdio::null()).stderr(Stdio::null()).status()
}

pub fn command_output(mut command: Command, label: &str) -> std::io::Result<Output> {
    debug!("Running command: {label}");

    let output = command.output();
    log_command_result(label, &output);
    output
}

fn log_command_result(label: &str, output: &std::io::Result<Output>) {
    match output {
        Ok(output) => {
            log_command_stream(label, "stdout", &output.stdout);
            log_command_stream(label, "stderr", &output.stderr);
            debug!("Command finished: {label} ({})", output.status);
        }
        Err(error) => {
            debug!("Command failed to start: {label} ({error})");
        }
    }
}

fn log_command_stream(label: &str, stream: &str, bytes: &[u8]) {
    if bytes.is_empty() {
        return;
    }

    for line in String::from_utf8_lossy(bytes).lines() {
        debug!("{label} {stream}: {line}");
    }
}
