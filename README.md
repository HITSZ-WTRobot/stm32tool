# stm32tool

本工具遵照 [STM32 + Git 多人合作方案](https://syhanjin.moe/20250908/698d9cb67753/) 初始化项目结构

**Windows 下需要配置环境变量 `STM32CubeMX_dir` 为 `STM32CubeMX` 的安装路径**

## Help

```
STM32 project helper tool

Usage: stm32tool <COMMAND>

Commands:
  init      初始化 STM32 项目
  create    创建新项目
  purge     清除生成的代码和构建文件
  generate  生成代码
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```