use crate::utils::find_single_ioc_file;
use anyhow::Result;
use rand::distr::Alphanumeric;
use rand::{Rng, rng};
use std::env;
use std::fmt::Write;
use std::fs::{File, remove_file};
use std::io::Write as IoWrite;
use std::process::{Command, Stdio};
use tracing::{error, warn};

fn generate_random_string(length: usize) -> String {
    let mut rng = rng();
    (0..length)
        .map(|_| rng.sample(Alphanumeric))
        .map(char::from)
        .collect()
}

const CMAKE_TOOLCHAIN: &str = "CMake";

pub fn generate_code() -> Result<()> {
    let ioc_file = find_single_ioc_file().map_err(|error| {
        warn!("{error}");
        error
    })?;
    let ioc_file = ioc_file.to_string_lossy().to_string();
    let mut script = String::new();
    writeln!(script, "config load {ioc_file}")?;
    writeln!(script, "project toolchain \"{CMAKE_TOOLCHAIN}\"")?;
    writeln!(script, "project couplefilesbyip 1")?;
    writeln!(script, "project generate")?;
    write!(script, "exit")?;

    run_script(script)
}

pub fn run_script(script: String) -> Result<()> {
    // generate tmp file in system temp directory
    let tmp_path = env::temp_dir().join(format!("tmp-script-{}", generate_random_string(8)));
    let tmp_path_str = tmp_path.to_str().expect("failed to convert path to string");

    // create temporary script file
    let mut temp_script_file = File::create(&tmp_path)?;
    temp_script_file.write_all(script.as_bytes())?;

    // run stm32cubemx
    let status = if cfg!(target_os = "windows") {
        // return Err(anyhow::anyhow!("not support windows"));
        let dir = match env::var("STM32CubeMX_dir") {
            Ok(dir) => dir,
            Err(_) => {
                error!(
                    "Environment variable STM32CubeMX_dir is not set. Please configure the STM32CubeMX installation path."
                );
                return Err(anyhow::anyhow!(
                    "Missing environment variable: STM32CubeMX_dir"
                ));
            }
        };
        Command::new("cmd")
            .args([
                "/C",
                &format!(
                    r#""{dir}\jre\bin\java.exe" -jar "{dir}\STM32CubeMX.exe" -s "{tmp_path_str}" -q"#,
                ),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
    } else {
        Command::new("stm32cubemx")
            .arg("-s")
            .arg(tmp_path_str)
            .arg("-q")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
    };

    // remove temp file
    remove_file(tmp_path)?;

    // handle result
    match status {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => {
            error!("Run script failed with status: {}", status);
            Err(anyhow::anyhow!("Run script failed with status: {status}"))
        }
        Err(e) => {
            error!("Failed to execute stm32cubemx: {}", e);
            Err(anyhow::anyhow!("Failed to execute stm32cubemx: {e}"))
        }
    }
}
