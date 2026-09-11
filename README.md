# stm32tool

本工具遵照 [STM32 + Git 多人合作方案](https://syhanjin.moe/20250908/698d9cb67753/) 初始化 STM32 CMake 项目结构。

当前仅支持 STM32CubeMX 的 CMake 工具链；不再提供 Makefile、STM32CubeIDE、Keil、IAR 等非 CMake 生成架构的初始化或生成入口。纯 CMake 项目应保留并跟踪 `CMakeLists.txt`。

**Windows 下需要配置环境变量 `STM32CubeMX_PATH` 为 `STM32CubeMX` 的安装路径**

## 项目结构

- `src/commands/`：各个 CLI 子命令的执行入口，按命令拆分，避免逻辑堆在单文件中。
- `src/configs/`：内置配置资产；其中 `gitignore/` 存放 `.gitignore` 片段，`stm32cubemx/scripts/` 存放按 MCU 分类的 CubeMX 脚本模板。
- `src/templates/`：项目初始化时写入目标工程的文本模板。
- `src/configs.rs` 与 `src/templates.rs`：统一管理内置资产入口，业务代码不再直接引用深层相对路径。

## 生成结果

- 默认生成 C++17 的 STM32CubeMX CMake 工作区，并启用 `C CXX ASM` 语言。
- `UserCode/` 默认只生成 `app.cpp` 与 `arena.cpp` 两个源文件。
- 若环境中存在 `cpkg`，初始化时会静默调用 `cpkg init` 与 `cpkg add --offline utils`，并在根 `CMakeLists.txt` 中接入 `cmake/wtr_modules.cmake`；随后会询问是否立即执行 `cpkg sync`，在非交互环境（stderr 未连接终端）下跳过询问并提示手动执行。
- 若环境中缺少 `cpkg`，工具会继续初始化，但需要手动将 `BasicComponents/utils` 加入项目，以满足 `UserCode/arena.cpp` 的依赖。

## Help

```
STM32 CMake project helper tool

Usage: stm32tool [OPTIONS] <COMMAND>

Commands:
  init      初始化 STM32 CMake 项目
  create    创建新的 STM32 CMake 项目
  purge     清除生成的代码和构建文件
  generate  使用 STM32CubeMX 生成 CMake 代码
  help      Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose  输出调试信息，包括外部命令的 stdout/stderr
  -h, --help     Print help
```
