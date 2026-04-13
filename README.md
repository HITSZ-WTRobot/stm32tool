# stm32tool

本工具遵照 [STM32 + Git 多人合作方案](https://syhanjin.moe/20250908/698d9cb67753/) 初始化 STM32 CMake 项目结构。

当前仅支持 STM32CubeMX 的 CMake 工具链；不再提供 Makefile、STM32CubeIDE、Keil、IAR 等非 CMake 生成架构的初始化或生成入口。纯 CMake 项目应保留并跟踪 `CMakeLists.txt`。

**Windows 下需要配置环境变量 `STM32CubeMX_dir` 为 `STM32CubeMX` 的安装路径**

## Help

```
STM32 CMake project helper tool

Usage: stm32tool <COMMAND>

Commands:
  init      初始化 STM32 CMake 项目
  create    创建新的 STM32 CMake 项目
  purge     清除生成的代码和构建文件
  generate  使用 STM32CubeMX 生成 CMake 代码
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```
