use crate::cli::InitArgs;
use crate::cpkg;
use crate::gitignore::generate_gitignore;
use crate::initializers;
use crate::render::render_file;
use crate::templates;
use crate::utils::{command_status, get_author};
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
    debug!(?ctx, force, "Generating default UserCode layout");

    for dir in USER_CODE_DIRECTORIES {
        fs::create_dir_all(dir)?;
        info!("Created dir {}", dir);
    }

    render_file("UserCode/app.cpp", templates::APP_CPP, ctx, force)?;
    render_file("UserCode/arena.cpp", templates::ARENA_CPP, ctx, force)?;

    Ok(())
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
