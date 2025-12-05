use crate::creators::{CreateContext, STM32ProjectCreator};
use crate::patches::{Patch, apply_patch};
use crate::render::render_string;
use crate::stm32cubemx::run_script;
use anyhow::anyhow;
use tracing::{error, info};

pub struct STM32H723VETx;

impl STM32ProjectCreator for STM32H723VETx {
    fn name(&self) -> &'static str {
        "STM32H723VETx"
    }

    fn run(&self, ctx: &CreateContext) -> anyhow::Result<()> {
        info!("Running first script");
        let script = render_string(
            include_str!("../configs/create-script/STM32H723VETx/01.tmpl"),
            &ctx,
        )?;
        match run_script(script) {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to run first script: {}", e);
                return Err(anyhow!("Failed to run first script: {}", e));
            }
        };
        info!("Patching .ioc file");
        apply_patch(&Patch::RegexReplace {
            file: format!("{}.ioc", ctx.project_name),
            pattern: r"RCC\.HSE_VALUE=(\d+)".to_string(),
            insert: "RCC.HSE_VALUE=8000000".to_string(),
        })?;
        apply_patch(&Patch::RegexReplace {
            file: format!("{}.ioc", ctx.project_name),
            pattern: "(MMT.+\n)+".to_string(),
            insert: include_str!("../configs/create-script/STM32H723VETx/default.mmt.tmpl")
                .to_string(),
        })?;
        // 渲染第二次运行的脚本
        let script = render_string(
            include_str!("../configs/create-script/STM32H723VETx/02.tmpl"),
            &ctx,
        )?;
        info!("Running second script");
        match run_script(script) {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to run second script: {}", e);
                Err(anyhow!("Failed to run second script: {}", e))
            }
        }
    }
}
