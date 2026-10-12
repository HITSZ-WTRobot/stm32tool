use crate::cli::InitArgs;
use crate::cpkg;
use crate::gitignore::generate_gitignore;
use crate::initializers;
use crate::render::render_file;
use crate::templates;
use crate::utils::{command_status, find_single_ioc_file, get_author};
use anyhow::Context as _;
use chrono::Local;
use serde::Serialize;
use std::{fs, process::Command};
use tracing::{debug, error, info};

#[derive(Debug, Serialize)]
struct InitContext {
    author: String,
    date: String,
    year: String,
}

const USER_CODE_DIRECTORIES: [&str; 1] = ["UserCode"];

pub fn run(args: InitArgs) -> anyhow::Result<()> {
    debug!(?args, "Running init command");
    let ctx = init_context();
    debug!(?ctx, "Prepared init context");

    init_git_repository();
    info!("Generating .gitignore file...");
    generate_gitignore(None, args.force)?;

    if !args.skip_generate_clang_format {
        info!("Generating .clang-format file");
        render_file(".clang-format", templates::CLANG_FORMAT, &ctx, args.force)?;
    }

    if !args.skip_generate_user_code {
        generate_user_code_layout(&ctx, args.force)?;
    }

    initializers::init_cmake()?;
    cpkg::bootstrap_project(args.force)?;

    create_initial_commit();

    info!("STM32 project initialized!");
    Ok(())
}

fn init_context() -> InitContext {
    let now = Local::now();

    InitContext {
        author: get_author(),
        date: now.format("%Y-%m-%d").to_string(),
        year: now.format("%Y").to_string(),
    }
}

fn init_git_repository() {
    info!("Initializing git repository...");
    let mut command = Command::new("git");
    command.arg("init");
    let status = command_status(command, "git init");

    match status {
        Ok(status) if status.success() => {
            info!("Git repository initialized successfully!");
        }
        Ok(status) => {
            error!("Git init failed with status: {}", status);
        }
        Err(error) => {
            error!("Failed to execute git: {}", error);
        }
    }
}

fn generate_user_code_layout(ctx: &InitContext, force: bool) -> anyhow::Result<()> {
    info!("Generating user code directories...");

    // Resolve the exact MCU part number before writing any UserCode file so
    // that an unreadable or ambiguous .ioc fails without leaving partial output.
    let ioc_file = find_single_ioc_file()?;
    let ioc_contents = fs::read_to_string(&ioc_file)
        .with_context(|| format!("Failed to read .ioc file {}", ioc_file.display()))?;
    let is_h723vet6 = is_stm32h723vet6_ioc(&ioc_contents);
    debug!(
        ioc_file = %ioc_file.display(),
        is_h723vet6, "Resolved MCU part number from .ioc file"
    );

    debug!(?ctx, force, "Generating default UserCode layout");

    for dir in USER_CODE_DIRECTORIES {
        fs::create_dir_all(dir)?;
        info!("Created dir {}", dir);
    }

    render_file("UserCode/app.cpp", templates::APP_CPP, ctx, force)?;

    if is_h723vet6 {
        info!("Detected STM32H723VET6 MCU, generating UserCode/memory.hpp");
        render_file("UserCode/memory.hpp", templates::MEMORY_HPP, ctx, force)?;
    }

    Ok(())
}

/// Returns `true` when the .ioc contents declare `Mcu.CPN=STM32H723VET6`.
fn is_stm32h723vet6_ioc(ioc_contents: &str) -> bool {
    ioc_contents.lines().any(|line| {
        let Some((key, value)) = line.split_once('=') else {
            return false;
        };

        key.trim() == "Mcu.CPN" && value.trim() == "STM32H723VET6"
    })
}

fn create_initial_commit() {
    debug!("Creating initial git commit");
    let mut add_command = Command::new("git");
    add_command.args(["add", "."]);
    let status = command_status(add_command, "git add .");
    match status {
        Ok(status) if status.success() => {
            let mut commit_command = Command::new("git");
            commit_command.args(["commit", "-m", "Initial commit"]);
            let status = command_status(commit_command, "git commit -m Initial commit");
            if !matches!(status, Ok(status) if status.success()) {
                error!("Git first commit failed");
            }
        }
        Ok(_) | Err(_) => {
            error!("Git first commit failed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::is_stm32h723vet6_ioc;

    #[test]
    fn detects_stm32h723vet6_cpn() {
        let ioc = "ProjectManager.ProjectName=demo\nMcu.Family=STM32H7\nMcu.Name=STM32H723VETx\nMcu.CPN=STM32H723VET6\n";

        assert!(is_stm32h723vet6_ioc(ioc));
    }

    #[test]
    fn detects_stm32h723vet6_cpn_with_crlf_and_whitespace() {
        let ioc = "ProjectManager.ProjectName=demo\r\nMcu.Family = STM32H7 \r\nMcu.CPN = STM32H723VET6 \r\nMcu.Name=STM32H723VETx\r\n";

        assert!(is_stm32h723vet6_ioc(ioc));
    }

    #[test]
    fn rejects_ioc_files_without_exact_stm32h723vet6_cpn() {
        let cases: [(&str, &str); 9] = [
            (
                "adjacent h723 part numbers",
                "Mcu.Family=STM32H7\nMcu.CPN=STM32H723VET7\n",
            ),
            (
                "different package h723vet",
                "Mcu.Family=STM32H7\nMcu.CPN=STM32H723ZET6\n",
            ),
            (
                "different flash h723vet",
                "Mcu.Family=STM32H7\nMcu.CPN=STM32H723VGT6\n",
            ),
            (
                "other h7 part number",
                "Mcu.Family=STM32H7\nMcu.CPN=STM32H743VIT6\n",
            ),
            (
                "f4 part number",
                "Mcu.Family=STM32F4\nMcu.CPN=STM32F407VGT6\n",
            ),
            (
                "family and name only, cpn missing",
                "Mcu.Family=STM32H7\nMcu.Name=STM32H723VETx\n",
            ),
            (
                "h7 name with x suffix instead of cpn",
                "Mcu.Family=STM32H7\nMcu.Name=STM32H723VETx\nMcu.CPN=STM32H723VETx\n",
            ),
            (
                "commented out cpn",
                "# Mcu.CPN=STM32H723VET6\nMcu.Family=STM32F4\n",
            ),
            (
                "full part number in unrelated fields",
                "Mcu.Name=STM32H743ZITx\nMcu.CPN=STM32H7B3IIT6\nMcu.Family=STM32H7\nProjectManager.ProjectName=STM32H723VET6-board\n",
            ),
        ];

        for (case, ioc) in cases {
            assert!(!is_stm32h723vet6_ioc(ioc), "should reject: {case}");
        }
    }
}
