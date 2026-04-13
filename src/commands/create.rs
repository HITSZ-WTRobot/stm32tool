use crate::cli::CreateArgs;
use crate::creators::{CreateContext, all};
use anyhow::anyhow;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Select};
use std::path::Path;
use std::{env, fs};
use tracing::info;

pub fn run(args: CreateArgs) -> anyhow::Result<()> {
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
    let mcus = all();
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
        super::init::run(args.init_args)?;
    }

    Ok(())
}
