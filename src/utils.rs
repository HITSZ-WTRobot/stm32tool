use anyhow::anyhow;
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

pub fn get_author() -> String {
    Command::new("git")
        .args(["config", "user.name"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "unknown".into())
        .trim()
        .to_string()
}

pub fn find_single_ioc_file() -> anyhow::Result<PathBuf> {
    let current_dir = env::current_dir()?;
    let mut ioc_files = fs::read_dir(&current_dir)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("ioc"))
        .collect::<Vec<_>>();
    ioc_files.sort();

    match ioc_files.as_slice() {
        [ioc_file] => Ok(ioc_file.clone()),
        [] => Err(anyhow!("No .ioc file found in {}", current_dir.display())),
        _ => Err(anyhow!(
            "Multiple .ioc files found in {}",
            current_dir.display()
        )),
    }
}
