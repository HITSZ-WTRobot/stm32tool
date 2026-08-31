use crate::configs::cubemx_scripts;
use crate::creators::{CreateContext, STM32ProjectCreator, run_rendered_script};
use tracing::debug;

pub struct STM32G474CBTx;

impl STM32ProjectCreator for STM32G474CBTx {
    fn name(&self) -> &'static str {
        "STM32G474CBTx"
    }

    fn run(&self, ctx: &CreateContext<'_>) -> anyhow::Result<()> {
        debug!(?ctx, "Running STM32G474CBTx project creator");
        run_rendered_script("bootstrap", cubemx_scripts::stm32g474cbtx::BOOTSTRAP, ctx)
    }
}
