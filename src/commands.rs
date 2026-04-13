use crate::cli::{Commands, CreateArgs, InitArgs};
use crate::creators::CreateContext;
use crate::generate_gitignore::generate_gitignore;
use crate::initializers;
use crate::render::render_file;
use crate::stm32cubemx::generate_code;
use crate::utils::get_author;
use anyhow::anyhow;
use chrono::Local;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Select};
use serde::Serialize;
use std::path::Path;
use std::process::{Command, Stdio};
use std::{env, fs};
use tracing::{error, info};

#[derive(Serialize)]
struct InitContext {
    author: String,
    date: String,
    year: String,
}

pub fn run(command: Commands) -> anyhow::Result<()> {
    match command {
        Commands::Init(args) => run_init(args),
        Commands::Create(args) => run_create(args),
        Commands::Purge => run_purge(),
        Commands::Generate => generate_code(),
    }
}

fn run_purge() -> anyhow::Result<()> {
    let status = Command::new("git")
        .args(["clean", "-fdX"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if status.success() {
        info!("purge successfully!");
    } else {
        error!("purge failed!, {}", status);
    }

    Ok(())
}

fn run_init(args: InitArgs) -> anyhow::Result<()> {
    let ctx = init_context();

    init_git_repository();
    info!("Generating .gitignore file...");
    generate_gitignore(None, args.force)?;

    if !args.skip_generate_clang_format {
        info!("Generating .clang-format file");
        render_file(
            ".clang-format",
            include_str!("templates/clang-format.tmpl"),
            &ctx,
            args.force,
        )?;
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
        Err(e) => {
            error!("Failed to execute git: {}", e);
        }
    }
}

fn generate_user_code_layout(ctx: &InitContext, force: bool) -> anyhow::Result<()> {
    info!("Generating user code directories...");

    let directories = [
        "UserCode/bsp",
        "UserCode/drivers",
        "UserCode/third_party",
        "UserCode/libs",
        "UserCode/interfaces",
        "UserCode/controllers",
        "UserCode/app",
        "Modules",
    ];

    for dir in directories {
        fs::create_dir_all(dir)?;
        info!("Created dir {}", dir);
    }

    render_file(
        "UserCode/app/app.h",
        include_str!("templates/app.h.tmpl"),
        ctx,
        force,
    )?;
    render_file(
        "UserCode/app/app.c",
        include_str!("templates/app.c.tmpl"),
        ctx,
        force,
    )?;
    render_file(
        "UserCode/README.md",
        include_str!("templates/README.md.tmpl"),
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

fn run_create(args: CreateArgs) -> anyhow::Result<()> {
    let path = Path::new(&args.project_name);
    if path.exists() {
        let result = Confirm::new()
            .with_prompt(
                "Project already exists. Regenerate? This will delete all existing content.",
            )
            .default(false)
            .interact()?;
        if !result {
            info!("Creation aborted!");
            return Err(anyhow!("Creation aborted!"));
        }
        fs::remove_dir_all(path)?;
    }

    fs::create_dir_all(&args.project_name)?;
    env::set_current_dir(&args.project_name)?;

    let current_dir = env::current_dir()?;
    let mcus = crate::creators::all();
    let items: Vec<&str> = mcus.iter().map(|creator| creator.name()).collect();

    let chosen = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose MCU")
        .items(&items)
        .interact()?;

    let project_dir = current_dir.to_string_lossy().to_string();
    let ioc_file_path = current_dir
        .join(format!("{}.ioc", args.project_name))
        .to_string_lossy()
        .to_string();

    let ctx = CreateContext {
        project_name: &args.project_name,
        project_dir: &project_dir,
        ioc_file_path: &ioc_file_path,
    };

    info!("Using toolchain CMake");
    mcus[chosen].run(&ctx)?;

    if args.run_init {
        info!("Running init process");
        run_init(args.init_args)?;
    }

    Ok(())
}
