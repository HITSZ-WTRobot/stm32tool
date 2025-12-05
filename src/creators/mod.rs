use crate::creators::stm32f407vetx::STM32F407VETx;
use crate::creators::stm32h723vetx::STM32H723VETx;
use clap::Parser;
use serde::Serialize;

mod stm32f407vetx;
mod stm32h723vetx;

#[derive(Debug, Parser)]
pub struct CreatorArgs {}

#[derive(Serialize)]
pub struct CreateContext<'a> {
    pub project_name: &'a String,
    pub project_dir: &'a String,
    pub ioc_file_path: &'a String,
    pub toolchain: &'a str,
    pub generate_under_root: bool,
}

pub trait STM32ProjectCreator {
    fn name(&self) -> &'static str;
    fn run(&self, ctx: &CreateContext) -> anyhow::Result<()>;
}

pub fn all() -> Vec<Box<dyn STM32ProjectCreator>> {
    vec![Box::new(STM32F407VETx), Box::new(STM32H723VETx)]
}
