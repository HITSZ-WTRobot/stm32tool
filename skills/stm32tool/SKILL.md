---
name: stm32tool
description: Use when an agent needs to operate the compiled `stm32tool` CLI to create, initialize, regenerate, clean STM32CubeMX CMake projects, or update the CLI itself. Covers command selection for `create`, `init`, `generate`, `purge`, and `update`; prerequisite checks such as `STM32CubeMX_PATH` or `stm32cubemx`; optional `cpkg` bootstrap behavior; and safe handling of `.ioc`, `UserCode/`, and generated files. Do not use this skill for modifying the `stm32tool` source repository itself.
---

# Use stm32tool

## Start Here

- Confirm that the task is about using the `stm32tool` executable, not changing this repository's source code.
- Prefer the installed `stm32tool` binary. Use `cargo run -- ...` only when the user explicitly wants to validate a source checkout instead of the compiled artifact.
- Run `stm32tool --help` and the relevant subcommand `--help` before using unfamiliar flags.
- Use [references/prerequisites.md](references/prerequisites.md) before the first run in a new environment.
- Use [references/command-recipes.md](references/command-recipes.md) to pick the right command and flags.
- Treat generated CubeMX output as generated. Prefer changing the `.ioc` file and rerunning generation instead of hand-editing `Core/`, `Drivers/`, or `Middlewares/`.

## Choose A Workflow

### Create A New Project

- Use `stm32tool create <PROJECT_NAME>` when the project directory does not exist yet.
- Expect an interactive MCU selection prompt. Use a TTY when driving this command from an agent session.
- Add `--run-init` to chain `init` immediately after project creation.
- Remember that `--skip-generate-user-code`, `--skip-generate-clang-format`, and `--force` on `create` only affect the forwarded `init` step when `--run-init` is present.

### Initialize An Existing Project

- Use `stm32tool init` from the project root after the `.ioc` file already exists.
- Use `--skip-generate-user-code` to leave `UserCode/` untouched.
- Use `--skip-generate-clang-format` to skip `.clang-format` generation.
- Use `--force` only when the user wants existing templated files to be overwritten.
- Expect `init` to run `git init`, generate `.gitignore`, scaffold `UserCode/`, patch `CMakeLists.txt`, attempt optional `cpkg` bootstrap, and try an initial Git commit.

### Regenerate CubeMX Output

- Use `stm32tool generate` from the project root after the `.ioc` file changes.
- Expect exactly one `.ioc` file in the working directory.
- On Windows, ensure `STM32CubeMX_PATH` points to the STM32CubeMX installation directory.
- On non-Windows, ensure the `stm32cubemx` command is available in `PATH`.

### Clean Ignored Output

- Use `stm32tool purge` carefully. It runs `git clean -fdX`, which removes ignored files and directories such as build output.
- Confirm the user really wants cleanup before invoking `purge` in a repository with local build artifacts.

### Update The Tool

- Use `stm32tool update --check` to report whether a newer GitHub Release exists without downloading.
- Use `stm32tool update` to confirm interactively, or `stm32tool update --yes` in non-interactive sessions.
- Expect `update` to refuse when the running binary lives in a Cargo `target/` directory; tell the user to install a release build instead.
- Expect `update` to report "up to date" without downloading when the local build is newer than the newest published release.

### Troubleshoot

- Add `-v` to any command when you need external command stdout and stderr.
- If `cpkg` is missing, explain that initialization still succeeds but `BasicComponents/utils` must be added manually for `UserCode/arena.cpp`.
- If cpkg is available, note that init asks whether to run cpkg sync; when the user declines or stderr is not a terminal, tell them to run cpkg sync manually.
- If `update` fails with a checksum or missing-digest error, tell the user the release asset could not be verified and to install manually rather than retrying blindly.
- If `generate` or `init` fails because `.ioc` discovery is ambiguous, report that zero or multiple `.ioc` files are present and resolve that first.

## Report Clearly

- State which command you ran and from which directory.
- Call out whether the run created or modified `.gitignore`, `.clang-format`, `UserCode/`, `CMakeLists.txt`, or `cmake/wtr_modules.cmake`.
- Mention any follow-up step the user still needs to do, especially `cpkg sync`, fixing environment variables, or restarting a shell after installation.
