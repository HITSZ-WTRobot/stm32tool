use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "stm32-project-tool")]
#[command(about = "STM32 CMake project helper tool", long_about = None)]
pub struct Cli {
    /// 输出调试信息，包括外部命令的 stdout/stderr
    #[arg(short, long, global = true, default_value_t = false)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 初始化 STM32 CMake 项目
    Init(InitArgs),

    /// 创建新的 STM32 CMake 项目
    Create(CreateArgs),

    /// 清除生成的代码和构建文件
    ///
    /// 运行 git clean -fdX
    Purge,

    /// 使用 STM32CubeMX 生成 CMake 代码
    Generate,
}

#[derive(Parser, Debug)]
pub struct CreateArgs {
    /// 项目名
    pub project_name: String,

    /// 是否在创建后立即初始化项目
    #[arg(long)]
    pub run_init: bool,

    /// 使用 init 的参数
    #[command(flatten)]
    pub init_args: InitArgs,
}

#[derive(Parser, Debug)]
pub struct InitArgs {
    /// 跳过生成 UserCode C++ 源文件
    #[arg(long, default_value_t = false)]
    pub skip_generate_user_code: bool,

    /// 跳过生成 .clang-format
    #[arg(long, default_value_t = false)]
    pub skip_generate_clang_format: bool,

    /// 强制重新生成
    #[arg(long)]
    pub force: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_verbose_before_subcommand() {
        let cli = Cli::try_parse_from(["stm32tool", "-v", "purge"]).unwrap();

        assert!(cli.verbose);
        assert!(matches!(cli.command, Commands::Purge));
    }

    #[test]
    fn parses_verbose_after_subcommand() {
        let cli = Cli::try_parse_from(["stm32tool", "purge", "-v"]).unwrap();

        assert!(cli.verbose);
        assert!(matches!(cli.command, Commands::Purge));
    }
}
