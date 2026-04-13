use crate::configs::cubemx_scripts;
use crate::creators::{CreateContext, STM32ProjectCreator, run_rendered_script};
use crate::patches::{Patch, apply_patch};
use tracing::{debug, info};

pub struct STM32H723VETx;

impl STM32ProjectCreator for STM32H723VETx {
    fn name(&self) -> &'static str {
        "STM32H723VETx"
    }

    fn run(&self, ctx: &CreateContext<'_>) -> anyhow::Result<()> {
        debug!(?ctx, "Running STM32H723VETx project creator");
        run_rendered_script("bootstrap", cubemx_scripts::stm32h723vetx::BOOTSTRAP, ctx)?;

        info!("Patching .ioc file");
        apply_patch(&Patch::RegexReplace {
            file: format!("{}.ioc", ctx.project_name),
            pattern: r"RCC\.HSE_VALUE=(\d+)".to_string(),
            insert: "RCC.HSE_VALUE=8000000".to_string(),
        })?;
        apply_patch(&Patch::RegexReplace {
            file: format!("{}.ioc", ctx.project_name),
            pattern: "(MMT.+\n)+".to_string(),
            insert: cubemx_scripts::stm32h723vetx::DEFAULT_MEMORY_MAP.to_string(),
        })?;

        run_rendered_script("generate", cubemx_scripts::stm32h723vetx::GENERATE, ctx)
    }
}
