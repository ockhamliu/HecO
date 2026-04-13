# AGENTS.md - OpenCode Instructions for heco

## Project Overview
Rust CLI tool for creating and building HarmonyOS projects using `clap`.

## Structure
```
heco/
├── src/
│   ├── main.rs      # CLI entry, command dispatcher
│   ├── new.rs       # heco new subcommand (dialoguer prompts)
│   ├── build.rs     # heco build subcommand
│   ├── run.rs       # heco run subcommand
│   ├── check.rs     # heco check subcommand
│   ├── test.rs      # heco test subcommand
│   └── setup.rs     # heco setup subcommand (interactive config)
```

## Commands
- **Build**: `cargo build`
- **Run**: `cargo run -- <command>`

## Subcommands
- `heco new <path> [options]` - Create new HarmonyOS project
- `heco build [options]` - Build project modules
- `heco run [options]` - Run on device
- `heco check [options]` - Lint and type check
- `heco test [options]` - Run tests
- `heco setup [scope]` - Configure dev environment (global/project)

## Common Patterns
- Subcommand handlers: `pub(crate) fn handle_<cmd>(args: <Args>)`
- CLI definitions: `#[derive(Parser, Debug)]` + `#[command(name)]` for clap
- Optional params: `Option<String>` or typed (e.g., `Option<u32>`)
- `--quiet` flag: check, run, test subcommands
- Avoid duplicate imports from same module (E0252)

## Dependencies
- `clap` (derive) - CLI parsing
- `anyhow` - Error handling
- `dialoguer` - Interactive prompts
- `toml`, `serde` - Config serialization (TOML v1 format)
- `dirs` - Platform config directories

## Config System
- Global base: `~/.config/heco/config.toml`
- Project override: `.heco/config.toml`
- Setup prompts for `deveco_studio_root`, `command_line_tools_root`, `java_home`
- Java auto-detection via `JAVA_HOME` and PATH

## Notes
- `setup_design.md` documents the config design specification
- HarmonyOS-specific knowledge comes from MCP server in `opencode.json`
