use crate::utils::{command_status, find_single_ioc_file};
#[cfg(any(target_os = "windows", test))]
use anyhow::bail;
use anyhow::{Context, Result};
use rand::distr::Alphanumeric;
use rand::{Rng, rng};
use std::env;
use std::fmt::Write;
use std::fs::{File, remove_file};
use std::io::Write as IoWrite;
use std::path::Path;
#[cfg(any(target_os = "windows", test))]
use std::path::PathBuf;
use std::process::{Command, ExitStatus};
use tracing::{debug, error, warn};

fn generate_random_string(length: usize) -> String {
    let mut rng = rng();
    (0..length)
        .map(|_| rng.sample(Alphanumeric))
        .map(char::from)
        .collect()
}

const CMAKE_TOOLCHAIN: &str = "CMake";

#[cfg(any(target_os = "windows", test))]
#[derive(Debug, PartialEq, Eq)]
struct WindowsCubeMxInstallation {
    java_executable: PathBuf,
    cubemx_executable: PathBuf,
}

pub fn generate_code() -> Result<()> {
    let ioc_file = find_single_ioc_file().map_err(|error| {
        warn!("{error}");
        error
    })?;
    debug!(ioc_file = %ioc_file.display(), "Generating code from .ioc file");
    let ioc_file = ioc_file.to_string_lossy().to_string();
    let mut script = String::new();
    writeln!(script, "config load {ioc_file}")?;
    writeln!(script, "project toolchain \"{CMAKE_TOOLCHAIN}\"")?;
    writeln!(script, "project couplefilesbyip 1")?;
    writeln!(script, "project generate")?;
    write!(script, "exit")?;

    run_script(script)
}

#[cfg(target_os = "windows")]
fn read_windows_cubemx_installation() -> Result<WindowsCubeMxInstallation> {
    let dir = env::var("STM32CubeMX_PATH").with_context(|| {
        "Missing environment variable: STM32CubeMX_PATH. Please configure the STM32CubeMX installation path."
    })?;

    resolve_windows_cubemx_installation(Path::new(&dir))
}

#[cfg(any(target_os = "windows", test))]
fn resolve_windows_cubemx_installation(dir: &Path) -> Result<WindowsCubeMxInstallation> {
    if !dir.exists() {
        bail!(
            "`STM32CubeMX_PATH` points to a non-existent path: {}",
            dir.display()
        );
    }

    if !dir.is_dir() {
        bail!(
            "`STM32CubeMX_PATH` must point to the STM32CubeMX installation directory: {}",
            dir.display()
        );
    }

    let java_executable = dir.join("jre").join("bin").join("java.exe");
    ensure_file_exists(
        &java_executable,
        "`STM32CubeMX_PATH` is invalid; missing Java executable",
    )?;

    let cubemx_executable = dir.join("STM32CubeMX.exe");
    ensure_file_exists(
        &cubemx_executable,
        "`STM32CubeMX_PATH` is invalid; missing STM32CubeMX executable",
    )?;

    Ok(WindowsCubeMxInstallation {
        java_executable,
        cubemx_executable,
    })
}

#[cfg(any(target_os = "windows", test))]
fn ensure_file_exists(path: &Path, message: &str) -> Result<()> {
    if path.is_file() {
        return Ok(());
    }

    bail!("{message}: {}", path.display())
}

#[cfg(target_os = "windows")]
fn run_stm32cubemx(tmp_path: &Path) -> Result<ExitStatus> {
    let installation = read_windows_cubemx_installation()?;
    let mut command = Command::new(&installation.java_executable);
    command
        .arg("-jar")
        .arg(&installation.cubemx_executable)
        .arg("-s")
        .arg(tmp_path)
        .arg("-q");
    let label = format!(
        "{} -jar {} -s {} -q",
        installation.java_executable.display(),
        installation.cubemx_executable.display(),
        tmp_path.display()
    );

    command_status(command, &label).with_context(|| {
        format!(
            "Failed to execute STM32CubeMX via {}",
            installation.java_executable.display()
        )
    })
}

#[cfg(not(target_os = "windows"))]
fn run_stm32cubemx(tmp_path: &Path) -> Result<ExitStatus> {
    let mut command = Command::new("stm32cubemx");
    command.arg("-s").arg(tmp_path).arg("-q");
    let label = format!("stm32cubemx -s {} -q", tmp_path.display());

    command_status(command, &label).context("Failed to execute stm32cubemx")
}

pub fn run_script(script: String) -> Result<()> {
    let tmp_path = env::temp_dir().join(format!("tmp-script-{}", generate_random_string(8)));
    debug!(
        tmp_path = %tmp_path.display(),
        bytes = script.len(),
        lines = script.lines().count(),
        "Writing temporary STM32CubeMX script"
    );

    let mut temp_script_file = File::create(&tmp_path)?;
    temp_script_file.write_all(script.as_bytes())?;

    let status = run_stm32cubemx(&tmp_path);

    debug!(tmp_path = %tmp_path.display(), "Removing temporary STM32CubeMX script");
    remove_file(tmp_path)?;

    match status {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => {
            error!("Run script failed with status: {}", status);
            Err(anyhow::anyhow!("Run script failed with status: {status}"))
        }
        Err(error) => {
            error!("{error:#}");
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir_all, remove_dir_all, write};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Result<Self> {
            let unique_suffix = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time before unix epoch")
                .as_nanos();
            let path = env::temp_dir().join(format!(
                "stm32tool-test-{}-{unique_suffix}",
                std::process::id()
            ));
            create_dir_all(&path)?;
            Ok(Self { path })
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = remove_dir_all(&self.path);
        }
    }

    #[test]
    fn rejects_missing_installation_directory() -> Result<()> {
        let temp_dir = TestDir::new()?;
        let missing_dir = temp_dir.path().join("missing");

        let error = resolve_windows_cubemx_installation(&missing_dir).unwrap_err();

        assert!(error.to_string().contains("non-existent path"));
        Ok(())
    }

    #[test]
    fn rejects_missing_java_executable() -> Result<()> {
        let temp_dir = TestDir::new()?;

        let error = resolve_windows_cubemx_installation(temp_dir.path()).unwrap_err();

        assert!(error.to_string().contains("java.exe"));
        Ok(())
    }

    #[test]
    fn rejects_missing_cubemx_executable() -> Result<()> {
        let temp_dir = TestDir::new()?;
        let java_dir = temp_dir.path().join("jre").join("bin");
        create_dir_all(&java_dir)?;
        write(java_dir.join("java.exe"), b"")?;

        let error = resolve_windows_cubemx_installation(temp_dir.path()).unwrap_err();

        assert!(error.to_string().contains("STM32CubeMX.exe"));
        Ok(())
    }

    #[test]
    fn resolves_installation_when_expected_files_exist() -> Result<()> {
        let temp_dir = TestDir::new()?;
        let java_executable = temp_dir.path().join("jre").join("bin").join("java.exe");
        create_dir_all(
            java_executable
                .parent()
                .expect("java path should have a parent"),
        )?;
        write(&java_executable, b"")?;

        let cubemx_executable = temp_dir.path().join("STM32CubeMX.exe");
        write(&cubemx_executable, b"")?;

        let installation = resolve_windows_cubemx_installation(temp_dir.path())?;

        assert_eq!(
            installation,
            WindowsCubeMxInstallation {
                java_executable,
                cubemx_executable,
            }
        );
        Ok(())
    }
}
