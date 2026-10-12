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
- `UserCode/` 默认生成 `app.cpp`；仅当项目唯一 `.ioc` 文件声明 `Mcu.CPN=STM32H723VET6` 时，另生成 `memory.hpp`，不按 H7 系列或 `Mcu.Name=STM32H723VETx` 通配名称判断。
- 工具内部维护初始依赖表，默认依赖为 `memory::StaticObject`；若环境中存在 `cpkg`，初始化时会静默调用 `cpkg init` 与 `cpkg add --offline memory::StaticObject`，并在根 `CMakeLists.txt` 中接入 `cmake/wtr_modules.cmake`；随后会询问是否立即执行 `cpkg sync`，在非交互环境（stderr 未连接终端）下跳过询问并提示手动执行。
- 若环境中缺少 `cpkg`，工具会继续初始化，但需要手动将初始依赖表中的包（默认 `memory::StaticObject`）加入项目。
- 默认 `.gitignore` 忽略 CubeMX 生成的 `cmake/stm32cubemx/`；`cmake/gcc-arm-none-eabi.cmake` 保持跟踪，并在 `init` 时把 C/C++ 的 Debug 优化档由 `-O0 -g3` 改写为 `-O2 -g3`。

### STM32H723VET6 内存段头文件

`UserCode/memory.hpp` 提供 `MEM_DTCMRAM`、`MEM_RAM_AXI`、`MEM_RAM_D2`、`MEM_RAM_D3`、`MEM_ITCMRAM`、`MEM_BKUPRAM`；`MEM_DMA` 指向 AXI RAM，`MEM_DEFAULT` 指向 DTCMRAM。匹配该芯片时会输出 `Detected STM32H723VET6 MCU, generating UserCode/memory.hpp`。

- 默认 `create` 和直接 `init` 都会为 STM32H723VET6 生成；其他芯片不生成。`--skip-generate-user-code` 同时跳过源文件和头文件，已有文件仅在 `--force` 时覆盖。
- 宏仅设置 section，适用于全局或 static 变量；专用段首次读取前须显式写入，DMA 可达性、Cache 一致性和备份 RAM 供电由调用方处理。
- 头文件按 STM32H723VET6 使用的 `STM32H723xx_FLASH.ld` section 名称生成，不修改链接脚本。
- 模板注释保留 `.data/.bss` 位于 DTCMRAM 的使用场景；仓库当前 H723 内存配置将 `RAM` 标为默认数据区，实际放置应以项目链接脚本为准。

## Help

```
STM32 CMake project helper tool

Usage: stm32tool [OPTIONS] <COMMAND>

Commands:
  init      初始化 STM32 CMake 项目
  create    创建新的 STM32 CMake 项目
  purge     清除生成的代码和构建文件
  generate  使用 STM32CubeMX 生成 CMake 代码
  update    更新 stm32tool 到最新版本
  help      Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose  输出调试信息，包括外部命令的 stdout/stderr
  -h, --help     Print help
```
