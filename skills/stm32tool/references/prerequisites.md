# Prerequisites

## Binary Availability

- Prefer the installed `stm32tool` executable.
- Confirm the binary is reachable with `stm32tool --help` before using it in automation.
- Use `cargo run -- --help` only when the user explicitly wants to run from a source checkout.

## Platform Requirements

- Windows: set `STM32CubeMX_PATH` to the STM32CubeMX installation directory before running `generate` or workflows that invoke CubeMX.
- Non-Windows: ensure the `stm32cubemx` command is available in `PATH`.
- All platforms: this tool only supports the STM32CubeMX CMake workflow. Do not expect Makefile, Keil, IAR, or STM32CubeIDE project generation.

## Project Layout Expectations

- `init` and `generate` expect exactly one `.ioc` file in the current working directory.
- `create` creates a new project directory and then asks the user to choose a supported MCU interactively.
- Generated application code lives in `UserCode/`.
- Generated HAL and middleware output should be treated as regenerated assets, not primary edit targets.

## Optional Tools

- `git` is expected for repository initialization and the initial commit attempt performed by `init`.
- `cpkg` is optional:
  - When present, `init` runs `cpkg init` and `cpkg add --offline utils`, then asks whether to run `cpkg sync` (the prompt is skipped when stderr is not a terminal).
  - When absent, `init` still succeeds, but `BasicComponents/utils` must be integrated manually for `UserCode/arena.cpp`.
