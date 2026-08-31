use include_dir::{Dir, include_dir};

pub static DEFAULT_GITIGNORE_CONFIG_DIR: Dir<'static> = include_dir!("src/configs/gitignore");

pub mod cubemx_scripts {
    pub mod stm32f407vetx {
        pub const BOOTSTRAP: &str =
            include_str!("configs/stm32cubemx/scripts/stm32f407vetx/bootstrap.tmpl");
        pub const GENERATE: &str =
            include_str!("configs/stm32cubemx/scripts/stm32f407vetx/generate.tmpl");
    }

    pub mod stm32g474cbtx {
        pub const BOOTSTRAP: &str =
            include_str!("configs/stm32cubemx/scripts/stm32g474cbtx/bootstrap.tmpl");
    }

    pub mod stm32h723vetx {
        pub const BOOTSTRAP: &str =
            include_str!("configs/stm32cubemx/scripts/stm32h723vetx/bootstrap.tmpl");
        pub const DEFAULT_MEMORY_MAP: &str =
            include_str!("configs/stm32cubemx/scripts/stm32h723vetx/default_memory_map.tmpl");
        pub const GENERATE: &str =
            include_str!("configs/stm32cubemx/scripts/stm32h723vetx/generate.tmpl");
    }
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_GITIGNORE_CONFIG_DIR, cubemx_scripts};

    #[test]
    fn embedded_gitignore_configs_are_available() {
        assert_eq!(DEFAULT_GITIGNORE_CONFIG_DIR.files().count(), 4);
    }

    #[test]
    fn embedded_cubemx_scripts_are_available() {
        assert!(cubemx_scripts::stm32f407vetx::BOOTSTRAP.contains("project toolchain"));
        assert!(cubemx_scripts::stm32f407vetx::GENERATE.contains("project generate"));
        assert!(cubemx_scripts::stm32g474cbtx::BOOTSTRAP.contains("load STM32G474CBTx"));
        assert!(cubemx_scripts::stm32g474cbtx::BOOTSTRAP.contains("project generate"));
        assert!(cubemx_scripts::stm32h723vetx::BOOTSTRAP.contains("mmt load pre-config default"));
        assert!(cubemx_scripts::stm32h723vetx::DEFAULT_MEMORY_MAP.contains("MMTAppRegionsCount"));
        assert!(cubemx_scripts::stm32h723vetx::GENERATE.contains("project generate"));
    }
}
