# Command Recipes

## Inspect Available Commands

- `stm32tool --help`
- `stm32tool init --help`
- `stm32tool create --help`
- `stm32tool generate --help`
- `stm32tool purge --help`

## Create A New Project

- Use `stm32tool create <PROJECT_NAME>` for a brand-new project directory.
- Use `stm32tool create <PROJECT_NAME> --run-init` to create the project and immediately perform initialization.
- If you also pass `--skip-generate-user-code`, `--skip-generate-clang-format`, or `--force`, those flags only apply to the forwarded `init` step.

## Initialize An Existing Project

- Use `stm32tool init` from the project root after the `.ioc` file is already present.
- Add `--skip-generate-user-code` when the agent must not touch `UserCode/`.
- Add `--skip-generate-clang-format` when the agent must not write `.clang-format`.
- Add `--force` only when overwriting templated files is intended.

## Regenerate After `.ioc` Changes

- Use `stm32tool generate` from the project root.
- Fix missing or ambiguous `.ioc` files before retrying.
- When generation fails on Windows, check `STM32CubeMX_PATH`.
- When generation fails on Linux or macOS, check whether `stm32cubemx` is available in `PATH`.

## Clean Ignored Build Output

- Use `stm32tool purge` only when the user wants to remove ignored files and directories.
- Remember that `purge` maps to `git clean -fdX`.

## Expected Side Effects

- `create`: creates the project directory, writes the `.ioc`, and generates the selected STM32CubeMX project skeleton.
- `init`: initializes Git, writes `.gitignore`, optionally writes `.clang-format`, optionally writes `UserCode/app.cpp` and `UserCode/arena.cpp`, patches `CMakeLists.txt`, and optionally bootstraps `cpkg`.
- `generate`: reruns STM32CubeMX code generation with the CMake toolchain.
- `purge`: deletes ignored files and directories from the repository.
