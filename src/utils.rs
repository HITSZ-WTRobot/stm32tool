use anyhow::anyhow;
use std::io::{self, BufRead, BufReader, Read};
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Output, Stdio};
use std::thread;
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
        return command_status_streaming(command, label);
    }

    command.stdout(Stdio::null()).stderr(Stdio::null()).status()
}

/// Runs a command with stdout/stderr inherited, so long-running progress stays visible.
pub fn command_status_live(mut command: Command, label: &str) -> std::io::Result<ExitStatus> {
    debug!("Running command: {label}");
    let status = command.status();
    match &status {
        Ok(status) => debug!("Command finished: {label} ({status})"),
        Err(error) => debug!("Command failed to start: {label} ({error})"),
    }
    status
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

fn command_status_streaming(mut command: Command, label: &str) -> io::Result<ExitStatus> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = command.spawn()?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other(format!("Missing stdout pipe for {label}")))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| io::Error::other(format!("Missing stderr pipe for {label}")))?;

    let stdout_handle = spawn_stream_logger(label.to_string(), "stdout", stdout);
    let stderr_handle = spawn_stream_logger(label.to_string(), "stderr", stderr);

    let status = child.wait()?;

    join_stream_logger(stdout_handle, label, "stdout")?;
    join_stream_logger(stderr_handle, label, "stderr")?;

    debug!("Command finished: {label} ({status})");
    Ok(status)
}

fn spawn_stream_logger<R>(
    label: String,
    stream: &'static str,
    reader: R,
) -> thread::JoinHandle<io::Result<()>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut reader = BufReader::new(reader);
        let mut buffer = Vec::new();

        loop {
            buffer.clear();
            let bytes_read = reader.read_until(b'\n', &mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            while matches!(buffer.last(), Some(b'\n' | b'\r')) {
                buffer.pop();
            }

            let line = String::from_utf8_lossy(&buffer);
            debug!("{label} {stream}: {line}");
        }

        Ok(())
    })
}

fn join_stream_logger(
    handle: thread::JoinHandle<io::Result<()>>,
    label: &str,
    stream: &str,
) -> io::Result<()> {
    match handle.join() {
        Ok(result) => result,
        Err(_) => Err(io::Error::other(format!(
            "Failed to join {stream} logger thread for {label}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::command_status;
    use std::process::Command;
    use tracing::Level;

    #[test]
    fn command_status_supports_streaming_in_debug_mode() {
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .with_test_writer()
            .finish();

        tracing::subscriber::with_default(subscriber, || {
            let command = test_command();
            let status = command_status(command, "test command").expect("run command");

            assert!(status.success());
        });
    }

    #[cfg(target_os = "windows")]
    fn test_command() -> Command {
        let mut command = Command::new("cmd");
        command.args(["/C", "echo hello & echo error 1>&2"]);
        command
    }

    #[cfg(not(target_os = "windows"))]
    fn test_command() -> Command {
        let mut command = Command::new("sh");
        command.args(["-c", "printf 'hello\\n'; printf 'error\\n' 1>&2"]);
        command
    }
}
