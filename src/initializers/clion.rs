use crate::initializers::{FPUType, IdeInitArgs, IdeInitializer};
use crate::patches::{Patch, apply_patch};
use crate::stm32cubemx::{Toolchain, generate_code};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tracing::{info, warn};

pub struct CLion;

impl IdeInitializer for CLion {
    fn name(&self) -> &'static str {
        "CLion (toolchain: STM32CubeIDE)"
    }

    fn init(&self, args: &IdeInitArgs, _force: bool) -> anyhow::Result<()> {
        info!("Initializing CLion project...");

        let mut template_exists: bool = true;

        if !Path::new("CMakeLists_template.txt").exists() {
            template_exists = false;
            File::create("CMakeLists_template.txt")?
                .write_all(include_str!("../templates/clion-cmakelists-template.tmpl").as_ref())?;
            // error!("CMakeLists_template.txt is not exists, initialization failed");
            // return Err(anyhow!(
            //     "CMakeLists_template.txt is not exists, initialization failed"
            // ));
        }

        apply_patch(&Patch::Replace {
            file: "CMakeLists_template.txt".to_string(),
            find: "include_directories(${includes})".to_string(),
            insert: "include_directories(${includes} UserCode)".to_string(),
        })?;
        apply_patch(&Patch::Replace {
            file: "CMakeLists_template.txt".to_string(),
            find: "file(GLOB_RECURSE SOURCES ${sources})".to_string(),
            insert: "file(GLOB_RECURSE SOURCES ${sources} \"UserCode/*.*\")".to_string(),
        })?;
        match args.fpu {
            FPUType::Hard => apply_patch(&Patch::UncommentBlock {
                file: "CMakeLists_template.txt".to_string(),
                marker: "#Uncomment for hardware floating point".to_string(),
            }),
            FPUType::Soft => apply_patch(&Patch::UncommentBlock {
                file: "CMakeLists_template.txt".to_string(),
                marker: "#Uncomment for software floating point".to_string(),
            }),
        }?;
        if template_exists {
            // 原本存在 CMakeLists_template.txt，应该处于 CLion 环境下，尝试重生成
            info!("Try to regenerate code(using STM32CubeMX)...");
            match generate_code(Some(Toolchain::STM32CubeIDE)) {
                Ok(_) => {
                    info!("Regenerate code successfully!")
                }
                Err(_) => {
                    warn!("Regenerate code failed, please regenerate code manually!");
                }
            };
        }
        Ok(())
    }
}
