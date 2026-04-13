use crate::cli::InitArgs;
use crate::gitignore::generate_gitignore;
use crate::initializers;
use crate::render::render_file;
use crate::templates;
use crate::utils::get_author;
use chrono::Local;
use serde::Serialize;
use std::{fs, process::Command, process::Stdio};
use tracing::{error, info};

#[derive(Serialize)]
struct InitContext {
    author: String,
    date: String,
    year: String,
}

const USER_CODE_DIRECTORIES: [&str; 8] = [
    "UserCode/bsp",
    "UserCode/drivers",
    "UserCode/third_party",
    "UserCode/libs",
    "UserCode/interfaces",
    "UserCode/controllers",
    "UserCode/app",
    "Modules",
];

pub fn run(args: InitArgs) -> anyhow::Result<()> {
    let ctx = init_context();

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

    initializers::init_cmake(!args.skip_non_intrusive_headers && !args.skip_generate_user_code)?;

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
    let status = Command::new("git")
        .arg("init")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

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

    for dir in USER_CODE_DIRECTORIES {
        fs::create_dir_all(dir)?;
        info!("Created dir {}", dir);
    }

    render_file("UserCode/app/app.h", templates::APP_H, ctx, force)?;
    render_file("UserCode/app/app.c", templates::APP_C, ctx, force)?;
    render_file(
        "UserCode/README.md",
        templates::USER_CODE_README,
        ctx,
        force,
    )?;

    Ok(())
}

fn create_initial_commit() {
    let status = Command::new("git").args(["add", "."]).status();
    match status {
        Ok(status) if status.success() => {
            let status = Command::new("git")
                .args(["commit", "-m", "Initial commit"])
                .status();
            if !matches!(status, Ok(status) if status.success()) {
                error!("Git first commit failed");
            }
        }
        Ok(_) | Err(_) => {
            error!("Git first commit failed");
        }
    }
}
