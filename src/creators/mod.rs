use crate::creators::stm32f407vetx::STM32F407VETx;
use crate::creators::stm32h723vetx::STM32H723VETx;
use crate::render::render_string;
use crate::stm32cubemx::run_script;
use anyhow::Context;
use clap::Parser;
use serde::Serialize;
use tracing::info;

mod stm32f407vetx;
mod stm32h723vetx;

#[derive(Debug, Parser)]
pub struct CreatorArgs {}

#[derive(Serialize)]
pub struct CreateContext<'a> {
    pub project_name: &'a str,
    pub project_dir: &'a str,
    pub ioc_file_path: &'a str,
}

pub trait STM32ProjectCreator {
    fn name(&self) -> &'static str;
    fn run(&self, ctx: &CreateContext<'_>) -> anyhow::Result<()>;
}

pub fn all() -> Vec<Box<dyn STM32ProjectCreator>> {
    vec![Box::new(STM32F407VETx), Box::new(STM32H723VETx)]
}

pub fn run_rendered_script(
    step: &str,
    template: &str,
    ctx: &CreateContext<'_>,
) -> anyhow::Result<()> {
    info!("Running {step} script");
    let script =
        render_string(template, ctx).with_context(|| format!("Failed to render {step} script"))?;

    run_script(script).with_context(|| format!("Failed to run {step} script"))
}
