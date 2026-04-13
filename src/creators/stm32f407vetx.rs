use crate::configs::cubemx_scripts;
use crate::creators::{CreateContext, STM32ProjectCreator, run_rendered_script};
use crate::patches::{Patch, apply_patch};
use tracing::{debug, info};

pub struct STM32F407VETx;

impl STM32ProjectCreator for STM32F407VETx {
    fn name(&self) -> &'static str {
        "STM32F407VETx"
    }

    fn run(&self, ctx: &CreateContext<'_>) -> anyhow::Result<()> {
        debug!(?ctx, "Running STM32F407VETx project creator");
        run_rendered_script("bootstrap", cubemx_scripts::stm32f407vetx::BOOTSTRAP, ctx)?;

        info!("Patching .ioc file");
        apply_patch(&Patch::RegexReplace {
            file: format!("{}.ioc", ctx.project_name),
            pattern: r"RCC\.HSE_VALUE=(\d+)".to_string(),
            insert: "RCC.HSE_VALUE=8000000".to_string(),
        })?;

        run_rendered_script("generate", cubemx_scripts::stm32f407vetx::GENERATE, ctx)
    }
}
