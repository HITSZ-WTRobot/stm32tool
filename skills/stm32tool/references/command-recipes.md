# Command Recipes

## Inspect Available Commands

- `stm32tool --help`
- `stm32tool init --help`
- `stm32tool create --help`
- `stm32tool generate --help`
- `stm32tool purge --help`
- `stm32tool update --help`

## Create A New Project

- Use `stm32tool create <PROJECT_NAME>` for a brand-new project directory; it creates the directory and immediately performs initialization.
- Add `--skip-init` to create the project skeleton only, without initialization.
- `--run-init` still parses for backward compatibility but only logs a warning that initialization already runs by default.
- If you also pass `--skip-generate-user-code`, `--skip-generate-clang-format`, or `--force`, those flags apply to the `init` step that runs by default; they are ignored when `--skip-init` is present.

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

## Update The Tool

- Use `stm32tool update --check` when the user only wants to know whether a newer release exists.
- Use `stm32tool update --yes` in agent sessions, because the interactive prompt requires a TTY.
- Only Windows and Linux release assets exist; other platforms must update manually.

## Expected Side Effects

- `create`: creates the project directory, writes the `.ioc`, generates the selected STM32CubeMX project skeleton, and then runs the `init` side effects below unless `--skip-init` is given.
- `init`: initializes Git, writes `.gitignore`, optionally writes `.clang-format`, optionally writes `UserCode/app.cpp` and `UserCode/arena.cpp`, patches `CMakeLists.txt`, and optionally bootstraps `cpkg`.
- `generate`: reruns STM32CubeMX code generation with the CMake toolchain.
- `purge`: deletes ignored files and directories from the repository.
- `update`: replaces the running `stm32tool` executable with the newest published release for the current platform, after verifying its sha256 digest. It refuses to touch binaries under a Cargo `target/` directory.
